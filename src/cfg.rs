// Copyright (C) 2023  Wojciech Graj
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

use std::fmt;

#[derive(Debug, Default)]
pub enum Standard {
    #[default]
    WIN3_0,
    WIN3_1,
}

impl TryFrom<&str> for Standard {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "win3.0" => Ok(Self::WIN3_0),
            "win3.1" => Ok(Self::WIN3_1),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Standard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Standard::WIN3_0 => "3.0",
                Standard::WIN3_1 => "3.1",
            }
        )
    }
}

#[derive(Debug, Default)]
pub struct Config {
    pub pedantic: bool,
    pub standard: Standard,
}
