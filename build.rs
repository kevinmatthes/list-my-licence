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

//! Harvest the dependency licences at build time.
//!
//! Dogfoods this crate's own `build` feature on itself, embedding the
//! notices for a consumer of the `build` feature to print and refreshing
//! the committed `THIRDPARTY.md`.  Under continuous integration (`CI` set)
//! it checks that file against the graph instead of rewriting it, so
//! licence drift cannot be merged unnoticed.
//!
//! `Builder::run` resolves against whichever features Cargo happens to
//! have enabled for *this* build (`Resolver::from_build_env`), which
//! genuinely varies across this crate's own `cargo-features` CI matrix —
//! a `--features compression` build needs a different graph than
//! `--all-features`.  A single committed file cannot match all of them,
//! so `THIRDPARTY.md` is only written or checked under `--all-features`,
//! the one combination that is the true superset and so the only one
//! stable across every build.

fn main() {
    let checking = std::env::var_os("CI").is_some();
    let all_features =
        ["BUILD", "CLAP", "COMPRESSION"].into_iter().all(|feature| {
            std::env::var_os(format!("CARGO_FEATURE_{feature}")).is_some()
        });

    let mut builder = list_my_licence::build::Builder::new().checking(checking);

    if all_features {
        builder = builder.publish("THIRDPARTY.md");
    }

    if let Err(error) = builder.run() {
        panic!("{error}");
    }
}

/******************************************************************************/
