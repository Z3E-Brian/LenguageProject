// src/main.rs

mod parser;
mod lexer;
mod utils;

use lexer::Lexer;
use parser::{Parser, print_ast};
use std::env;
use std::fs;

/// Imprime un listado compacto de tokens (kind, lexeme)
fn print_tokens(tokens: &[crate::utils::token::Token]) {
    println!("=== TOKENS ({} items) ===", tokens.len());
    for t in tokens {
        println!("{:?} '{}'\t@({},{})", t.kind, t.lexeme, t.line, t.col);
    }
}

fn main() {
    // ------------------------------------------------------------
    // CLI:
    //   cargo run -- [opciones] [archivo]
    //
    // Opciones:
    //   --no-tokens  → no imprime los tokens
    //   --no-ast     → no imprime el AST
    //
    // Si no pasás archivo, usa un snippet de ejemplo.
    // ------------------------------------------------------------
    let mut show_tokens = true;
    let mut show_ast = true;
    let mut file_arg: Option<String> = None;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--no-tokens" => show_tokens = false,
            "--no-ast"    => show_ast = false,
            s => file_arg = Some(s.to_string()),
        }
    }

    // Código fuente
    let code = if let Some(path) = file_arg {
        match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("No pude leer '{}': {}", path, e);
                std::process::exit(1);
            }
        }
    } else {
        // Snippet de ejemplo (puedes cambiarlo libremente)
        r#"
            !! comentario de linea
            atom numero : atom_num = 13;

            reaction name(numero1: atom_num, mass numero2) {
                atom agua : formula = "H2O";
                atom sal  : formula = "NaCl";
                emitln "Dentro de reaction";
            }

            ion PI : mass = 3.1416;

            itest ( numero > 10 and not (PI == 3) ) {
                emitln "Hola\n";
            } inotest (numero == 10) {
                emitln "Es diez";
            } notest {
                emit "Adios";
            }

            !! ExprStmt de ejemplo (llamada futura):
            !! foo();
        "#.to_string()
    };

    // 1) LEX
    let mut lexer = Lexer::new(&code);
    let tokens = match lexer.lex_all() {
        Ok(toks) => toks,
        Err(e) => {
            eprintln!("Error léxico: {}", e);
            // Si el error es un String, no tiene posición; solo muestra el mensaje.
            // Si tu tipo de error tiene posición, ajusta aquí.
            std::process::exit(1);
        }
    };

    if show_tokens {
        print_tokens(&tokens);
    }

    // 2) PARSE
    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Error de parseo: {}", e.message);
            eprintln!("@({},{})", e.line, e.col);
            show_source_arrow(&code, e.line, e.col);
            std::process::exit(1);
        }
    };

    if show_ast {
        println!("\n=== AST ===");
        print_ast(&program);
    }

    // 3) (Opcional) Semántica / Ejecución
    // Aquí podrías invocar tu verificador semántico o un intérprete/traductor.
}

/// Muestra la línea del error y una flecha apuntando a la columna.
fn show_source_arrow(src: &str, err_line: usize, err_col: usize) {
    // Las líneas/columnas del lexer suelen ser 1-based.
    let line_idx = err_line.saturating_sub(1);
    if let Some(line_str) = src.lines().nth(line_idx) {
        eprintln!("\n>> {}", line_str);
        if err_col > 0 {
            let mut arrow = String::new();
            // Ojo con tabs; aquí contamos espacios simples.
            for _ in 0..(err_col.saturating_sub(1)) { arrow.push(' '); }
            arrow.push('^');
            eprintln!("   {}", arrow);
        }
    }
}
