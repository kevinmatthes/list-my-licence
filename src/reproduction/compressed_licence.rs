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

/// One licence of one package, held compressed.
///
/// The counterpart of [`Licence`](crate::Licence) for
/// [`embed_compressed!`](crate::embed_compressed).  The text is stored as
/// DEFLATE bytes and inflated on demand, so a binary carries its licences at
/// a fraction of their size and pays to expand them only when it shows them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompressedLicence {
    /// The SPDX identifier, or the reference invented for a custom licence.
    pub identifier: &'static str,

    /// The text, compressed exactly as it was read.
    pub bytes: &'static [u8],

    /// Where that text came from, so the reader can judge it.
    pub origin: crate::Origin,
}

impl CompressedLicence {
    /// The text, inflated and reproduced exactly.
    ///
    /// # Panics
    ///
    /// Panics if the bytes do not inflate.  They are written by this crate's
    /// own build half and never touched afterwards, so a failure here means
    /// the embedded artefact is corrupt.  That is not a condition a caller
    /// can sensibly handle, and returning a partial licence text would be
    /// worse than stopping:  reproducing a licence *almost* verbatim is the
    /// one thing this crate exists to avoid.
    #[must_use]
    pub fn text(&self) -> String {
        String::from_utf8(
            miniz_oxide::inflate::decompress_to_vec(self.bytes)
                .expect("embedded licence text does not inflate"),
        )
        .expect("embedded licence text is not UTF-8")
    }
}

/******************************************************************************/
