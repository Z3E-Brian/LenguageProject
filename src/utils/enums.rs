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
    KwCapture,
    KwSynthesize,
    KwChain,
    KwTo,
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
    KwSolution,
    KwSample,
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
    Dot,
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
pub enum TypeName { AtomNum, Mass, Polarized, VoidState, Formula, Symbol, Ion, Custom(String),Solution(Box<TypeName>),Sample(Box<TypeName>),}

#[derive(Debug, Clone)]
pub enum Stmt {
    VarDecl { name: String, ty: TypeName, init: Option<Expr> },
    ConstDecl { name: String, ty: TypeName, value: Expr },
    Assign { name: String, value: Expr },
    If { arms: Vec<(Expr, Block)>, else_block: Option<Block> },
    ExprStmt(Expr),
    EmitLn(Vec<Expr>),  // Cambiado para aceptar múltiples argumentos
    Emit(Vec<Expr>),    // Cambiado para aceptar múltiples argumentos
    Capture { var_name: String },  // Capturar entrada del usuario
    ReactionDecl { name: String, params: Vec<(String, TypeName)>, body: Block },
    Block(Block),
    Chain { start: Option<i32>, end: Option<i32>, body: Block },
}

#[derive(Debug, Clone)]
pub enum Expr {
    LitNumber(String),
    LitString(String),
    Ident(String),
    Unary { op: TokenKind, rhs: Box<Expr> },
    Binary { lhs: Box<Expr>, op: TokenKind, rhs: Box<Expr> },
    VecLiteral(Vec<Expr>),
    ListLiteral(Vec<Expr>),
    Index { target: Box<Expr>, index: Box<Expr> },
    MethodCall { receiver: Box<Expr>, name: String, args: Vec<Expr> },
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add, Sub, Mul, Div, And, Or, Eq, Ne, Lt, Le, Gt, Ge, Concat
}

#[derive(Debug, Clone)]
pub enum UnOp {
    Not, Neg
}


#[derive(Debug, Clone)]
pub enum Instruction {
    // Manejo de valores
    LoadConst(Value),           // Cargar constante al stack
    LoadVar(String),            // Cargar variable al stack  
    StoreVar(String),           // Guardar del stack a variable
    
    // Operaciones aritméticas
    Add, Sub, Mul, Div, Mod,    // Operaciones binarias
    Neg, Not,                   // Operaciones unarias
    
    // Operaciones de comparación
    Equal, NotEqual,            // == !=
    Less, Greater,              // < >
    LessEqual, GreaterEqual,    // <= >=
    And, Or,                    // && ||
    
    // Control de flujo
    Jump(usize),                // Salto incondicional
    JumpIfFalse(usize),         // Salto condicional
    JumpIfTrue(usize),          // Salto condicional
    Label(String),              // Etiqueta para saltos
    
    // Funciones
    Call(String, usize),        // Llamar función (nombre, num_args)
    Return,                     // Retornar de función
    PushScope,                  // Crear nuevo scope
    PopScope,                   // Eliminar scope actual
    
    // I/O
    EmitLn(usize),              // Imprimir con salto (num_args)
    Emit(usize),                // Imprimir sin salto (num_args)
    Capture(String),            // Capturar entrada del usuario (nombre de variable)
    RegisterVarType(String, Ty), // Registrar tipo de variable (nombre, tipo)
    
    // Control de bucles (para futuro)
    Break,                      // Salir del bucle
    Continue,                   // Siguiente iteración
    
    // 🆕 INSTRUCCIONES PARA STACK AUXILIAR DE BUCLES
    StartLoopCapture(i32, i32, bool), // (inicial, límite, ascendente) - Empezar captura
    EndLoopCapture,             // Terminar captura y empezar ejecución cíclica
    
    // Utilidades
    Pop,                        // Eliminar valor del stack
    Dup,                        // Duplicar valor en stack
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),                // atom_num, mass
    String(String),             // formula
    Bool(bool),                 // polarized
    Char(char),                 // symbol (futuro)
    Void,                       // VoidState
    Vector {
        elem: Ty,        // tipo de los elementos
        data: Vec<Value> // datos
    },
    List {
        elem: Ty,        // tipo de los elementos
        nodes: Vec<Value> // nodos enlazados (simulados con Vec para simplicidad)
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    AtomNum,    // int
    Mass,       // float
    Polarized,  // bool
    Formula,    // string
    VoidState,  // void/null
    Unknown,    // tipo desconocido (para no abortar al 1er error)
    Function(Vec<Ty>, Box<Ty>), // params, retorno
    Solution(Box<Ty>),
    Sample(Box<Ty>),
}
