# Scabby

Self-hosted Git hosting, built from scratch: a Rust backend and a Svelte user interface compiled
into a single binary. Apache License 2.0.

Repository: <https://github.com/HuntedByTheIRS/scabby>

## What it is

Scabby keeps bare Git repositories on disk and serves them over HTTP. A browser and a `git` client
talk to the same server: the Svelte app is served from the binary, and `git clone` and `git push`
go through the smart-HTTP endpoints the same binary exposes. There is no separate frontend server
to deploy and no external identity provider to configure.

## Features

- Bare repository hosting on disk, created through the API or the UI.
- Clone over HTTP for public repositories, push with token authentication.
- Repository browsing: file tree at a ref, README rendering, commit list, and a per-commit view
  with the changed files and their added and deleted line counts.
- Issues with comments, open and closed state, and per-repository numbering.
- User profiles and a dashboard listing repositories.
- Settings for personal access tokens and SSH public keys, with the SHA-256 fingerprint shown for
  each key.
- Password sign-in with argon2id hashing and server-side sessions.
- A REST API at `/api/v1` that the UI itself uses, so anything the browser can do a script can do.

## Requirements

- Rust 1.90 or later (the workspace uses the 2024 edition).
- Node 22 or later and npm, for the Svelte app.
- PostgreSQL 18, for the runtime database.
- `git` on `PATH` at runtime. Clone and push are served by running `git upload-pack --stateless-rpc`
  and `git receive-pack --stateless-rpc`, so the binary needs the real `git` executable.

## Quickstart

Start the development database:

```sh
docker compose up -d
```

Point the server at it and run it:

```sh
export SCABBY_DATABASE_URL=postgres://scabby:scabby@localhost:5432/scabby
cargo run -p scabby -- serve
```

Open <http://localhost:3000> and register. The first account registered becomes the administrator.

`docker compose up -d` brings up the single PostgreSQL 18 instance in `docker-compose.yml` as user
`scabby`, password `scabby`, database `scabby`, published on localhost:5432. Those credentials are
development throwaways and match the built-in default for `database_url`, so exporting the variable
is optional on a default setup — it is spelled out above because anything other than a throwaway
password belongs in the environment.

## Build from source

The user interface must be built before the Rust build if you want it embedded:

```sh
cd web
npm install
npm run build
cd ..
cargo build --release
```

The binary embeds `web/build` with `rust-embed` at compile time. `cargo build` succeeds even when
that directory does not exist: the build script writes a placeholder page, and a server started from
that binary serves a page telling the visitor to run `npm install && npm run build` in `web/` and
rebuild. The API works either way.

### Development loop

Run the SvelteKit dev server and point the backend at the build output directory instead of the
embedded copy:

```sh
cargo run -p scabby -- serve --frontend-dir web/build
```

```sh
cd web
npm run dev
```

`--frontend-dir` swaps where the UI comes from; the API does not change with it, so the flag is also
what a custom frontend uses.

## Command line

```text
scabby serve                     start the HTTP server (also the default when no subcommand is given)
scabby migrate                   run database migrations and exit
scabby admin create-user --username <NAME> --email <ADDRESS> [--admin] [--password-stdin]
scabby hook post-receive --repo <OWNER/NAME>   internal; invoked by the git hook installed in each bare repository, not by people directly
scabby --version / --help
```

`scabby hook post-receive` is called by the hook Scabby installs inside each bare repository when a
push updates it. It is documented here so the command is not a mystery in a process list; it is not
meant to be run by hand.

## Configuration

Every setting has a built-in default, an environment variable, a flag, and a key in the TOML config
file. A flag beats an environment variable, and both beat the config file.

| flag | environment | config key | default |
| --- | --- | --- | --- |
| `-c, --config <FILE>` | `SCABBY_CONFIG` | — | none |
| `--listen <ADDR>` | `SCABBY_LISTEN` | `listen` | `127.0.0.1:3000` |
| `--database-url <URL>` | `SCABBY_DATABASE_URL` | `database_url` | `postgres://scabby:scabby@localhost:5432/scabby` |
| `--data-dir <PATH>` | `SCABBY_DATA_DIR` | `data_dir` | `./data` |
| `--base-url <URL>` | `SCABBY_BASE_URL` | `base_url` | `http://localhost:3000` |
| `--frontend-dir <PATH>` | `SCABBY_FRONTEND_DIR` | `frontend_dir` | embedded UI |
| `--registration <open\|closed>` | `SCABBY_REGISTRATION` | `registration` | `open` |
| `--git <PATH>` | `SCABBY_GIT` | `git` | `git` from `PATH` |
| `--log <FILTER>` | `SCABBY_LOG` | `log` | none |
| `--max-connections <N>` | `SCABBY_MAX_CONNECTIONS` | `max_connections` | none |

Precedence, in order: built-in defaults, then the TOML file, then environment variables, then flags.
Unknown keys in the TOML file are an error rather than a silent no-op, so a typo in a key name is
reported at startup instead of leaving a setting at its default.

`scabby.toml.example` and `.env.example` are copies of the defaults to start from. A real
`scabby.toml` holds `database_url` and is therefore a secret; keep it out of the repository.

## API

The REST API lives under `/api/v1` and is documented in [docs/api.md](docs/api.md). The
documentation is the contract: resource paths, request and response shapes, status codes, the error
envelope, and the deviations from GitHub's conventions are all listed there.

A worked example. Register a user and keep the session cookie:

```sh
curl -sS -c cookies.txt -X POST http://localhost:3000/api/v1/auth/register \
  -H 'content-type: application/json' \
  -d '{"username":"ada","email":"ada@example.com","password":"a-long-enough-password"}'
```

Create a personal access token with that session:

```sh
curl -sS -b cookies.txt -X POST http://localhost:3000/api/v1/user/tokens \
  -H 'content-type: application/json' \
  -d '{"name":"laptop","scopes":["repo","user"],"expires_in_days":30}'
```

The response contains the token in `plaintext`, and that is the only time it is returned — the server
stores a SHA-256 hash plus a short prefix for display. Use it as a bearer token:

```sh
curl -sS -H "Authorization: Bearer $SCABBY_TOKEN" http://localhost:3000/api/v1/user
```

The same token is the password when `git` prompts for credentials:

```sh
git clone http://localhost:3000/ada/scabby.git
```

## Project layout

| path | what is in it |
| --- | --- |
| `crates/scabby-core` | Domain types, identifiers, errors, configuration, and the secret hashing helpers. No IO. |
| `crates/scabby-db` | PostgreSQL via sqlx: the connection pool, embedded migrations, and one module per resource. |
| `crates/scabby-git` | The bare repository store, commit and tree reads through gix, diffs, hooks, and the smart-HTTP transport. |
| `crates/scabby-web` | The axum router, session auth, the `/api/v1` REST API, the git HTTP endpoints, and the embedded UI. |
| `crates/scabby` | The binary: argument parsing, configuration resolution, and wiring the other crates into `serve`. |
| `crates/scabby-db/migrations` | Numbered SQL migrations, embedded in the binary. |
| `web/` | The SvelteKit 2 / Svelte 5 single-page app, built with `adapter-static` into `web/build`. |
| `docs/api.md` | The REST API contract. |
| `DESIGN.md` | The design contract for the UI: palette, typography, layout, components, and budgets. |

## Contributing and security

[CONTRIBUTING.md](CONTRIBUTING.md) covers setup, the local check list, and commit conventions.
Report vulnerabilities privately as described in [SECURITY.md](SECURITY.md).

## License

Apache License 2.0. See [LICENSE](LICENSE).
