mod parser;
mod lexer;
mod utils;
pub mod semantics;
mod gui;
mod codegen;
mod executor;

use lexer::Lexer;
use parser::Parser;
use crate::semantics::{SemCtx, SemanticPass};
use crate::codegen::CodeGenerator;
use crate::executor::Executor;
use crate::utils::enums::{self};

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

/// Genera código intermedio del AST verificado.
pub fn run_codegen(program: &parser::Program) -> Result<Vec<enums::Instruction>, String> {
    let mut generator = CodeGenerator::new();
    generator.generate(program)
}

/// Ejecuta el código intermedio y devuelve la salida.
pub fn run_executor(instructions: Vec<enums::Instruction>) -> Result<String, executor::ExecutionError> {
    let mut executor = Executor::new();
    executor.execute(&instructions)
}

/// Pipeline completo: compilar y ejecutar código ElementScript.
pub fn compile_and_execute(code: &str) -> Result<String, String> {
    // 1. Análisis léxico
    let tokens = run_lexer(code)
        .map_err(|e| format!("Error léxico: {}", e))?;

    // 2. Análisis sintáctico
    let ast = run_parser(tokens)
        .map_err(|e| format!("Error de parseo: {} @ {}:{}", e.message, e.line, e.col))?;

    // 3. Análisis semántico
    run_semantics(&ast)
        .map_err(|errors| {
            let mut err_msg = String::from("Errores semánticos:\n");
            for e in errors {
                err_msg.push_str(&format!("- {} @ {}:{}\n", e.msg, e.line, e.col));
            }
            err_msg
        })?;

    // 4. Generación de código intermedio
    let instructions = run_codegen(&ast)
        .map_err(|e| format!("Error de generación de código: {}", e))?;

    // 5. Ejecución
    let result = run_executor(instructions)
        .map_err(|e| format!("Error de ejecución: {}", e))?;

    Ok(result)
}

fn main() {
    gui::run();
}
