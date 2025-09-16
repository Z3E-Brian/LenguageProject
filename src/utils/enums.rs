#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // keywords/control
    KwITest,
    KwNotest,
    KwInotest,
    KwInhibit,
    KwRelease,
    KwEmitln,
    KwEmit,
    KwSynthesize,
    // decl
    KwAtom,
    KwMolecule,
    KwReaction,
    // types
    KwSymbol,
    KwAtomNum,
    KwMass,
    KwPolarized,
    KwVoidState,
    KwFormula,
    KwIon,
    // logical
    And,
    Or,
    Not,
    // general
    Ident,
    Number,
    StringLit,
    // operators / signs
    Assign,
    Plus,
    Minus,
    Star,
    Slash,
    Semi,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Colon,
    Comma,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    EndOfFile,
}

pub type Block = Vec<Stmt>;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum TypeName { AtomNum, Mass, Polarized, VoidState, Formula, Symbol, Ion, Custom(String) }

#[derive(Debug, Clone)]
pub enum Stmt {
    VarDecl { name: String, ty: TypeName, init: Option<Expr> },
    ConstDecl { name: String, ty: TypeName, value: Expr },
    Assign { name: String, value: Expr },
    If { arms: Vec<(Expr, Block)>, else_block: Option<Block> },
    ExprStmt(Expr),
    EmitLn(Expr),
    Emit(Expr),
    ReactionDecl { name: String, params: Vec<(String, TypeName)>, body: Block },
    Block(Block),
}

#[derive(Debug, Clone)]
pub enum Expr {
    LitNumber(String),
    LitString(String),
    Ident(String),
    Unary { op: TokenKind, rhs: Box<Expr> },
    Binary { lhs: Box<Expr>, op: TokenKind, rhs: Box<Expr> },
}