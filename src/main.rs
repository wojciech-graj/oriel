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

    let prog = match parse::parse(&src) {
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
