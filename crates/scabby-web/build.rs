//! Makes sure the UI directory exists before rust-embed reads it.
//!
//! The Svelte app is built into `web/build` by `npm run build` in `web/`, and the binary embeds
//! whatever is there. On a clean checkout that directory does not exist yet, and rust-embed fails at
//! compile time rather than at first request, so the placeholder below is written instead: a page
//! that says the UI has not been built and how to build it. A real build overwrites it.

use std::path::PathBuf;

const PLACEHOLDER: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Scabby</title>
    <style>
      :root { color-scheme: dark; }
      body {
        margin: 0; min-height: 100vh; display: grid; place-content: center; gap: 12px;
        background: #04030a; color: #ede9f7; text-align: center;
        font: 15px/1.6 system-ui, sans-serif;
      }
      code { background: #1a1029; color: #c7b0ff; padding: 2px 6px; border-radius: 6px;
             font-family: "JetBrains Mono", monospace; }
      p { color: #9a8fb5; margin: 0; }
    </style>
  </head>
  <body>
    <h1>Scabby is running</h1>
    <p>The user interface has not been built into this binary.</p>
    <p>Run <code>npm install &amp;&amp; npm run build</code> in <code>web/</code>, then rebuild, or start the server with <code>--frontend-dir web/build</code>.</p>
    <p>The API is available at <code>/api/v1</code>.</p>
  </body>
</html>
"#;

fn main() {
    let build_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"))
        .join("../../web/build");
    let index = build_dir.join("index.html");

    // Rebuilding the UI is a reason to re-embed it, so the binary and the bundle never disagree.
    println!("cargo:rerun-if-changed={}", build_dir.display());

    if index.is_file() {
        return;
    }

    if let Err(error) = std::fs::create_dir_all(&build_dir) {
        panic!("could not create {}: {error}", build_dir.display());
    }
    if let Err(error) = std::fs::write(&index, PLACEHOLDER) {
        panic!("could not write {}: {error}", index.display());
    }
}
