use crate::utils::enums::TokenKind;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub col: usize,
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
            KwCapture => "KW_CAPTURE",
            KwSynthesize => "KW_SYNTHESIZE",
            KwChain => "KW_CHAIN",
            KwTo => "KW_TO",
            KwOrbite => "KW_ORBITE",
            KwAtom => "KW_ATOM",
            KwMolecule => "KW_MOLECULE",
            KwReaction => "KW_REACTION",
            KwSymbol => "KW_SYMBOL",
            KwAtomNum => "KW_ATOM_NUM",
            KwMass => "KW_MASS",
            KwPolarized => "KW_POLARIZED",
            KwTrue => "KW_TRUE",
            KwFalse => "KW_FALSE",
            KwVoidState => "KW_VOIDSTATE",
            KwFormula => "KW_FORMULA",
            KwIon => "KW_ION",
            KwSolution => "KW_SOLUTION",
            KwSample => "KW_SAMPLE",
            And => "AND",
            Or => "OR",
            Not => "NOT",
            Ident => "IDENT",
            Number => "NUMBER",
            StringLit => "STRING",
            CharLit => "CHAR",
            Assign => "ASSIGN",
            Plus => "PLUS",
            Minus => "MINUS",
            Star => "STAR",
            Slash => "SLASH",
            Percent => "PERCENT",
            Semi => "SEMI",
            LParen => "LP",
            RParen => "RP",
            LBrace => "LBRACE",
            RBrace => "RBRACE",
            LBracket => "LBRACKET",
            RBracket => "RBRACKET",
            Colon => "COLON",
            Comma => "COMMA",
            Dot => "DOT",
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
