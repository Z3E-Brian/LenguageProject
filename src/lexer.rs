mod tokenkind;use tokenkind::TokenKind;
use std::fmt;

//--------------------------------------- Clase Token ---------------------------------------//
#[derive(Debug, Clone)]
pub struct Token {
    kind: TokenKind,
    lexeme: String,
    line: usize,
    col: usize,
}
impl Token {
    pub fn get_kind(&self) -> TokenKind {
        self.kind
    }

    pub fn get_lexeme(&self) -> &str {
        &self.lexeme
    }

    pub fn get_line(&self) -> usize {
        self.line
    }

    pub fn get_col(&self) -> usize {
        self.col
    }
}
impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TokenKind::*;
        let s = match self {
            KwITest => "KW_ITEST",
            KwNotest => "KW_NOTEST",
            KwInotest => "KW_INOTEST",
            KwInhibit => "KW_INHIBIT",
            KwRelease => "KW_RELEASE",
            KwEmitln => "KW_EMITLN",
            KwEmit => "KW_EMIT",
            KwSynthesize => "KW_SYNTHESIZE",
            KwAtom => "KW_ATOM",
            KwMolecule => "KW_MOLECULE",
            KwReaction => "KW_REACTION",
            KwSymbol => "KW_SYMBOL",
            KwAtomNum => "KW_ATOM_NUM",
            KwMass => "KW_MASS",
            KwPolarized => "KW_POLARIZED",
            KwVoidState => "KW_VOIDSTATE",
            KwFormula => "KW_FORMULA",
            KwIon => "KW_ION",
            And => "AND",
            Or => "OR",
            Not => "NOT",
            Ident => "IDENT",
            Number => "NUMBER",
            StringLit => "STRING",
            Assign => "ASSIGN",
            Plus => "PLUS",
            Minus => "MINUS",
            Star => "STAR",
            Slash => "SLASH",
            Semi => "SEMI",
            LParen => "LP",
            RParen => "RP",
            LBrace => "LBRACE",
            RBrace => "RBRACE",
            LBracket => "LBRACKET",
            RBracket => "RBRACKET",
            Colon => "COLON",
            Comma => "COMMA",
            Lt => "LT",
            Gt => "GT",
            Le => "LE",
            Ge => "GE",
            Eq => "EQ",
            Ne => "NE",
            EndOfFile => "EOF",
        };
        write!(f, "{s}")
    }
}
//------------------------------------- Fin Clase Token -------------------------------------//


// --- util de keywords (case-insensitive) ---
fn keyword_kind(s_lower: &str) -> Option<TokenKind> {
    use TokenKind::*;
    Some(match s_lower {
        // control
        "itest" => KwITest,
        "notest" => KwNotest,
        "inotest" => KwInotest,
        "inhibit" => KwInhibit,
        "release" => KwRelease,
        "emitln" => KwEmitln,
        "emit" => KwEmit,
        "synthesize" => KwSynthesize,
        // declaración
        "atom" => KwAtom,
        "molecule" => KwMolecule,
        "reaction" => KwReaction,
        // tipos
        "symbol" => KwSymbol,
        "atom_num" => KwAtomNum,
        "mass" => KwMass,
        "polarized" => KwPolarized,
        "voidstate" => KwVoidState, // case-insensitive
        "formula" => KwFormula,
        "ion" => KwIon,
        // lógicos
        "and" => And,
        "or" => Or,
        "not" => Not,
        _ => return None,
    })
}


//--------------------------------------- Clase Lexer ---------------------------------------//
pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}
impl Lexer {
    pub fn new(src: &str) -> Self {
        Self {
            chars: src.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    // función principal de lexing
    pub fn lex_all(&mut self) -> Result<Vec<Token>, String> {
        let mut toks: Vec<Token> = Vec::new();

        while let Some(c) = self.peek(0) {
            // espacios y saltos
            if c.is_whitespace() {
                self.consume_whitespace();
                continue;
            }

            // comentarios de línea: !!
            if c == '!' && self.peek(1) == Some('!') {
                self.skip_line_comment();
                continue;
            }

            // identificadores/keywords
            if Self::is_alpha(c) {
                let (lexeme, line, col) = self.consume_ident_like();
                let lower = lexeme.to_lowercase();
                let kind = keyword_kind(&lower).unwrap_or(TokenKind::Ident);
                toks.push(Token {
                    kind,
                    lexeme,
                    line,
                    col,
                });
                continue;
            }

            // números: entero / decimal / exponente
            if c.is_ascii_digit() {
                let (lexeme, line, col) = self.consume_number()?;
                toks.push(Token {
                    kind: TokenKind::Number,
                    lexeme,
                    line,
                    col,
                });
                continue;
            }

            // strings con escapes
            if c == '"' {
                let (content, line, col) = self.consume_string()?;
                toks.push(Token {
                    kind: TokenKind::StringLit,
                    lexeme: content,
                    line,
                    col,
                });
                continue;
            }

            // operadores de 2 caracteres
            if let Some(tok) = self.try_two_char_op() {
                toks.push(tok);
                continue;
            }

            // operadores de 1 carácter y signos
            let sc = self.col;
            let line = self.line;
            let ch = self.next().unwrap(); // avanza uno
            let token = match ch {
                '=' => Token {
                    kind: TokenKind::Assign,
                    lexeme: "=".into(),
                    line,
                    col: sc,
                },
                '+' => Token {
                    kind: TokenKind::Plus,
                    lexeme: "+".into(),
                    line,
                    col: sc,
                },
                '-' => Token {
                    kind: TokenKind::Minus,
                    lexeme: "-".into(),
                    line,
                    col: sc,
                },
                '*' => Token {
                    kind: TokenKind::Star,
                    lexeme: "*".into(),
                    line,
                    col: sc,
                },
                '/' => Token {
                    kind: TokenKind::Slash,
                    lexeme: "/".into(),
                    line,
                    col: sc,
                },
                ';' => Token {
                    kind: TokenKind::Semi,
                    lexeme: ";".into(),
                    line,
                    col: sc,
                },
                '(' => Token {
                    kind: TokenKind::LParen,
                    lexeme: "(".into(),
                    line,
                    col: sc,
                },
                ')' => Token {
                    kind: TokenKind::RParen,
                    lexeme: ")".into(),
                    line,
                    col: sc,
                },
                '{' => Token {
                    kind: TokenKind::LBrace,
                    lexeme: "{".into(),
                    line,
                    col: sc,
                },
                '}' => Token {
                    kind: TokenKind::RBrace,
                    lexeme: "}".into(),
                    line,
                    col: sc,
                },
                '[' => Token {
                    kind: TokenKind::LBracket,
                    lexeme: "[".into(),
                    line,
                    col: sc,
                },
                ']' => Token {
                    kind: TokenKind::RBracket,
                    lexeme: "]".into(),
                    line,
                    col: sc,
                },
                ':' => Token {
                    kind: TokenKind::Colon,
                    lexeme: ":".into(),
                    line,
                    col: sc,
                },
                ',' => Token {
                    kind: TokenKind::Comma,
                    lexeme: ",".into(),
                    line,
                    col: sc,
                },
                '<' => Token {
                    kind: TokenKind::Lt,
                    lexeme: "<".into(),
                    line,
                    col: sc,
                },
                '>' => Token {
                    kind: TokenKind::Gt,
                    lexeme: ">".into(),
                    line,
                    col: sc,
                },
                _ => {
                    let msg = format!("Caracter inesperado '{}' en linea {}, col {}", ch, line, sc);
                    return Err(msg);
                }
            };
            toks.push(token);
        }

        toks.push(Token {
            kind: TokenKind::EndOfFile,
            lexeme: String::new(),
            line: self.line,
            col: self.col,
        });
        Ok(toks)
    }

    // --- HELPERS DE CONSUMO ---

    // para espacios y saltos de línea
    fn consume_whitespace(&mut self) {
        while let Some(c) = self.peek(0) {
            if c == '\r' {
                self.next();
                continue;
            } // tolera CRLF
            if !c.is_whitespace() {
                break;
            }
            self.next();
        }
    }

    // para "!!"
    fn skip_line_comment(&mut self) {
        self.next();
        self.next();
        while let Some(c) = self.peek(0) {
            if c == '\n' {
                break;
            }
            self.next();
        }
    }

    // para identificadores y keywords
    fn consume_ident_like(&mut self) -> (String, usize, usize) {
        let start_line = self.line;
        let start_col = self.col;
        let mut s = String::new();
        if let Some(c) = self.next() {
            s.push(c);
        }
        while let Some(c) = self.peek(0) {
            if Self::is_alnum(c) {
                s.push(self.next().unwrap());
            } else {
                break;
            }
        }
        (s, start_line, start_col)
    }

    // para números (entero, decimal, exponente)
    fn consume_number(&mut self) -> Result<(String, usize, usize), String> {
        let start_line = self.line;
        let start_col = self.col;
        let mut s = String::new();

        // parte entera
        while let Some(c) = self.peek(0) {
            if c.is_ascii_digit() {
                s.push(self.next().unwrap());
            } else {
                break;
            }
        }

        // decimal
        if self.peek(0) == Some('.') && self.peek(1).map(|c| c.is_ascii_digit()).unwrap_or(false) {
            s.push(self.next().unwrap()); // '.'
            while let Some(c) = self.peek(0) {
                if c.is_ascii_digit() {
                    s.push(self.next().unwrap());
                } else {
                    break;
                }
            }
        }

        // exponente
        if matches!(self.peek(0), Some('e' | 'E')) {
            let p1 = self.peek(1);
            let p2 = self.peek(2);
            let valid = p1.map(|c| c.is_ascii_digit()).unwrap_or(false)
                || (matches!(p1, Some('+' | '-'))
                    && p2.map(|c| c.is_ascii_digit()).unwrap_or(false));
            if valid {
                s.push(self.next().unwrap()); // e/E
                if matches!(self.peek(0), Some('+' | '-')) {
                    s.push(self.next().unwrap());
                }
                while let Some(c) = self.peek(0) {
                    if c.is_ascii_digit() {
                        s.push(self.next().unwrap());
                    } else {
                        break;
                    }
                }
            }
        }

        if s.is_empty() {
            return Err(format!(
                "Número inválido en linea {}, col {}",
                start_line, start_col
            ));
        }
        Ok((s, start_line, start_col))
    }

    // para strings con escapes
    fn consume_string(&mut self) -> Result<(String, usize, usize), String> {
        let start_line = self.line;
        let start_col = self.col;
        // consume la comilla inicial
        self.next();

        let mut content = String::new();
        while let Some(c) = self.peek(0) {
            if c == '"' {
                self.next(); // cierra "
                return Ok((content, start_line, start_col));
            }
            let ch = self.next().unwrap();
            if ch == '\\' {
                // escape
                let e = self.next().ok_or_else(|| {
                    format!("Escape incompleto en linea {}, col {}", self.line, self.col)
                })?;
                match e {
                    'n' => content.push('\n'),
                    't' => content.push('\t'),
                    'r' => content.push('\r'),
                    '\\' => content.push('\\'),
                    '"' => content.push('"'),
                    other => content.push(other),
                }
            } else {
                content.push(ch);
            }
        }
        Err(format!("String sin cerrar en linea {}", start_line))
    }

    // intenta consumir operadores de 2 caracteres
    fn try_two_char_op(&mut self) -> Option<Token> {
        let line = self.line;
        let col = self.col;
        match (self.peek(0), self.peek(1)) {
            (Some('<'), Some('=')) => {
                self.next();
                self.next();
                Some(Token {
                    kind: TokenKind::Le,
                    lexeme: "<=".into(),
                    line,
                    col,
                })
            }
            (Some('>'), Some('=')) => {
                self.next();
                self.next();
                Some(Token {
                    kind: TokenKind::Ge,
                    lexeme: ">=".into(),
                    line,
                    col,
                })
            }
            (Some('='), Some('=')) => {
                self.next();
                self.next();
                Some(Token {
                    kind: TokenKind::Eq,
                    lexeme: "==".into(),
                    line,
                    col,
                })
            }
            (Some('!'), Some('=')) => {
                self.next();
                self.next();
                Some(Token {
                    kind: TokenKind::Ne,
                    lexeme: "!=".into(),
                    line,
                    col,
                })
            }
            _ => None,
        }
    }

    // --- navegación del input ---

    // mira el siguiente caracter sin consumirlo
    fn peek(&self, k: usize) -> Option<char> {
        self.chars.get(self.pos + k).copied()
    }

    // consume el siguiente caracter
    fn next(&mut self) -> Option<char> {
        if let Some(c) = self.chars.get(self.pos).copied() {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(c)
        } else {
            None
        }
    }

    // verifica si un caracter es una letra
    #[inline]
    fn is_alpha(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    // verifica si un caracter es un alfanumérico
    #[inline]
    fn is_alnum(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }
}
//------------------------------------- Fin Clase Lexer -------------------------------------//
