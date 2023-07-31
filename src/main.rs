use std::{env, fs::read_to_string};

mod parse;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Must provide exactly one argument.");
        return;
    }

    let src = read_to_string(&args[1]).unwrap();

    let prog = match parse::parse(&src) {
        Ok(prog) => prog,
        Err(e) => panic!("{}", e),
    };

    println!("{:?}", prog.commands);
}
