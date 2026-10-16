# Contributing to Scabby

Thanks for wanting to work on Scabby. This document covers what you need installed, the checks a
change has to pass, how commits and branches are expected to look, and where the two contracts that
constrain most changes live.

## Getting set up

Install:

- Rust 1.90 or later with `rustfmt` and `clippy`. The workspace is edition 2024 and pins
  `rust-version = "1.90"`.
- Node 22 or later with npm.
- PostgreSQL 18. The repository ships a development instance:

```sh
docker compose up -d
```

That container is user `scabby`, password `scabby`, database `scabby`, on `localhost:5432`, and its
data lives in a named volume so it survives recreation. It also creates the `scabby` database the
runtime defaults point at; the test database is created by the test setup or by you (see below).

Then build:

```sh
cd web && npm install && npm run build && cd ..
cargo build
```

The frontend build is not strictly required to compile the backend — the build script writes a
placeholder page when `web/build` is missing — but a server without a built UI serves that
placeholder instead of the app.

## The local check list

Run all of this before you open a pull request:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

and in `web/`:

```sh
npm run check
npm run build
```

`npm run check` runs `svelte-check` against `tsconfig.json`, so it type-checks the app the way the
compiler sees it. `npm run build` is the same command the release pipeline uses.

CI runs the same steps plus a smoke check that the release binary can parse its arguments
(`./target/release/scabby --version` and `--help`). If a check fails locally it will fail there.

## Tests and the database

sqlx queries in this repository are runtime-checked, not compile-time-checked against a live
connection, so `cargo build` needs no database. The tests do.

Tests that touch PostgreSQL read their connection string from `SCABBY_TEST_DATABASE_URL`. Point it at
a database you are willing to have dropped and recreated:

```sh
export SCABBY_TEST_DATABASE_URL=postgres://scabby:scabby@localhost:5432/scabby_test
cargo test
```

The development `docker compose` instance works for this. Use a separate database name from the
runtime one (`scabby_test`, not `scabby`) so a test run cannot destroy the data you are working with.

## Commit messages

Conventional commits, with the crate as the scope where one crate is the subject:

```text
feat(core): add repository visibility to the domain model
fix(web): keep the repo header from shrinking on long names
docs: describe the token endpoints
```

Scopes in use are `core`, `db`, `git`, `web`, and `cli`. Use `docs` with no scope for documentation
changes, and `chore` for dependency bumps and tooling. Write the subject in the imperative mood and
keep it under about 72 characters; put the reasoning in the body when it is not obvious from the
diff.

Two rules matter more than the format:

1. **Every commit builds on its own.** `cargo build` and `cargo test` have to pass at each commit,
   not only at the tip of the branch. That is what makes `git bisect` usable.
2. **Prefer several small commits over one large commit.** Separate the mechanical change from the
   behavioral change, and separate a dependency bump from the code that uses it.

## Branches and pull requests

Branch off `main` and name the branch after what it does (`repo-settings`, `token-scopes`). Open a
pull request against `main` when the checks pass locally.

A pull request should say what changed, why, and what you did to verify it. If the change is
user-visible, say what you expect the reviewer to see. Keep the branch rebased on `main` rather than
merging `main` into it, and expect review comments on both the code and the description.

Because the checks are the same locally and in CI, a green local run plus a description of what you
verified is usually enough to get a review started.

## Migrations

Migrations live in `crates/scabby-db/migrations` and are embedded in the binary, so they run from the
artifact, not from a file on the host.

To add one:

1. Create the next number in sequence: if the highest is `0004_...`, add `0005_<short_name>.sql`.
   Numbers are four digits, zero-padded, and names are lowercase with underscores.
2. Write forward-only SQL. There are no down migrations.
3. **Never edit a migration that has landed.** Someone has already run it. A correction is a new
   migration.

## The two contracts

Two files constrain more of this repository than any others, and both are updated in the same commit
as the change that invalidated them:

- **`DESIGN.md`** is the design contract for everything under `web/`. It defines the palette
  (every color is a token in `web/src/app.css`), the type scale, spacing, radii, motion, the three
  page shells, the component inventory, and the performance and accessibility budgets. There is no
  Tailwind and no new color outside those tokens — a component that needs a new value needs a token
  added to `DESIGN.md` and `app.css` in the same change.
- **`docs/api.md`** is the REST API contract. The web UI talks to `/api/v1` for everything it shows,
  so the documented request and response shapes are what the frontend is written against. If you
  change an endpoint, a field, or a status code, update `docs/api.md` in the same commit.

If a change makes `DESIGN.md` or `docs/api.md` wrong and the file is not updated, the change is
incomplete.

## Secrets

`scabby.toml` and `.env` are gitignored because both hold `database_url`. Do not commit a real
connection string, do not paste one into an issue, and do not echo credentials in test output or
logs. The values in `docker-compose.yml`, `.env.example`, and `scabby.toml.example` are development
throwaways and are safe to read.

## Code of conduct

Participation in this project is covered by [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). Report security
problems through the private channel in [SECURITY.md](SECURITY.md), not in a public issue.
