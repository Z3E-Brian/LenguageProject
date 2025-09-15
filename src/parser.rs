use crate::utils::enums::TokenKind;
use crate::utils::enums::TypeName;
use crate::utils::enums::Stmt;
use crate::utils::enums::Expr;
use crate::utils::token::Token;
use std::fmt;

// ===================== AST =====================

pub type Program = Vec<Stmt>;
pub type Block = Vec<Stmt>;

// Hace que los campos de Expr “se usen” al imprimir y evita warnings de dead_code
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::LitNumber(s)   => write!(f, "{s}"),
            Expr::LitString(s)   => write!(f, "\"{s}\""),
            Expr::Ident(id)      => write!(f, "{id}"),
            Expr::Unary { op, rhs } =>
                write!(f, "({:?} {})", op, rhs),
            Expr::Binary { lhs, op, rhs } =>
                write!(f, "({} {:?} {})", lhs, op, rhs),
        }
    }
}

// ===================== Parser =====================

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { Self { tokens, pos: 0 } }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(self.tokens.last().unwrap())
    }
    fn is_at_end(&self) -> bool { self.peek().kind == TokenKind::EndOfFile }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() { self.pos += 1; }
        self.previous()
    }
    fn previous(&self) -> &Token { &self.tokens[self.pos - 1] }

    fn check(&self, kind: &TokenKind) -> bool { !self.is_at_end() && &self.peek().kind == kind }

    fn match_next(&mut self, kinds: &[TokenKind]) -> bool {
        for k in kinds {
            if self.check(k) { self.advance(); return true; }
        }
        false
    }

    fn consume(&mut self, expected: TokenKind, msg: &str) -> Result<Token, ParseError> {
        if self.check(&expected) { Ok(self.advance().clone()) }
        else {
            let t = self.peek();
            Err(ParseError { message: format!("{msg}. Encontré {:?}", t.kind), line: t.line, col: t.col })
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        use TokenKind::*;
        match self.peek().kind {
            KwAtom | KwMolecule => self.parse_var_decl(),
            KwIon               => self.parse_const_decl(),
            KwITest             => self.parse_if_stmt(),
            KwEmitln            => self.parse_emit_stmt(true),
            KwEmit              => self.parse_emit_stmt(false),
            KwReaction          => self.parse_reaction_decl(),
            LBrace              => self.parse_block().map(Stmt::Block),
            Ident               => self.parse_assign_stmt(),
            _ => {
                let t = self.peek().clone();
                Err(ParseError { message: format!("Token inesperado en sentencia: {:?}", t.kind), line: t.line, col: t.col })
            }
        }
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        self.consume(TokenKind::LBrace, "Se esperaba '{' para iniciar bloque")?;
        let mut items = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            items.push(self.parse_stmt()?);
        }
        self.consume(TokenKind::RBrace, "Se esperaba '}' para cerrar bloque")?;
        Ok(items)
    }

    fn parse_var_decl(&mut self) -> Result<Stmt, ParseError> {
        // KwAtom | KwMolecule <Ident> : <Type> ( '=' expr )? ';'
        self.advance(); // consume KwAtom/KwMolecule
        let name = self.consume(TokenKind::Ident, "Se esperaba nombre de variable")?.lexeme;
        self.consume(TokenKind::Colon, "Se esperaba ':'")?;
        let ty = self.parse_type()?;
        let init = if self.match_next(&[TokenKind::Assign]) {
            Some(self.parse_expr(0)?)
        } else { None };
        self.consume(TokenKind::Semi, "Se esperaba ';'")?;
        Ok(Stmt::VarDecl { name, ty, init })
    }

    fn parse_const_decl(&mut self) -> Result<Stmt, ParseError> {
        // KwIon <Ident> : <Type> = expr ';'
        self.consume(TokenKind::KwIon, "Falta 'ion'")?;
        let name = self.consume(TokenKind::Ident, "Se esperaba nombre de constante")?.lexeme;
        self.consume(TokenKind::Colon, "Se esperaba ':'")?;
        let ty = self.parse_type()?;
        self.consume(TokenKind::Assign, "Se esperaba '='")?;
        let value = self.parse_expr(0)?;
        self.consume(TokenKind::Semi, "Se esperaba ';'")?;
        Ok(Stmt::ConstDecl { name, ty, value })
    }

    fn parse_assign_stmt(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenKind::Ident, "Se esperaba identificador")?.lexeme;
        self.consume(TokenKind::Assign, "Se esperaba '='")?;
        let value = self.parse_expr(0)?;
        self.consume(TokenKind::Semi, "Se esperaba ';'")?;
        Ok(Stmt::Assign { name, value })
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::KwITest, "Falta 'itest'")?;
        self.consume(TokenKind::LParen, "Se esperaba '('")?;
        let cond = self.parse_expr(0)?;
        self.consume(TokenKind::RParen, "Se esperaba ')'")?;

        // Permite bloque o una sola sentencia
        let then_block = if self.check(&TokenKind::LBrace) {
            self.parse_block()?
        } else {
            vec![ self.parse_stmt()? ]
        };

        let mut arms = vec![(cond, then_block)];

        while self.match_next(&[TokenKind::KwInotest]) {
            self.consume(TokenKind::LParen, "Se esperaba '('")?;
            let c = self.parse_expr(0)?;
            self.consume(TokenKind::RParen, "Se esperaba ')'")?;
            let blk = if self.check(&TokenKind::LBrace) {
                self.parse_block()?
            } else { vec![ self.parse_stmt()? ] };
            arms.push((c, blk));
        }

        let else_block = if self.match_next(&[TokenKind::KwNotest]) {
            if self.check(&TokenKind::LBrace) {
                Some(self.parse_block()?)
            } else {
                Some(vec![ self.parse_stmt()? ])
            }
        } else { None };

        Ok(Stmt::If { arms, else_block })
    }

    fn parse_emit_stmt(&mut self, with_newline: bool) -> Result<Stmt, ParseError> {
        if with_newline {
            self.consume(TokenKind::KwEmitln, "Falta 'emitln'")?;
        } else {
            self.consume(TokenKind::KwEmit, "Falta 'emit'")?;
        }
        let e = self.parse_expr(0)?;
        self.consume(TokenKind::Semi, "Se esperaba ';'")?;
        Ok(if with_newline { Stmt::EmitLn(e) } else { Stmt::Emit(e) })
    }

    fn parse_reaction_decl(&mut self) -> Result<Stmt, ParseError> {
        // reaction <Ident> '(' param_list? ')' block
        self.consume(TokenKind::KwReaction, "Falta 'reaction'")?;
        let name = self.consume(TokenKind::Ident, "Se esperaba nombre de función")?.lexeme;
        self.consume(TokenKind::LParen, "Se esperaba '('")?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                // Acepta dos formas:  nombre : tipo  ||  tipo nombre
                use TokenKind::*;

                // helper: intenta leer un tipo
                let read_type = |this: &mut Parser| -> Option<TypeName> {
                    match this.peek().kind {
                        KwAtomNum | KwMass | KwPolarized | KwVoidState | KwFormula | KwSymbol | KwIon => {
                            let k = this.advance().kind; // consume keyword de tipo
                            Some(match k {
                                KwAtomNum   => TypeName::AtomNum,
                                KwMass      => TypeName::Mass,
                                KwPolarized => TypeName::Polarized,
                                KwVoidState => TypeName::VoidState,
                                KwFormula   => TypeName::Formula,
                                KwSymbol    => TypeName::Symbol,
                                KwIon       => TypeName::Ion,
                                _ => unreachable!(),
                            })
                        }
                        _ => None,
                    }
                };

                let first = self.peek().clone();
                let (pname, pty) = match first.kind {
                    // nombre : tipo
                    Ident => {
                        let name = self.advance().lexeme.clone(); // Ident
                        self.consume(Colon, "Se esperaba ':' tras el nombre del parámetro")?;
                        let ty = self.parse_type()?;
                        (name, ty)
                    }
                    // tipo nombre
                    KwAtomNum | KwMass | KwPolarized | KwVoidState | KwFormula | KwSymbol | KwIon => {
                        let ty = read_type(self).expect("tipo válido");
                        let name = self.consume(Ident, "Se esperaba nombre de parámetro tras el tipo")?.lexeme;
                        (name, ty)
                    }
                    _ => {
                        return Err(ParseError {
                            message: format!("Se esperaba parámetro (nombre: tipo) o (tipo nombre). Encontré {:?}", first.kind),
                            line: first.line, col: first.col
                        });
                    }
                };

                params.push((pname, pty));
                if self.match_next(&[Comma]) { continue; } else { break; }
            }
        }
        self.consume(TokenKind::RParen, "Se esperaba ')'")?;
        let body = self.parse_block()?;
        Ok(Stmt::ReactionDecl { name, params, body })
    }

    fn parse_type(&mut self) -> Result<TypeName, ParseError> {
        let tk = self.advance().clone();
        let ty = match tk.kind {
            TokenKind::KwAtomNum   => TypeName::AtomNum,
            TokenKind::KwMass      => TypeName::Mass,
            TokenKind::KwPolarized => TypeName::Polarized,
            TokenKind::KwVoidState => TypeName::VoidState,
            TokenKind::KwFormula   => TypeName::Formula,
            TokenKind::KwSymbol    => TypeName::Symbol,
            TokenKind::KwIon       => TypeName::Ion,
            _ => return Err(ParseError{ message: "Tipo no válido".into(), line: tk.line, col: tk.col }),
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
                Expr::Unary { op: t.kind, rhs: Box::new(rhs) }
            } else {
                self.parse_primary()?
            }
        };

        loop {
            let op = self.peek().clone();
            if let Some((lbp, rbp)) = infix_bp(&op.kind) {
                if lbp < min_bp { break; }
                self.advance(); // consume op
                let rhs = self.parse_expr(rbp)?;
                lhs = Expr::Binary { lhs: Box::new(lhs), op: op.kind, rhs: Box::new(rhs) };
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
            Number    => Ok(Expr::LitNumber(t.lexeme)),
            StringLit => Ok(Expr::LitString(t.lexeme)),
            Ident     => Ok(Expr::Ident(t.lexeme)),
            LParen    => {
                let e = self.parse_expr(0)?;
                self.consume(RParen, "Falta ')'")?;
                Ok(e)
            }
            _ => Err(ParseError { message: format!("Expresión inválida. Encontré {:?}", t.kind), line: t.line, col: t.col })
        }
    }

    #[allow(dead_code)]
    fn _synchronize(&mut self) {
        while !self.is_at_end() {
            if self.previous().kind == TokenKind::Semi { return; }
            use TokenKind::*;
            match self.peek().kind {
                KwAtom | KwMolecule | KwIon | KwITest | KwEmitln | KwEmit | KwReaction | LBrace => return,
                _ => { self.advance(); }
            }
        }
    }
}

// ====== Precedencias / binding powers ======

fn infix_bp(op: &TokenKind) -> Option<(u8, u8)> {
    use TokenKind::*;
    match op {
        Or                  => Some((1, 2)),
        And                 => Some((3, 4)),
        Eq | Ne             => Some((5, 6)),
        Lt | Le | Gt | Ge   => Some((7, 8)),
        Plus | Minus        => Some((9, 10)),
        Star | Slash        => Some((11, 12)),
        _ => None,
    }
}

fn prefix_bp(op: &TokenKind) -> Option<u8> {
    use TokenKind::*;
    match op {
        Minus | Not => Some(13),
        _ => None,
    }
}

// ============ Pretty print helpers (opcional) ============

pub fn print_ast(stmts: &Program) {
    fn pad(n: usize) -> String { " ".repeat(n) }
    fn print_block(b: &Block, indent: usize) {
        for s in b {
            print_stmt(s, indent);
        }
    }
    fn print_stmt(s: &Stmt, indent: usize) {
        let p = pad(indent);
        match s {
            Stmt::VarDecl { name, ty, init } => {
                println!("{p}VarDecl {} : {:?} {}", name, ty, init.as_ref().map(|e| e.to_string()).unwrap_or_default());
            }
            Stmt::ConstDecl { name, ty, value } => {
                println!("{p}ConstDecl {} : {:?} {}", name, ty, value);
            }
            Stmt::Assign { name, value } => {
                println!("{p}Assign {} = {}", name, value);
            }
            Stmt::EmitLn(e) => println!("{p}Emitln {}", e),
            Stmt::Emit(e) => println!("{p}Emit {}", e),
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
                for (n, t) in params { println!("{}  param {} : {:?}", p, n, t); }
                println!("{p}) Body {{");
                print_block(body, indent + 2);
                println!("{p}}}");
            }
            Stmt::Block(b) => {
                println!("{p}Block {{");
                print_block(b, indent + 2);
                println!("{p}}}");
            }
        }
    }
    for s in stmts { print_stmt(s, 0); }
}
