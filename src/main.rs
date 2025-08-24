mod parser;
mod lexer;
use lexer::Lexer;

fn main() {
    let code = r#"
        !! comentario de linea
        atom numero : atom_num = 13;
        Reaction name(atom numero1){
            Molecule agua : formula = "H2O";
            Molecule sal : formula = "NaCl";
        }
        ion PI : mass = 34;
        itest ( numero > 10 and not (PI == 3) ) emitln "Hola\n";
        { [ ] } , :
    "#;

    let mut lexer = Lexer::new(code);
    match lexer.lex_all() {
        Ok(toks) => {
            for t in toks {
                println!("{:<12} lex='{}'  @({},{})", t.get_kind(), t.get_lexeme(), t.get_line(), t.get_col());
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
