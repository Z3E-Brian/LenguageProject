use crate::utils::enums::TokenKind;
use std::fmt;

#[derive(Debug, Clone)]

pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub col: usize,
}

/* implementacion de getters
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
}*/

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