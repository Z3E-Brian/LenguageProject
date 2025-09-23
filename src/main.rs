mod parser;
mod lexer;
mod utils;
pub mod semantics;
mod gui;

use lexer::Lexer;
use parser::Parser;
use crate::semantics::{SemCtx, SemanticPass};

/// Realiza el análisis léxico del código fuente y devuelve los tokens.
pub fn run_lexer(code: &str) -> Result<Vec<crate::utils::token::Token>, String> {
    let mut lexer = Lexer::new(code);
    lexer.lex_all()
}

/// Realiza el análisis sintáctico de los tokens y devuelve el AST.
pub fn run_parser(tokens: Vec<crate::utils::token::Token>) -> Result<parser::Program, parser::ParseError> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

/// Realiza el análisis semántico del AST.
pub fn run_semantics(program: &parser::Program) -> Result<(), Vec<semantics::SemError>> {
    let mut ctx = SemCtx::new();
    {
        let mut pass = SemanticPass::new(&mut ctx);
        pass.check_program(program);
    }

    if ctx.errors.is_empty() {
        Ok(())
    } else {
        Err(ctx.errors)
    }
}

fn main() {
    gui::run();
}
