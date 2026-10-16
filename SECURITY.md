# Security policy

## Reporting a vulnerability

Report security problems privately. Do not open a public issue, pull request, or discussion for
them.

- **Preferred: a GitHub security advisory** on this repository —
  <https://github.com/HuntedByTheIRS/scabby/security/advisories/new>. This keeps the report, the
  discussion, and the eventual fix in one private place and gives us a coordinated-disclosure
  workflow.
- **Fallback: email** <huntedbyth3irs@gmail.com> if you cannot use GitHub advisories for any reason.

Include what you can: the version or commit you tested, what you did, what happened, what you
expected, and the smallest reproduction you have. A proof of concept is welcome but not required —
a clear description of the flaw is more useful than a large script.

## What to expect

1. **Acknowledgement.** We will confirm we received the report, usually within three business days.
2. **Assessment.** We will reproduce it and tell you whether we consider it a vulnerability, and how
   severe we think it is. Expect an initial assessment within ten business days; if it takes longer,
   we will say so rather than go quiet.
3. **Fix.** We will develop and test a fix and tell you when it is ready. Fixes land on `main` and
   are noted in the release they ship in.
4. **Disclosure.** We will coordinate the public write-up with you and credit you by name or handle
   unless you prefer otherwise. Our target is disclosure ninety days after the report or when a fixed
   release is available, whichever comes first. If a fix is going to take longer than that, we will
   tell you why and agree on a new date instead of letting the deadline pass in silence.

Please give us a chance to ship a fix before you publish. If you believe a vulnerability is being
exploited in the wild, say so in the first message and we will move faster.

## Supported versions

| version | supported |
| --- | --- |
| 0.1.x | yes |
| `main` | yes, fixes to the development branch are released in the next patch version |
| anything older | no |

Scabby is pre-1.0 and has no long-term support branches. If you are running a version older than the
latest release, the first step we will ask you to take is to upgrade.

## In scope

- Authentication and session handling: sign-in, session cookie issuance and invalidation, session
  fixation, and the password hashing path.
- Personal access tokens: creation, storage, verification, revocation, and the scope of what a token
  can reach.
- The Git transport and its access checks: the smart-HTTP endpoints, credentials on push, and any way
  to read from or write to a repository whose visibility or ownership should stop you.
- SSRF and path traversal in the repository and content endpoints: anything that lets a request reach
  a resource outside the served repository, or a path escape out of the repository store.
- Permission bugs that let a user read or write another user's repository, or act as another user
  through the API.

## Out of scope

- Findings that require a compromised host or a compromised database. If an attacker already reads
  `scabby.toml` or has the PostgreSQL credentials, the platform's guarantees are gone by design.
- The development credentials in `docker-compose.yml` and the example configuration files. They are
  documented throwaways for a local instance and are never correct in production.
- Missing hardening headers on pages Scabby does not serve.

## What Scabby stores

- **Passwords** are stored only as argon2id hashes. There is no plaintext password anywhere and no
  reversible encoding of one.
- **Sessions and tokens** are stored as SHA-256 digests. A personal access token's plaintext is
  returned exactly once, at creation; after that the server holds only the digest and a short prefix
  for display. A leaked database therefore does not hand over live credentials.
- **Private repositories** do not acknowledge their own existence to callers who are not allowed to
  see them. A request for a private repository returns `404` to an authenticated user who is not an
  owner, and `401` to an anonymous client.
