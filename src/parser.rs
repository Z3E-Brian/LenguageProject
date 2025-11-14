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
            Expr::LitChar(c) => write!(f, "'{c}'"),
            Expr::LitTrue => write!(f, "pos"),
            Expr::LitFalse => write!(f, "neg"),
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
            KwCapture => self.parse_capture_stmt(),
            KwReaction => self.parse_reaction_decl(),
            KwRelease => self.parse_release_stmt(),
            KwChain => self.parse_chain_stmt(),
            KwOrbite => self.parse_orbite_stmt(),
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

    fn parse_capture_stmt(&mut self) -> Result<Stmt, ParseError> {
        // capture(variable_name);
        self.consume(TokenKind::KwCapture, "Falta 'capture'")?;
        self.consume(TokenKind::LParen, "Se esperaba '(' después de capture")?;
        let var_name = self
            .consume(TokenKind::Ident, "Se esperaba nombre de variable")?
            .lexeme;
        self.consume(
            TokenKind::RParen,
            "Se esperaba ')' después del nombre de variable",
        )?;
        self.consume(TokenKind::Semi, "Se esperaba ';'")?;
        Ok(Stmt::Capture { var_name })
    }

    fn parse_release_stmt(&mut self) -> Result<Stmt, ParseError> {
        // release <expr>;
        self.consume(TokenKind::KwRelease, "Falta 'release'")?;
        let value = self.parse_expr(0)?;
        self.consume(TokenKind::Semi, "Se esperaba ';' después de release")?;
        Ok(Stmt::Release(value))
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

        // Parsear tipo de retorno: : tipo
        self.consume(
            Colon,
            "Se esperaba ':' tras ')' para especificar tipo de retorno",
        )?;
        let return_type = self.parse_type()?;

        let body = self.parse_block()?;
        Ok(Stmt::ReactionDecl {
            name,
            params,
            return_type,
            body,
        })
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
            //Soporte para solution<T> (vectores/arrays)
            TokenKind::KwSolution => {
                // Esperar '<'
                self.consume(TokenKind::Lt, "Se esperaba '<' después de 'solution'")?;

                // Parsear el tipo interno recursivamente
                let inner_type = self.parse_type()?;

                // Esperar '>'
                self.consume(
                    TokenKind::Gt,
                    "Se esperaba '>' para cerrar el tipo 'solution'",
                )?;

                TypeName::Solution(Box::new(inner_type))
            }
            // Soporte para sample<T>
            TokenKind::KwSample => {
                // Esperar '<'
                self.consume(TokenKind::Lt, "Se esperaba '<' después de 'sample'")?;

                // Parsear el tipo interno recursivamente
                let inner_type = self.parse_type()?;

                // Esperar '>'
                self.consume(
                    TokenKind::Gt,
                    "Se esperaba '>' para cerrar el tipo 'sample'",
                )?;

                TypeName::Sample(Box::new(inner_type))
            }
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

    fn parse_chain_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::KwChain, "Se esperaba 'chain'")?;

        let start_token = self.consume(
            TokenKind::Number,
            "chain solo acepta números literales (no variables)",
        )?;
        let start = start_token.lexeme.parse::<i32>().map_err(|_| ParseError {
            message: "Número inválido para chain".to_string(),
            line: start_token.line,
            col: start_token.col,
        })?;

        let end = if self.match_next(&[TokenKind::KwTo]) {
            let end_token = self.consume(
                TokenKind::Number,
                "chain 'to' solo acepta números literales (no variables)",
            )?;
            Some(end_token.lexeme.parse::<i32>().map_err(|_| ParseError {
                message: "Número inválido para chain end".to_string(),
                line: end_token.line,
                col: end_token.col,
            })?)
        } else {
            None
        };

        let body = self.parse_block()?;

        Ok(Stmt::Chain {
            start: Some(start),
            end,
            body,
        })
    }

    fn parse_orbite_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::KwOrbite, "Se esperaba 'orbite'")?;
        self.consume(TokenKind::LParen, "Se esperaba '(' después de 'orbite'")?;
        let condition = self.parse_expr(0)?;
        self.consume(TokenKind::RParen, "Se esperaba ')' después de la condición")?;
        let body = self.parse_block()?;
        
        Ok(Stmt::Orbite { condition, body })
    }

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

            // Manejar operadores postfijos: indexación [idx] y llamadas a métodos .method()
            if op.kind == TokenKind::LBracket {
                // Indexación: expr[index]
                self.advance(); // consumir '['
                let index = self.parse_expr(0)?;
                self.consume(TokenKind::RBracket, "Falta ']' después del índice")?;
                lhs = Expr::Index {
                    target: Box::new(lhs),
                    index: Box::new(index),
                };
                continue;
            }

            // Llamadas a métodos: expr.method(args)
            if op.kind == TokenKind::Dot {
                self.advance(); // consumir '.'
                let method_name = self.consume(
                    TokenKind::Ident,
                    "Se esperaba nombre de método después de '.'",
                )?;

                // Verificar si hay paréntesis para argumentos
                let args = if self.check(&TokenKind::LParen) {
                    self.advance(); // consumir '('
                    let mut arguments = Vec::new();

                    if !self.check(&TokenKind::RParen) {
                        loop {
                            arguments.push(self.parse_expr(0)?);
                            if !self.check(&TokenKind::Comma) {
                                break;
                            }
                            self.advance(); // consumir ','
                        }
                    }

                    self.consume(TokenKind::RParen, "Falta ')' después de argumentos")?;
                    arguments
                } else {
                    Vec::new()
                };

                lhs = Expr::MethodCall {
                    receiver: Box::new(lhs),
                    name: method_name.lexeme.clone(),
                    args,
                };
                continue;
            }

            // Operadores binarios normales
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
            CharLit => {
                let ch = t.lexeme.chars().next().unwrap_or('\0');
                Ok(Expr::LitChar(ch))
            }
            KwTrue => Ok(Expr::LitTrue),
            KwFalse => Ok(Expr::LitFalse),
            Ident => {
                // Verificar si es una llamada a función: nombre(args)
                if self.check(&LParen) {
                    self.advance(); // consumir '('
                    let mut args = Vec::new();

                    if !self.check(&RParen) {
                        loop {
                            args.push(self.parse_expr(0)?);
                            if !self.check(&Comma) {
                                break;
                            }
                            self.advance(); // consumir ','
                        }
                    }

                    self.consume(RParen, "Falta ')' después de argumentos de función")?;
                    Ok(Expr::FunctionCall {
                        name: t.lexeme,
                        args,
                    })
                } else {
                    Ok(Expr::Ident(t.lexeme))
                }
            }
            LParen => {
                let e = self.parse_expr(0)?;
                self.consume(RParen, "Falta ')'")?;
                Ok(e)
            }
            LBracket => {
                let mut items = Vec::new();

                // Vector vacío: []
                if self.check(&RBracket) {
                    self.advance();
                    return Ok(Expr::VecLiteral(items));
                }

                // Parsear elementos separados por comas
                loop {
                    items.push(self.parse_expr(0)?);

                    if !self.check(&Comma) {
                        break;
                    }
                    self.advance(); // consumir la coma
                }

                self.consume(RBracket, "Falta ']' al final del vector")?;
                Ok(Expr::VecLiteral(items))
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
                KwAtom | KwMolecule | KwIon | KwITest | KwEmitln | KwEmit | KwCapture
                | KwReaction | LBrace => {
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }
}

fn infix_bp(op: &TokenKind) -> Option<(u8, u8)> {
    use TokenKind::*;
    match op {
        Or => Some((1, 2)),
        And => Some((3, 4)),
        Eq | Ne => Some((5, 6)),
        Lt | Le | Gt | Ge => Some((7, 8)),
        Plus | Minus => Some((9, 10)),
        Star | Slash | Percent => Some((11, 12)),
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
            Stmt::Capture { var_name } => {
                println!("{p}Capture({})", var_name);
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
            Stmt::ReactionDecl {
                name,
                params,
                return_type,
                body,
            } => {
                println!("{p}Reaction {}(", name);
                for (n, t) in params {
                    println!("{}  param {} : {:?}", p, n, t);
                }
                println!("{p}) : {:?} Body {{", return_type);
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
