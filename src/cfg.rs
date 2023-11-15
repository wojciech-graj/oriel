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

#[derive(Debug, Default)]
pub enum Standard {
    #[default]
    WIN3,
}

impl TryFrom<&str> for Standard {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "win3" => Ok(Self::WIN3),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Default)]
pub struct Config {
    pub pedantic: bool,
    pub standard: Standard,
}
