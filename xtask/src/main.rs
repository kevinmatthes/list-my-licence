/*********************** GNU General Public License 3.0 ***********************\
|                                                                              |
|  Copyright (C) 2026 Kevin Matthes                                            |
|                                                                              |
|  This program is free software: you can redistribute it and/or modify        |
|  it under the terms of the GNU General Public License as published by        |
|  the Free Software Foundation, either version 3 of the License, or           |
|  (at your option) any later version.                                         |
|                                                                              |
|  This program is distributed in the hope that it will be useful,             |
|  but WITHOUT ANY WARRANTY; without even the implied warranty of              |
|  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the               |
|  GNU General Public License for more details.                                |
|                                                                              |
|  You should have received a copy of the GNU General Public License           |
|  along with this program.  If not, see <https://www.gnu.org/licenses/>.      |
|                                                                              |
\******************************************************************************/

//! Refreshes or checks the workspace's own committed `THIRDPARTY.md` and
//! `debian/copyright`.
//!
//! Run explicitly (`cargo run -p xtask`) rather than from a `build.rs`, so
//! this crate never depends on itself to build.  `CI` set selects checking
//! over rewriting, matching the committed files against the full,
//! `--all-features` dependency graph.

/// Refreshes or checks the committed `debian/copyright`.
///
/// The DEP-5 counterpart of what [`list_my_licence::build::Builder::run`]
/// already does for `THIRDPARTY.md`, hand-rolled because
/// [`list_my_licence::build::Emitter::check`] is markdown-specific — there
/// is no DEP-5 equivalent to call instead.
fn refresh_or_check_copyright(
    root: &std::path::Path,
    outcome: &list_my_licence::build::Outcome,
    checking: bool,
) -> std::io::Result<()> {
    let packages: Vec<list_my_licence::build::Reproduced<'_>> = outcome
        .packages
        .iter()
        .map(|(package, verdict)| (package, verdict))
        .collect();
    let expected = list_my_licence::build::Emitter::dep5(&packages);
    let path = root.join("debian/copyright");

    if !checking {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        return std::fs::write(&path, expected);
    }

    match std::fs::read_to_string(&path) {
        Ok(found) if found == expected => Ok(()),
        _ => Err(std::io::Error::other(format!(
            "{} is missing or out of date",
            path.display()
        ))),
    }
}

fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn simulate_build_script(root: &std::path::Path) {
    let out_dir = std::env::temp_dir().join("list-my-licence-xtask");
    std::fs::create_dir_all(&out_dir).expect("a writable temporary directory");

    for (key, value) in [
        ("CARGO_MANIFEST_DIR", root.as_os_str()),
        ("OUT_DIR", out_dir.as_os_str()),
        ("CARGO_FEATURE_BUILD", std::ffi::OsStr::new("1")),
        ("CARGO_FEATURE_CLAP", std::ffi::OsStr::new("1")),
        ("CARGO_FEATURE_COMPRESSION", std::ffi::OsStr::new("1")),
    ] {
        unsafe { std::env::set_var(key, value) };
    }
}

fn main() {
    let root = root();
    let checking = std::env::var_os("CI").is_some();

    simulate_build_script(&root);

    let builder = list_my_licence::build::Builder::new()
        .checking(checking)
        .publish(root.join("THIRDPARTY.md"));

    let outcome = match builder.run() {
        Ok(outcome) => outcome,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };

    if let Err(error) = refresh_or_check_copyright(&root, &outcome, checking) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

/******************************************************************************/
