# Scabby REST API v1

Everything the web UI does goes through this API, so a script can do it too. The shape follows
GitHub's REST conventions (snake_case fields, `/repos/{owner}/{repo}/...` resource nesting,
`Link` headers for pagination, and `Authorization: Bearer` for tokens), with the documented deviations
listed at the bottom.

Base path: `/api/v1`. All bodies are JSON. Timestamps are RFC 3339 in UTC.

## Authentication

Three ways in, checked in this order:

1. `Authorization: Bearer scabby_pat_...` — a personal access token. Also accepts GitHub's
   `Authorization: token scabby_pat_...` form.
2. `Authorization: Basic base64(<anything>:scabby_pat_...)` — for git clients, which cannot send
   a bearer token.
3. The `scabby_session` cookie, set by login and register. Browser UI only.

Unauthenticated read access is allowed for public resources. Private resources return `404` to
an unauthorized caller rather than `403`, so the API does not disclose that a private repo exists.

Tokens are created with `POST /api/v1/user/tokens` and returned in plaintext exactly once. The
server stores only a SHA-256 hash and the first characters of the token as a prefix for display.

## Errors

Every failure answers with the same envelope: `{"message", "status", "errors"}`. A path under
`/api/v1` that matches no route is a JSON 404 with `resource: "endpoint"` in `errors`; it is not
served the single-page app, so a mistyped endpoint is never a `200` with an HTML body. A path outside
the prefix is the app, and an unsupported method on a real route is a plain `405`.

## Errors

```json
{
  "message": "Not Found",
  "status": 404,
  "errors": [
    { "resource": "repository", "field": "name", "code": "not_found" }
  ]
}
```

`errors` is present only for validation failures. Validation failures return `422`, except a
duplicate name, which returns `409`.

| status | meaning |
| --- | --- |
| 401 | missing or invalid credentials |
| 403 | authenticated but not allowed (non-owner write) |
| 404 | no such resource, or a private one you cannot see |
| 409 | conflict (duplicate username, repo, token prefix) |
| 422 | validation failure, detail in `errors[]` |
| 429 | reserved; no rate limiting in v0 |

## Pagination

Collection endpoints take `?page=1&per_page=30`. `per_page` clamps to 1..100, default 30.

The body is an envelope (see deviations):

```json
{ "items": [], "page": 1, "per_page": 30, "total": 0, "total_pages": 0 }
```

A `Link` header carries `rel="next"`, `"prev"`, `"first"`, `"last"`, as GitHub does.

Lists that cannot be long enough to page are not wrapped: `GET .../branches` and `GET .../tags`
answer `{items:[...]}` with no page metadata, and `GET /user/tokens` and `GET /user/keys` answer a
bare JSON array. A repository has a handful of branches and a person has a handful of tokens, so page
arithmetic over them would be noise rather than information. A directory listing
(`GET .../contents/{path}`) is a different shape again: `{type, path, entries}`. A client that
handles collections generically should accept all of these.

A list whose total was not counted sends `"total": null` and `"total_pages": null`, and its `Link`
header offers only `rel="next"`, so a client learns that another page exists without being told how
many there are. The commit list is the one endpoint that does this: counting a large history is
work nobody asked for.

## Meta

| method | path | notes |
| --- | --- | --- |
| GET | `/api/v1/version` | `{"name":"scabby","version":"0.1.0","api_version":"v1"}` |
| GET | `/api/v1/health` | `{"status":"ok","database":"ok","uptime_seconds":12}` |
| GET | `/healthz` | unversioned, plain `ok`, for load balancers |

## Auth and session

| method | path | body | notes |
| --- | --- | --- | --- |
| POST | `/auth/register` | `{username, email, password}` | 201, sets the session cookie; 403 when registration is closed |
| POST | `/auth/login` | `{username_or_email, password}` | 200, sets the cookie; 401 on bad credentials |
| POST | `/auth/logout` | – | 204, clears the cookie and deletes the session row |
| GET | `/auth/session` | – | 200 `{user}` when signed in, 401 otherwise |

## User

| method | path | notes |
| --- | --- | --- |
| GET | `/user` | the authenticated user, including `email` |
| PATCH | `/user` | `{display_name?, bio?}` |
| GET | `/users/{username}` | public profile |
| GET | `/users/{username}/repos` | that user's visible repositories; `?stats=true` as above |

```json
{
  "id": 1,
  "username": "ada",
  "display_name": "Ada",
  "email": "ada@example.com",
  "bio": null,
  "avatar_url": null,
  "is_admin": false,
  "created_at": "2026-10-15T18:00:00Z"
}
```

`email` is present only on your own user object.

## Tokens and keys

| method | path | notes |
| --- | --- | --- |
| GET | `/user/tokens` | never returns the secret |
| POST | `/user/tokens` | `{name, scopes?: ["repo","user"], expires_in_days?: 30}` → 201 `{token, plaintext}` |
| DELETE | `/user/tokens/{id}` | 204 |
| GET | `/user/keys` | SSH public keys, bare array |
| POST | `/user/keys` | `{title, key}` → 201, `fingerprint` computed as `SHA256:<base64>` |
| DELETE | `/user/keys/{id}` | 204 |

Scope strings are recorded and displayed but not enforced in v0; a token is either valid or
revoked. That is a known gap, not a promise.

A key object is `{id, title, public_key, fingerprint, created_at, last_used_at}`: the field is named
`public_key` on the way out even though the create request calls it `key`, and the fingerprint is
computed by the server from the pasted key as `SHA256:<base64>`.

## Activity

| method | path | notes |
| --- | --- | --- |
| GET | `/events` | the instance feed, newest first; requires credentials because entries name repositories |

`?stats=true` on a repository list is the one place Scabby spends CPU to answer a request: each
repository on the page is opened and walked on a rayon pool, in parallel, so a dashboard shows
branch counts and last commits without a request per card.

## Repositories

| method | path | notes |
| --- | --- | --- |
| GET | `/repos` | public repositories, newest first, plus the caller's own private ones; `?stats=true` adds a `git` block to each item |
| POST | `/user/repos` | `{name, description?, private?}` → 201, creates the bare repo on disk |
| GET | `/repos/{owner}/{repo}` | includes a `git` block when the on-disk repo exists |
| PATCH | `/repos/{owner}/{repo}` | `{description?, private?, default_branch?}`, owner only |
| DELETE | `/repos/{owner}/{repo}` | 204, owner only, removes the directory |
| GET | `/repos/{owner}/{repo}/branches` | `{items:[Branch]}` |
| GET | `/repos/{owner}/{repo}/tags` | `{items:[Tag]}` |
| GET | `/repos/{owner}/{repo}/stats` | branch count, tag count, size on disk, commit count |

```json
{
  "id": 3,
  "owner": { "id": 1, "username": "ada", "display_name": "Ada" },
  "name": "scabby",
  "full_name": "ada/scabby",
  "description": "Git hosting, self-hosted",
  "visibility": "public",
  "default_branch": "main",
  "size_bytes": 14832,
  "open_issue_count": 4,
  "clone_url": "http://localhost:3000/ada/scabby.git",
  "html_url": "http://localhost:3000/ada/scabby",
  "created_at": "2026-10-15T18:00:00Z",
  "updated_at": "2026-10-15T18:20:00Z"
}
```

## Commits and contents

| method | path | notes |
| --- | --- | --- |
| GET | `/repos/{o}/{r}/commits` | `?ref=main&page=1&per_page=30`; `total` and `total_pages` are usually `null` |
| GET | `/repos/{o}/{r}/commits/{sha}` | adds `body`, `files[]`, and `stats` |
| GET | `/repos/{o}/{r}/contents/{path}` | `?ref=main`; directory listing or file object |
| GET | `/repos/{o}/{r}/raw/{ref}/{path}` | raw bytes with a guessed content type |
| GET | `/repos/{o}/{r}/readme` | `?ref=main`; `{path, text}` or 404 |

```json
{
  "sha": "8a5f...",
  "short_sha": "8a5f3c1",
  "message": "Add repository creation",
  "author": { "name": "Ada", "email": "ada@example.com", "when": "2026-10-15T18:00:00Z" },
  "committer": { "name": "Ada", "email": "ada@example.com", "when": "2026-10-15T18:00:00Z" },
  "parents": ["1b2c..."],
  "when": "2026-10-15T18:00:00Z"
}
```

A file object carries `content` (base64, always) plus `text` (decoded UTF-8, `null` when the file
is binary) and a `binary` flag. A directory listing carries `entries[]` with
`{type, name, path, size}` where `type` is `file`, `dir`, `symlink`, or `submodule`.

Commit `files[]` entries are `{path, status, additions, deletions, binary}` with `status` in
`added`, `modified`, `deleted`, `renamed`. Binary files report zero line counts.

v0 returns per-file counts and not unified diffs: `files[]` carries no `patch`, so the commit page
shows which files moved and by how much rather than the changed lines. Rendering a real diff means
either adding a `patch` field here or having the client fetch both blobs through
`/contents/{path}?ref=` and diff them; neither is implemented yet, and the API does not pretend
otherwise.

## Issues

| method | path | notes |
| --- | --- | --- |
| GET | `/repos/{o}/{r}/issues` | `?state=open|closed|all` (default `open`) |
| POST | `/repos/{o}/{r}/issues` | `{title, body?}` → 201 |
| GET | `/repos/{o}/{r}/issues/{number}` | |
| PATCH | `/repos/{o}/{r}/issues/{number}` | `{title?, body?, state?}`, author or repo owner |
| GET | `/repos/{o}/{r}/issues/{number}/comments` | |
| POST | `/repos/{o}/{r}/issues/{number}/comments` | `{body}` → 201 |

```json
{
  "id": 12,
  "number": 7,
  "title": "Clone URL is wrong behind a proxy",
  "body": "...",
  "state": "open",
  "author": { "id": 1, "username": "ada" },
  "comment_count": 2,
  "html_url": "http://localhost:3000/ada/scabby/issues/7",
  "created_at": "2026-10-15T18:00:00Z",
  "updated_at": "2026-10-15T18:05:00Z",
  "closed_at": null
}
```

Issue numbers are per repository and assigned by the server under a row lock, so two concurrent
creates cannot take the same number.

## Git over HTTP

Smart HTTP only; the dumb protocol is not served.

```
GET  /{owner}/{repo}.git/info/refs?service=git-upload-pack
POST /{owner}/{repo}.git/git-upload-pack
POST /{owner}/{repo}.git/git-receive-pack
```

Fetch works anonymously on public repositories. Push requires ownership and a token: git will
prompt for a username and password, and the password is the token. Credentials are checked before
the request body is streamed to `git receive-pack`.

The server runs `git upload-pack --stateless-rpc` and `git receive-pack --stateless-rpc` per
request, streaming the body in and the response out, so a host must have `git` on `PATH`.

## Deviations from GitHub

1. Collections return an envelope (`items`, `page`, `per_page`, `total`, `total_pages`) instead of
   a bare array. The `Link` header is still there for clients that follow GitHub's style.
2. `/auth/*` exists, because Scabby has no external identity provider. GitHub has no login
   endpoint to copy.
3. File objects carry both `content` (base64) and `text` (decoded), so the UI does not have to
   decode on the client.
4. There is no GraphQL API, no webhooks, and no rate limiting in v0. All three are wanted; none
   are claimed.
