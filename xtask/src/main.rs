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

//! Refreshes or checks the workspace's own committed `THIRDPARTY.md`.
//!
//! Run explicitly (`cargo run -p xtask`) rather than from a `build.rs`, so
//! this crate never depends on itself to build.  `CI` set selects checking
//! over rewriting, matching the committed file against the full,
//! `--all-features` dependency graph.

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
    simulate_build_script(&root);

    let builder = list_my_licence::build::Builder::new()
        .checking(std::env::var_os("CI").is_some())
        .publish(root.join("THIRDPARTY.md"));

    if let Err(error) = builder.run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

/******************************************************************************/
