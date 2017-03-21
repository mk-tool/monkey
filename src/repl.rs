use std::io::{self, Write};

use lexer;
use parser;
use ast::Node;
use evaluator::eval;

pub fn run() {
    let prompt = ">>";
    let mut scan = String::new();

    print!("read print eval loop is started {}", prompt);
    io::stdout().flush().unwrap();

    loop {
        io::stdin()
            .read_line(&mut scan)
            .expect("Failed to read line");

        let lex = lexer::new(scan.clone());
        let mut p = parser::new(lex);
        let program = p.parse_program();

        if p.errors.len() > 0 {
            for error in p.errors.into_iter() {
                println!("{}", error);
            }
            continue;
        }

        let evaluated = eval(program.to_enum());
        println!("{:?}", evaluated.inspect());
        scan = "".to_string();
        print!("{}", prompt);
        io::stdout().flush().unwrap();
    }
}

