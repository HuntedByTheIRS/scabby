# Guidance for AI coding agents

This file is for automated contributors working in this repository. It states the constraints that
are not visible from a single file, in the order they tend to bite.

## The crate split, and the rule that comes with it

One cargo workspace, five crates, each with a job:

| crate | job |
| --- | --- |
| `scabby-core` | Domain types, identifiers, errors, configuration, secret hashing. **No IO, and no dependency that does IO.** |
| `scabby-db` | PostgreSQL through sqlx, the connection pool, embedded migrations, one module per resource. |
| `scabby-git` | The bare repository store, commit and tree reads through gix, diffs, hooks, and the smart-HTTP transport. |
| `scabby-web` | The axum router, session auth, the `/api/v1` REST API, the git HTTP endpoints, and the embedded UI. |
| `scabby` | The binary: argument parsing, configuration resolution, wiring `serve` together. |

`scabby-core` is the only crate every other crate depends on, so it stays free of IO: no `tokio`, no
sqlx, no filesystem, no network, at any depth. If a change in `scabby-core` needs to read a file or
open a socket, the logic belongs in the crate that does that IO and the type it needs belongs in
`scabby-core`. Its dependency list is `serde`, `thiserror`, `time`, `toml`, and `url` — treat an
addition to that list as a design question, not a detail.

Dependencies are declared once in the root `Cargo.toml` under `[workspace.dependencies]`; crates opt
in with `dep = { workspace = true }`. Do not pin a version inside a crate that the workspace already
declares.

## The workspace lints are not suggestions

`Cargo.toml` sets `unsafe_code = "forbid"`, `missing_docs = "warn"`, clippy `all` at warn, and
`dbg_macro` and `todo` at warn. CI runs clippy with `-D warnings`, so a warning is a failed build.
That includes the `todo` lint: do not leave a `todo!()` behind in code you commit. Public items carry
doc comments. `rustfmt.toml` is checked in; format with `cargo fmt --all`, do not hand-align.

## Two contracts, and what "source of truth" means here

- **`docs/api.md` is the source of truth for anything the UI shows.** Every screen in `web/` talks to
  `/api/v1`; there is no server-side data loading and no second data path. When you need to know what
  a field is called, whether a value can be null, or which status code a failure returns, read
  `docs/api.md` rather than inferring it from a component. When you change an endpoint, a field, or a
  status code, `docs/api.md` changes in the same commit.
- **The design tokens live in `DESIGN.md` and `web/src/app.css`.** Styling is hand-written CSS custom
  properties in `web/src/app.css`. There is no Tailwind and no CSS-in-JS. No component hard-codes a
  hex value, and no new color is invented outside the tokens in those two files — if a surface needs
  a value that does not exist, add the token to both `DESIGN.md` and `app.css` in the same change.
  The same goes for the type scale, spacing, radii, and motion timings: they are listed in
  `DESIGN.md`, and a component that needs a new step is a change to the contract, not a local
  decision. Follow the component inventory and the page/state matrix in `DESIGN.md`; every state that
  is meaningful (hover, focus-visible, disabled, loading, empty, error) belongs in the component, not
  the page.

The design and API documents are contracts, so they are updated in the same commit as the code that
moved them. If you find one of them wrong, fix it — do not work around it.

## Verification commands

Run these before you claim a change is done, and report what actually happened when you ran them:

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

Two things about the environment these run in:

- **sqlx queries are runtime-checked**, not checked against a live database at compile time, so
  `cargo build` needs no database. Tests do: they read `SCABBY_TEST_DATABASE_URL`, so a database has
  to be running and that variable has to point at a throwaway database before `cargo test` means
  anything. Do not report the test suite as passing if it skipped for want of a database.
- **`scabby-git` shells out to `git`** for the transport (`git upload-pack --stateless-rpc` and
  `git receive-pack --stateless-rpc`), so tests that exercise clone or push need a `git` binary on
  `PATH`. Object reads go through gix and do not need one.

`cargo build --release` embeds `web/build` with `rust-embed`, so **the frontend has to be built
first** (`cd web && npm install && npm run build`) for the release artifact to contain the UI. A
missing `web/build` does not fail the build: the build script writes a placeholder page and the
binary serves it. That is a working build with a UI that says "the user interface has not been built
into this binary", not a correct release.

## Secrets

- Never commit `scabby.toml` or `.env`. Both are gitignored because both hold a real `database_url`.
  `scabby.toml.example` and `.env.example` are the committable templates.
- Never echo a credential — not in logs, not in a test fixture, not in a command you print, not in a
  suggested fix. When you need to show a connection string, use the development throwaway from
  `docker-compose.yml`.
- The values in `docker-compose.yml`, `.env.example`, and `scabby.toml.example` are development
  throwaways (`scabby`/`scabby`) and are the only ones safe to write down.

## Scratch state

Your own working files — capture scripts, notes, verification transcripts, one-off probes — belong in
the git-ignored `.omh/` directory at the repository root, not in the tree. The `.gitignore` entry for
`/data/`, `/scabby.toml`, `/.env`, and `/.omh/` is deliberate: nothing you generate while working
should show up in `git status` as an untracked file waiting to be committed by mistake. Use the
session scratch directory for anything temporary that does not need to survive.

## Before you say a change is done

- The verification commands above ran, and you are reporting their real output, not an expectation.
- Every commit builds on its own, and the commits are small enough to explain one at a time.
- A user-visible change is reflected in `docs/api.md` (if the API moved) or `DESIGN.md` (if the design
  contract moved), in the same commit.
- No placeholder text, no `todo!()`, no commented-out code, and no new color outside the tokens.
- If something could not be verified in this environment — no database, no `git` binary, no built
  frontend — say so plainly rather than describing the result you expect.
