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

use std::{env, fs::read_to_string};

mod cfg;
mod ir;
mod parse;
mod sys_gtk;
mod vm;

fn main() {
    let args: Vec<String> = env::args().collect();

    let opts = {
        let mut opts = getopts::Options::new();
        opts.optflag("", "pedantic", "");
        opts.optflagopt("", "std", "", "");
        opts
    };

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => panic!("{}", e),
    };

    let src = {
        let mut src = if !matches.free.is_empty() {
            match read_to_string(matches.free[0].clone()) {
                Ok(src) => src,
                Err(e) => panic!("{}", e),
            }
        } else {
            println!("Provide a source file.");
            return;
        };
        src.push('\n');
        src
    };

    let config = cfg::Config {
        pedantic: matches.opt_present("pedantic"),
        standard: if let Some(standard) = matches.opt_str("std") {
            match standard.as_str().try_into() {
                Ok(standard) => standard,
                Err(_) => panic!("Unrecognized standard '{}'", standard),
            }
        } else {
            cfg::Standard::default()
        },
    };

    let prog = match ir::Program::from_src(&src, &config) {
        Ok(prog) => prog,
        Err(e) => panic!("{}", e),
    };

    let mut sys = match sys_gtk::VMSysGtk::new(&args[1]) {
        Ok(sys) => sys,
        Err(e) => panic!("{}", e),
    };

    let mut vm = vm::VM::new(&prog, &config, &mut sys);
    let res = vm.run();
    if let Err(e) = res {
        panic!("{}", e);
    }
}
