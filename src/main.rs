mod parser;
mod lexer;
use lexer::Lexer;
mod utils;

use parser::{Parser, print_ast};

fn main() {
    let code = r#"
        !! comentario de linea
        !!atom numero : atom_num = 13;

        reaction name(numero1: atom_num) {
            molecule agua : formula = "H2O";
            molecule sal  : formula = "NaCl";
        }
        ion PI : mass = 34;

        itest ( numero > 10 and not (PI == 3) ) {
            emitln "Hola\n";
        } notest {
            emit "Adios";
        }
    "#;

    // 1) Lex
    let mut lexer = Lexer::new(code);
    let tokens = match lexer.lex_all() {
        Ok(toks) => toks,
        Err(e) => {
            eprintln!("Error léxico: {e}");
            std::process::exit(1);
        }
    };

    // 2) Parse
    let mut p = Parser::new(tokens);
    match p.parse_program() {
        Ok(ast) => {
            // 3) Usar el AST (así desaparece el warning de Stmt sin usar)
            print_ast(&ast);
        }
        Err(e) => {
            eprintln!("Error de parseo: {} @({},{})", e.message, e.line, e.col);
            std::process::exit(1);
        }
    }
}

