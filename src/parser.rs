// parser.rs
// Combina lo mejor de ambas versiones: cobertura completa + ExprStmt + ; tolerantes + TypeName::Custom + errores tipados

use crate::utils::enums::{Expr, Stmt, TokenKind, TypeName};
use crate::utils::token::Token;
use std::fmt;

// ===================== Tipos auxiliares =====================

pub type Program = Vec<Stmt>;
pub type Block = Vec<Stmt>;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::LitNumber(s) => write!(f, "{s}"),
            Expr::LitString(s) => write!(f, "\"{s}\""),
            Expr::Ident(id) => write!(f, "{id}"),
            Expr::Unary { op, rhs } => write!(f, "({:?} {})", op, rhs),
            Expr::Binary { lhs, op, rhs } => write!(f, "({} {:?} {})", lhs, op, rhs),
            _ => write!(f, "{:?}", self),
        }
    }
}

// ===================== Parser =====================

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    // ---------- util ----------
    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .unwrap_or(self.tokens.last().unwrap())
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::EndOfFile
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        self.previous()
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.pos - 1]
    }

    fn check(&self, kind: &TokenKind) -> bool {
        !self.is_at_end() && &self.peek().kind == kind
    }

    fn match_next(&mut self, kinds: &[TokenKind]) -> bool {
        for k in kinds {
            if self.check(k) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, expected: TokenKind, msg: &str) -> Result<Token, ParseError> {
        if self.check(&expected) {
            Ok(self.advance().clone())
        } else {
            let t = self.peek();
            Err(ParseError {
                message: format!("{msg}. Encontré {:?}", t.kind),
                line: t.line,
                col: t.col,
            })
        }
    }

    // Versión no-ruidosa para backtracking en asignación
    fn try_parse_assign_stmt(&mut self) -> Result<Stmt, ()> {
        let save = self.pos;
        if !self.check(&TokenKind::Ident) {
            return Err(());
        }
        let name = self.advance().lexeme.clone();
        if !self.check(&TokenKind::Assign) {
            self.pos = save;
            return Err(());
        }
        self.advance();
        match self.parse_expr(0) {
            Ok(value) => {
                if self.match_next(&[TokenKind::Semi]) {
                    Ok(Stmt::Assign { name, value })
                } else {
                    self.pos = save;
                    Err(())
                }
            }
            Err(_) => {
                self.pos = save;
                Err(())
            }
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            if self.match_next(&[TokenKind::Semi]) {
                continue;
            }
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    // ---------- sentencias ----------
    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        use TokenKind::*;
        match self.peek().kind {
            KwAtom | KwMolecule => self.parse_var_decl(),
            KwIon => self.parse_const_decl(),
            KwITest => self.parse_if_stmt(),
            KwEmitln => self.parse_emit_stmt(true),
            KwEmit => self.parse_emit_stmt(false),
            KwReaction => self.parse_reaction_decl(),
            LBrace => self.parse_block().map(Stmt::Block),
            Ident => {
                if let Ok(stmt) = self.try_parse_assign_stmt() {
                    return Ok(stmt);
                }
                let e = self.parse_expr(0)?;
                self.consume(Semi, "Se esperaba ';' tras expresión")?;
                Ok(Stmt::ExprStmt(e))
            }
            Number | StringLit | LParen | Plus | Minus | Not => {
                let e = self.parse_expr(0)?;
                self.consume(Semi, "Se esperaba ';' tras expresión")?;
                Ok(Stmt::ExprStmt(e))
            }
            _ => {
                let t = self.peek().clone();
                Err(ParseError {
                    message: format!("Token inesperado en sentencia: {:?}", t.kind),
                    line: t.line,
                    col: t.col,
                })
            }
        }
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        self.consume(TokenKind::LBrace, "Se esperaba '{' para iniciar bloque")?;
        let mut items = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            if self.match_next(&[TokenKind::Semi]) {
                continue;
            }
            items.push(self.parse_stmt()?);
        }
        self.consume(TokenKind::RBrace, "Se esperaba '}' para cerrar bloque")?;
        Ok(items)
    }

    fn parse_var_decl(&mut self) -> Result<Stmt, ParseError> {
        // KwAtom | KwMolecule <Ident> : <Type> ( '=' expr )? ';'
        self.advance(); // consume KwAtom/KwMolecule
        let name = self
            .consume(TokenKind::Ident, "Se esperaba nombre de variable")?
            .lexeme;
        self.consume(TokenKind::Colon, "Se esperaba ':'")?;
        let ty = self.parse_type()?;
        let init = if self.match_next(&[TokenKind::Assign]) {
            Some(self.parse_expr(0)?)
        } else {
            None
        };
        self.consume(TokenKind::Semi, "Se esperaba ';'")?;
        Ok(Stmt::VarDecl { name, ty, init })
    }

    fn parse_const_decl(&mut self) -> Result<Stmt, ParseError> {
        // KwIon <Ident> : <Type> = expr ';'
        self.consume(TokenKind::KwIon, "Falta 'ion'")?;
        let name = self
            .consume(TokenKind::Ident, "Se esperaba nombre de constante")?
            .lexeme;
        self.consume(TokenKind::Colon, "Se esperaba ':'")?;
        let ty = self.parse_type()?;
        self.consume(TokenKind::Assign, "Se esperaba '='")?;
        let value = self.parse_expr(0)?;
        self.consume(TokenKind::Semi, "Se esperaba ';'")?;
        Ok(Stmt::ConstDecl { name, ty, value })
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, ParseError> {
        use TokenKind::*;
        self.consume(KwITest, "Falta 'itest'")?;
        self.consume(LParen, "Se esperaba '('")?;
        let cond = self.parse_expr(0)?;
        self.consume(RParen, "Se esperaba ')'")?;

        // Permite bloque o una sola sentencia
        let then_block = if self.check(&LBrace) {
            self.parse_block()?
        } else {
            vec![self.parse_stmt()?]
        };

        let mut arms = vec![(cond, then_block)];

        // else-if como inotest
        while self.match_next(&[KwInotest]) {
            self.consume(LParen, "Se esperaba '('")?;
            let c = self.parse_expr(0)?;
            self.consume(RParen, "Se esperaba ')'")?;
            let blk = if self.check(&LBrace) {
                self.parse_block()?
            } else {
                vec![self.parse_stmt()?]
            };
            arms.push((c, blk));
        }

        let else_block = if self.match_next(&[KwNotest]) {
            if self.check(&LBrace) {
                Some(self.parse_block()?)
            } else {
                Some(vec![self.parse_stmt()?])
            }
        } else {
            None
        };

        Ok(Stmt::If { arms, else_block })
    }

    fn parse_emit_stmt(&mut self, with_newline: bool) -> Result<Stmt, ParseError> {
        if with_newline {
            self.consume(TokenKind::KwEmitln, "Falta 'emitln'")?;
        } else {
            self.consume(TokenKind::KwEmit, "Falta 'emit'")?;
        }

        // Esperamos paréntesis de apertura
        self.consume(TokenKind::LParen, "Se esperaba '(' después de emit/emitln")?;

        let mut args = Vec::new();

        // Si no está vacío, parseamos argumentos separados por coma
        if !self.check(&TokenKind::RParen) {
            loop {
                args.push(self.parse_expr(0)?);

                if self.match_next(&[TokenKind::Comma]) {
                    continue; // Continúa con el siguiente argumento
                } else {
                    break;
                }
            }
        }

        // Esperamos paréntesis de cierre
        self.consume(
            TokenKind::RParen,
            "Se esperaba ')' después de los argumentos",
        )?;
        self.consume(TokenKind::Semi, "Se esperaba ';'")?;

        Ok(if with_newline {
            Stmt::EmitLn(args)
        } else {
            Stmt::Emit(args)
        })
    }

    fn parse_reaction_decl(&mut self) -> Result<Stmt, ParseError> {
        // reaction <Ident> '(' param_list? ')' block
        use TokenKind::*;
        self.consume(KwReaction, "Falta 'reaction'")?;
        let name = self.consume(Ident, "Se esperaba nombre de función")?.lexeme;
        self.consume(LParen, "Se esperaba '('")?;
        let mut params = Vec::new();
        if !self.check(&RParen) {
            loop {
                // Acepta dos formas:  nombre : tipo  ||  tipo nombre
                let first = self.peek().clone();
                let (pname, pty) = match first.kind {
                    // nombre : tipo
                    Ident => {
                        let pname = self.advance().lexeme.clone(); // Ident
                        self.consume(Colon, "Se esperaba ':' tras el nombre del parámetro")?;
                        let ty = self.parse_type()?;
                        (pname, ty)
                    }
                    // tipo nombre
                    KwAtomNum | KwMass | KwPolarized | KwVoidState | KwFormula | KwSymbol
                    | KwIon => {
                        let ty = self.parse_type()?; // ya consume el keyword
                        let pname = self
                            .consume(Ident, "Se esperaba nombre de parámetro tras el tipo")?
                            .lexeme;
                        (pname, ty)
                    }
                    _ => {
                        return Err(ParseError {
                            message: format!(
                                "Se esperaba parámetro (nombre: tipo) o (tipo nombre). Encontré {:?}",
                                first.kind
                            ),
                            line: first.line,
                            col: first.col,
                        });
                    }
                };

                params.push((pname, pty));
                if self.match_next(&[Comma]) {
                    continue;
                } else {
                    break;
                }
            }
        }
        self.consume(RParen, "Se esperaba ')'")?;
        let body = self.parse_block()?;
        Ok(Stmt::ReactionDecl { name, params, body })
    }

    fn parse_type(&mut self) -> Result<TypeName, ParseError> {
        // keywords conocidos + Ident como Custom
        let tk = self.advance().clone();
        let ty = match tk.kind {
            TokenKind::KwAtomNum => TypeName::AtomNum,
            TokenKind::KwMass => TypeName::Mass,
            TokenKind::KwPolarized => TypeName::Polarized,
            TokenKind::KwVoidState => TypeName::VoidState,
            TokenKind::KwFormula => TypeName::Formula,
            TokenKind::KwSymbol => TypeName::Symbol,
            TokenKind::KwIon => TypeName::Ion,
            TokenKind::Ident => TypeName::Custom(tk.lexeme),
            _ => {
                return Err(ParseError {
                    message: "Tipo no válido".into(),
                    line: tk.line,
                    col: tk.col,
                });
            }
        };
        Ok(ty)
    }

    // ====== EXPRESIONES (Pratt) ======

    fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        let mut lhs = {
            let t = self.peek().clone();
            if let Some(bp) = prefix_bp(&t.kind) {
                self.advance(); // consume op
                let rhs = self.parse_expr(bp)?;
                Expr::Unary {
                    op: t.kind,
                    rhs: Box::new(rhs),
                }
            } else {
                self.parse_primary()?
            }
        };

        loop {
            let op = self.peek().clone();
            if let Some((lbp, rbp)) = infix_bp(&op.kind) {
                if lbp < min_bp {
                    break;
                }
                self.advance(); // consume op
                let rhs = self.parse_expr(rbp)?;
                lhs = Expr::Binary {
                    lhs: Box::new(lhs),
                    op: op.kind,
                    rhs: Box::new(rhs),
                };
                continue;
            }
            break;
        }

        Ok(lhs)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let t = self.advance().clone();
        use TokenKind::*;
        match t.kind {
            Number => Ok(Expr::LitNumber(t.lexeme)),
            StringLit => Ok(Expr::LitString(t.lexeme)),
            Ident => Ok(Expr::Ident(t.lexeme)),
            LParen => {
                let e = self.parse_expr(0)?;
                self.consume(RParen, "Falta ')'")?;
                Ok(e)
            }
            _ => Err(ParseError {
                message: format!("Expresión inválida. Encontré {:?}", t.kind),
                line: t.line,
                col: t.col,
            }),
        }
    }

    #[allow(dead_code)]
    fn _synchronize(&mut self) {
        while !self.is_at_end() {
            if self.previous().kind == TokenKind::Semi {
                return;
            }
            use TokenKind::*;
            match self.peek().kind {
                KwAtom | KwMolecule | KwIon | KwITest | KwEmitln | KwEmit | KwReaction | LBrace => {
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }
}

// ====== Precedencias / binding powers ======
fn infix_bp(op: &TokenKind) -> Option<(u8, u8)> {
    use TokenKind::*;
    match op {
        Or => Some((1, 2)),
        And => Some((3, 4)),
        Eq | Ne => Some((5, 6)),
        Lt | Le | Gt | Ge => Some((7, 8)),
        Plus | Minus => Some((9, 10)),
        Star | Slash => Some((11, 12)),
        _ => None,
    }
}

fn prefix_bp(op: &TokenKind) -> Option<u8> {
    use TokenKind::*;
    match op {
        Minus | Not | Plus => Some(13), // admite +unario
        _ => None,
    }
}

// ============ Pretty print helpers (opcional) ============

pub fn print_ast(stmts: &Program) {
    fn pad(n: usize) -> String {
        " ".repeat(n)
    }
    fn print_block(b: &Block, indent: usize) {
        for s in b {
            print_stmt(s, indent);
        }
    }
    fn print_stmt(s: &Stmt, indent: usize) {
        let p = pad(indent);
        match s {
            Stmt::VarDecl { name, ty, init } => {
                println!(
                    "{p}VarDecl {} : {:?} {}",
                    name,
                    ty,
                    init.as_ref().map(|e| e.to_string()).unwrap_or_default()
                );
            }
            Stmt::ConstDecl { name, ty, value } => {
                println!("{p}ConstDecl {} : {:?} {}", name, ty, value);
            }
            Stmt::Assign { name, value } => {
                println!("{p}Assign {} = {}", name, value);
            }
            Stmt::EmitLn(args) => {
                let args_str = args
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                println!("{p}Emitln({})", args_str);
            }
            Stmt::Emit(args) => {
                let args_str = args
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                println!("{p}Emit({})", args_str);
            }
            Stmt::If { arms, else_block } => {
                println!("{p}If");
                for (i, (cond, blk)) in arms.iter().enumerate() {
                    println!("{}  Arm #{} cond = {}", p, i, cond);
                    print_block(blk, indent + 4);
                }
                if let Some(eb) = else_block {
                    println!("{}  Else", p);
                    print_block(eb, indent + 4);
                }
            }
            Stmt::ReactionDecl { name, params, body } => {
                println!("{p}Reaction {}(", name);
                for (n, t) in params {
                    println!("{}  param {} : {:?}", p, n, t);
                }
                println!("{p}) Body {{");
                print_block(body, indent + 2);
                println!("{p}}}");
            }
            Stmt::Block(b) => {
                println!("{p}Block {{");
                print_block(b, indent + 2);
                println!("{p}}}");
            }
            Stmt::ExprStmt(e) => println!("{p}ExprStmt {}", e),
            _ => println!("{p}{:?}", s),
        }
    }
    for s in stmts {
        print_stmt(s, 0);
    }
}

// ===================== Tests basados en la 2da versión + extras =====================
/*
#[cfg(test)]
mod tests {
    use super::*;

    fn t(k: TokenKind, s: &str) -> Token {
        Token { kind: k, lexeme: s.into(), line: 1, col: 1 }
    }

    #[test]
    fn var_decl_and_expr() {
        // atom x : atom_num = 5 + 3 * 2;
        let toks = vec![
            t(TokenKind::KwAtom, "atom"),
            t(TokenKind::Ident, "x"),
            t(TokenKind::Colon, ":"),
            t(TokenKind::KwAtomNum, "atom_num"),
            t(TokenKind::Assign, "="),
            t(TokenKind::Number, "5"),
            t(TokenKind::Plus, "+"),
            t(TokenKind::Number, "3"),
            t(TokenKind::Star, "*"),
            t(TokenKind::Number, "2"),
            t(TokenKind::Semi, ";"),
            t(TokenKind::EndOfFile, ""),
        ];
        let mut p = Parser::new(toks);
        let prog = p.parse_program().unwrap();
        assert!(matches!(prog[0], Stmt::VarDecl { .. }));
    }

    #[test]
    fn assign_and_logic() {
        // x = 1 + 2 * 3 == 7 and not (0);
        let toks = vec![
            t(TokenKind::Ident, "x"),
            t(TokenKind::Assign, "="),
            t(TokenKind::Number, "1"),
            t(TokenKind::Plus, "+"),
            t(TokenKind::Number, "2"),
            t(TokenKind::Star, "*"),
            t(TokenKind::Number, "3"),
            t(TokenKind::Eq, "=="),
            t(TokenKind::Number, "7"),
            t(TokenKind::And, "and"),
            t(TokenKind::Not, "not"),
            t(TokenKind::LParen, "("),
            t(TokenKind::Number, "0"),
            t(TokenKind::RParen, ")"),
            t(TokenKind::Semi, ";"),
            t(TokenKind::EndOfFile, ""),
        ];
        let mut p = Parser::new(toks);
        let prog = p.parse_program().unwrap();
        assert!(matches!(prog[0], Stmt::Assign { .. }));
    }

    #[test]
    fn if_with_inotest_and_else() {
        // itest (1) { emitln "a"; } inotest (0) { emitln "b"; } notest { emitln "c"; }
        let toks = vec![
            t(TokenKind::KwITest, "itest"),
            t(TokenKind::LParen, "("),
            t(TokenKind::Number, "1"),
            t(TokenKind::RParen, ")"),
            t(TokenKind::LBrace, "{"),
            t(TokenKind::KwEmitln, "emitln"),
            t(TokenKind::StringLit, "a"),
            t(TokenKind::Semi, ";"),
            t(TokenKind::RBrace, "}"),
            t(TokenKind::KwInotest, "inotest"),
            t(TokenKind::LParen, "("),
            t(TokenKind::Number, "0"),
            t(TokenKind::RParen, ")"),
            t(TokenKind::LBrace, "{"),
            t(TokenKind::KwEmitln, "emitln"),
            t(TokenKind::StringLit, "b"),
            t(TokenKind::Semi, ";"),
            t(TokenKind::RBrace, "}"),
            t(TokenKind::KwNotest, "notest"),
            t(TokenKind::LBrace, "{"),
            t(TokenKind::KwEmitln, "emitln"),
            t(TokenKind::StringLit, "c"),
            t(TokenKind::Semi, ";"),
            t(TokenKind::RBrace, "}"),
            t(TokenKind::EndOfFile, ""),
        ];
        let mut p = Parser::new(toks);
        let prog = p.parse_program().unwrap();
        assert!(matches!(prog[0], Stmt::If { .. }));
    }

    #[test]
    fn reaction_two_param_forms() {
        // reaction f(a: atom_num, mass b) { emitln "ok"; }
        let toks = vec![
            t(TokenKind::KwReaction, "reaction"),
            t(TokenKind::Ident, "f"),
            t(TokenKind::LParen, "("),
            t(TokenKind::Ident, "a"),
            t(TokenKind::Colon, ":"),
            t(TokenKind::KwAtomNum, "atom_num"),
            t(TokenKind::Comma, ","),
            t(TokenKind::KwMass, "mass"),
            t(TokenKind::Ident, "b"),
            t(TokenKind::RParen, ")"),
            t(TokenKind::LBrace, "{"),
            t(TokenKind::KwEmitln, "emitln"),
            t(TokenKind::StringLit, "ok"),
            t(TokenKind::Semi, ";"),
            t(TokenKind::RBrace, "}"),
            t(TokenKind::EndOfFile, ""),
        ];
        let mut p = Parser::new(toks);
        let prog = p.parse_program().unwrap();
        assert!(matches!(prog[0], Stmt::ReactionDecl { .. }));
    }

    #[test]
    fn custom_type_in_var_decl() {
        // atom x : Vector3;
        let toks = vec![
            t(TokenKind::KwAtom, "atom"),
            t(TokenKind::Ident, "x"),
            t(TokenKind::Colon, ":"),
            t(TokenKind::Ident, "Vector3"), // TypeName::Custom("Vector3")
            t(TokenKind::Semi, ";"),
            t(TokenKind::EndOfFile, ""),
        ];
        let mut p = Parser::new(toks);
        let prog = p.parse_program().unwrap();
        match &prog[0] {
            Stmt::VarDecl { ty, .. } => match ty {
                TypeName::Custom(s) => assert_eq!(s, "Vector3"),
                _ => panic!("Se esperaba TypeName::Custom"),
            },
            _ => panic!("Se esperaba VarDecl"),
        }
    }
}
*/
