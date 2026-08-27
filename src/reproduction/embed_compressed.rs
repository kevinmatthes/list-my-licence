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

/// Pulls in the compressed attribution the build script wrote.
///
/// The counterpart of [`embed!`](crate::embed).  Requires the build half to
/// have called `Emitter::embed_compressed`, which lives behind the `build`
/// feature — deliberately not linked, since that feature need not be enabled
/// wherever this macro is read.
///
/// # Examples
///
/// ```text
/// static LICENCES: list_my_licence::CompressedAttribution =
///     list_my_licence::embed_compressed!();
///
/// fn main() {
///     print!("{LICENCES}");
/// }
/// ```
#[macro_export]
macro_rules! embed_compressed {
    () => {{
        use $crate::{
            CompressedAttribution, CompressedLicence, CompressedPackage, Origin,
        };

        include!(concat!(env!("OUT_DIR"), "/list-my-licence-compressed.rs"))
    }};
}

/******************************************************************************/
