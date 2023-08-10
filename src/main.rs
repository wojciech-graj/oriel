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

mod ir;
mod parse;
mod sys_gtk;
mod vm;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Must provide exactly one argument.");
        return;
    }

    let src = {
        let mut src = read_to_string(&args[1]).unwrap();
        src.push('\n');
        src
    };

    let prog = match ir::Program::from_src(&src) {
        Ok(prog) => prog,
        Err(e) => panic!("{}", e),
    };

    let mut sys = match sys_gtk::VMSysGtk::new(&args[1]) {
        Ok(sys) => sys,
        Err(e) => panic!("{}", e),
    };

    let mut vm = vm::VM::new(&prog, &mut sys);
    let res = vm.run();
    if let Err(e) = res {
        panic!("{}", e);
    }
}
