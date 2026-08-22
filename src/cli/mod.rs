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

//! Command line plumbing for applications built with clap.
//!
//! Deliberately additive.  These types contribute options and
//! subcommands to whatever parser an application already has, rather
//! than replacing the call that parses them:  a wrapper around
//! `parse` does not compose with a derived `Parser`, and an
//! application should not have to give up its own argument handling
//! to gain a `--licences` option.

mod licence_args;
mod licence_command;

pub use crate::cli::{
    licence_args::LicenceArgs, licence_command::LicenceCommand,
};

/******************************************************************************/
