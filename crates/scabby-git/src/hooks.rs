//! Hook scripts installed into every bare repository.
//!
//! The hooks call back into the same binary that created the repository (`scabby hook ...`), so a
//! push tells the server about itself without a queue or a watcher process. The script records the
//! path of the binary at install time: a hook that silently does nothing because `scabby` is not on
//! `PATH` is worse than one that fails loudly, and this way running from a checkout works too.

use std::io;
use std::path::Path;

use scabby_core::names::Slug;

/// Comment every generated hook starts with, so a human reading it knows not to edit it by hand.
pub const MARKER: &str = "# installed by scabby";

/// The command a hook runs when the install path was not recorded.
pub const FALLBACK_COMMAND: &str = "scabby";

/// The `post-receive` script for one repository.
///
/// git hands the updated refs to the hook on stdin; the script passes its own stdin straight through,
/// so the callback sees them. A failure inside the hook must not reject the push, which is why the
/// exit status is swallowed: at this point the objects are already in the repository and the refs
/// have already moved, so a hook that failed the push would report a lie.
pub fn post_receive(slug: &Slug, command: &Path) -> String {
    hook_script("post-receive", slug, command)
}

/// The `pre-receive` script for one repository.
///
/// Nothing is policy-gated yet, so this only reports the push. It exists so that adding a refusal
/// later does not mean every existing repository needs a new hook installed.
pub fn pre_receive(slug: &Slug, command: &Path) -> String {
    hook_script("pre-receive", slug, command)
}

/// Build one hook script.
///
/// The call itself is single quoted and the path is not: a repository is addressed as `owner/name`,
/// which is safe in a shell word, while the binary's path can contain anything, including a space
/// in someone's home directory.
fn hook_script(name: &str, slug: &Slug, command: &Path) -> String {
    format!(
        "#!/bin/sh\n\
         {MARKER} for {slug}; do not edit.\n\
         # Rewrite every repository with `scabby admin install-hooks` after moving the binary.\n\
         exec '{command}' hook {name} --repo '{slug}' >/dev/null 2>&1 || exit 0\n",
        command = command.display()
    )
}

/// Write both hooks into a bare repository's `hooks` directory.
pub fn install(repo_dir: &Path, slug: &Slug, command: &Path) -> io::Result<()> {
    let hooks_dir = repo_dir.join("hooks");
    std::fs::create_dir_all(&hooks_dir)?;

    for (name, script) in [
        ("post-receive", post_receive(slug, command)),
        ("pre-receive", pre_receive(slug, command)),
    ] {
        let path = hooks_dir.join(name);
        std::fs::write(&path, script)?;
        make_executable(&path)?;
    }
    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions)
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hooks_are_shell_scripts_that_fall_through() {
        let slug = Slug::parse("ada/scabby").unwrap();
        let script = post_receive(&slug, Path::new("/usr/local/bin/scabby"));

        assert!(script.starts_with("#!/bin/sh\n"));
        assert!(script.contains(MARKER));
        assert!(script.contains("--repo 'ada/scabby'"));
        assert!(script.contains("'/usr/local/bin/scabby' hook post-receive"));
        assert!(
            script.ends_with("|| exit 0\n"),
            "a hook must never reject a completed push"
        );
    }
}
