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

//! Tests for the feature `build`.

#![cfg(feature = "build")]

mod attribution {
    use list_my_licence::build::{Attribution, Provenance};

    fn example() -> Attribution {
        Attribution::new(
            "identifier".to_string(),
            "text".to_string(),
            Provenance::Canonical,
        )
    }

    mod identifier {
        #[test]
        fn get() {
            assert_eq!(super::example().identifier(), "identifier");
        }

        #[test]
        fn set() {
            assert_eq!(
                super::example().with_identifier("new").identifier(),
                "new"
            );
        }
    }

    mod provenance {
        use crate::attribution::example;
        use list_my_licence::build::Provenance;

        #[test]
        fn get() {
            assert_eq!(
                example().provenance(),
                &Provenance::Canonical
            );
        }

        #[test]
        fn set() {
            assert_eq!(
                &example()
                    .with_provenance(super::Provenance::Combined(
                        "LICENCE".into()
                    ))
                    .provenance(),
                &Provenance::Combined("LICENCE".into())
            );
        }
    }

    mod text {
        #[test]
        fn get() {
            assert_eq!(super::example().text(), "text");
        }

        #[test]
        fn set() {
            assert_eq!(super::example().with_text("new").text(), "new");
        }
    }
}

/******************************************************************************/
