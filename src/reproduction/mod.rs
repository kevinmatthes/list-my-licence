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

//! The embedded attribution, and how to render it.
//!
//! Everything here is compiled into the shipped binary, so it carries
//! **no dependencies whatsoever** — not even for parsing.  The build
//! half writes Rust source, which the compiler then checks; there is
//! no format to get wrong at runtime and no failure mode for reading it
//! back.
//!
//! The types are deliberately plain data.  A renderer is a function
//! over them, which is what lets another output format be added later
//! without disturbing anything that already works.

mod attribution;
#[cfg(feature = "compression")]
mod compressed_attribution;
#[cfg(feature = "compression")]
mod compressed_licence;
#[cfg(feature = "compression")]
mod compressed_package;
mod embed;
#[cfg(feature = "compression")]
mod embed_compressed;
mod licence;
mod markdown;
mod origin;
mod package;

pub use crate::reproduction::{
    attribution::Attribution, licence::Licence, markdown::Markdown,
    origin::Origin, package::Package,
};

#[cfg(feature = "compression")]
pub use crate::reproduction::{
    compressed_attribution::CompressedAttribution,
    compressed_licence::CompressedLicence,
    compressed_package::CompressedPackage,
};

/******************************************************************************/
