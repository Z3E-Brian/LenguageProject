use std::collections::HashMap;

use crate::utils::enums::{Expr, Stmt, TypeName, Block, Ty};
use crate::utils::enums::TokenKind;


// ---------------- Símbolos / Ámbitos / Errores ---------------- //
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: Ty,
    pub is_const: bool, // ion
    pub mutable: bool,  // atom = true, ion = false
}

#[derive(Debug)]
pub struct SemError {
    pub msg: String,
    pub line: usize,
    pub col: usize,
}
// ---------------- Simbolos / Ámbitos / Errores ---------------- //


// -- Pila de ambitos (para variables locales y globales) y guardado de errores -- //
type Scope = HashMap<String, Symbol>;

pub struct SemCtx {
    scopes: Vec<Scope>,
    pub errors: Vec<SemError>,
}

impl SemCtx {
    pub fn new() -> Self {
        Self { scopes: vec![Scope::new()], errors: vec![] }
    }
    
    fn push_scope(&mut self) { self.scopes.push(Scope::new()); }
    
    fn pop_scope(&mut self) { self.scopes.pop(); }
    
    fn declare(&mut self, sym: Symbol) {
        let top = self.scopes.last_mut().unwrap();
        if top.contains_key(&sym.name) {
            self.error(format!("Símbolo duplicado: {}", sym.name), 0, 0); // TODO: agregar span al parser
        } else {
            top.insert(sym.name.clone(), sym);
        }
    }
    
    fn lookup(&self, name: &str) -> Option<&Symbol> {
        for s in self.scopes.iter().rev() {
            if let Some(sym) = s.get(name) { return Some(sym); }
        }
        None
    }
    
    fn error<S: Into<String>>(&mut self, msg: S, line: usize, col: usize) {
        self.errors.push(SemError { msg: msg.into(), line, col });
    }
}


// ------------------ Entorno por función (reaction) -----------------
#[derive(Debug, Clone)]
struct FnEnv {ret: Ty}// tipo de retorno esperado

// ---------------------- Pase semántico -----------------------------
pub struct SemanticPass<'a> {
    ctx: &'a mut SemCtx,
    fn_stack: Vec<FnEnv>,
}

impl<'a> SemanticPass<'a> {
    pub fn new(ctx: &'a mut SemCtx) -> Self {
        Self { ctx, fn_stack: vec![] }
    }

    // -- Encargado de crear el contexto y chequear todo el programa -- //
    pub fn check_program(&mut self, program: &Block) {
        for stmt in program {
            if let Stmt::ReactionDecl { name, params, .. } = stmt {
                let param_tys = params.iter().map(|(_, ty)| self.map_typename_to_ty(ty)).collect();
                let ret = Ty::VoidState; // Asumimos void por ahora
                self.ctx.declare(Symbol {
                    name: name.clone(),
                    ty: Ty::Function(param_tys, Box::new(ret)),
                    is_const: true,
                    mutable: false,
                });
            }
        }

        for stmt in program {
            self.check_stmt(stmt);
        }
    }

    // -- Plato principal: chequea cada statement y expresión -- //
    fn check_stmt(&mut self, s: &Stmt) {
        match s {
            // atom x : T = expr?;
            Stmt::VarDecl { name, ty, init, .. } => {
                let init_ty = if let Some(e) = init { 
                    self.check_expr(e) 
                } else { 
                    Ty::Unknown 
                };
                
                let declared_ty = self.map_typename_to_ty(ty);
                
                if let Some(e) = init {
                    if !self.compat(&declared_ty, &init_ty) {
                        self.ctx.error(
                            format!("Tipo incompatible en inicialización de '{}': se esperaba {:?}, obtuviste {:?}", name, declared_ty, init_ty),
                            0, 0 // TODO: agregar span
                        );
                    }
                }
                
                self.ctx.declare(Symbol {
                    name: name.clone(),
                    ty: declared_ty,
                    is_const: false,
                    mutable: true,
                });
            }

            // ion C : T = expr;
            Stmt::ConstDecl { name, ty, value, .. } => {
                let vty = self.check_expr(value);
                let declared_ty = self.map_typename_to_ty(ty);
                
                if !self.compat(&declared_ty, &vty) {
                    self.ctx.error(
                        format!("Const '{}' con tipo incompatible: se esperaba {:?}, obtuviste {:?}", name, declared_ty, vty),
                        0, 0 // TODO: agregar span
                    );
                }
                
                self.ctx.declare(Symbol {
                    name: name.clone(),
                    ty: declared_ty,
                    is_const: true,
                    mutable: false,
                });
            }

            // x = expr;
            Stmt::Assign { name, value, .. } => {
                let vty = self.check_expr(value);
                match self.ctx.lookup(name) {
                    Some(sym) => {
                        if sym.is_const || !sym.mutable {
                            //self.ctx.error(format!("'{}' es constante y no puede reasignarse", name), 0, 0);
                        }
                        if !self.compat(&sym.ty, &vty) {
                            self.ctx.error(
                                format!("Asignación incompatible a '{}': {:?} ← {:?}", name, sym.ty, vty),
                                0, 0 // TODO: agregar span
                            );
                        }
                    }
                    None => self.ctx.error(format!("Símbolo no declarado: '{}'", name), 0, 0),
                }
            }

            // emit/emitln (args...);
            Stmt::EmitLn(args) | Stmt::Emit(args) => {
                // Verificamos que cada argumento sea válido
                // Los argumentos pueden ser de cualquier tipo ya que se convertirán a string
                for arg in args {
                    self.check_expr(arg);
                }
            }

            // itest (cond) { ... } notest { ... }
            Stmt::If { arms, else_block, .. } => {
                for (cond, _) in arms {
                    let cty = self.check_expr(cond);
                    if cty == Ty::Unknown || cty != Ty::Polarized {
                        self.ctx.error("itest requiere condición polarized (bool)", 0, 0);
                    }
                }
                
                self.ctx.push_scope();
                for (_, then_blk) in arms {
                    self.check_block(then_blk);
                }
                self.ctx.pop_scope();
                
                if let Some(else_blk) = else_block {
                    self.ctx.push_scope();
                    self.check_block(else_blk);
                    self.ctx.pop_scope();
                }
            }

            // reaction f(params...) { body }
            Stmt::ReactionDecl { params, body, .. } => {
                self.fn_stack.push(FnEnv { ret: Ty::VoidState });

                self.ctx.push_scope();
                for (pname, pty) in params {
                    let param_ty = self.map_typename_to_ty(pty);
                    self.ctx.declare(Symbol {
                        name: pname.clone(),
                        ty: param_ty,
                        is_const: false,
                        mutable: true,
                    });
                }
                self.check_block(body);
                self.ctx.pop_scope();

                self.fn_stack.pop();
            }

            // { ... }
            Stmt::Block(b) => {
                self.ctx.push_scope();
                self.check_block(b);
                self.ctx.pop_scope();
            }

            // Expresión statement
            Stmt::ExprStmt(expr) => {
                self.check_expr(expr); // Solo verificar la expresión
            }

            // TODO: agregar fors
        }
    }

    fn check_block(&mut self, b: &Block) {
        for s in b {
            self.check_stmt(s);
        }
    }

    fn check_expr(&mut self, e: &Expr) -> Ty {
        match e {
            // Literales
            Expr::LitNumber(s) => {
                if s.contains('.') { Ty::Mass } else { Ty::AtomNum }
            }
            Expr::LitString(_) => Ty::Formula,
            Expr::Ident(name) => {
                match self.ctx.lookup(name) {
                    Some(sym) => sym.ty.clone(),
                    None => { 
                        self.ctx.error(format!("Ident no declarado: '{}'", name), 0, 0); 
                        Ty::Unknown 
                    }
                }
            }

            // Unario
            Expr::Unary { op, rhs } => {
                let rt = self.check_expr(rhs);
                self.unary_type(op, rt)
            }

            // Binario
            Expr::Binary { op, lhs, rhs } => {
                let lt = self.check_expr(lhs);
                let rt = self.check_expr(rhs);
                self.binary_type(op, lt, rt)
            }

            // Si tienes más variantes de Expr, añádelas aquí
            _ => Ty::Unknown,
        }
    }

    // Compatibilidad un poco estricta:
    fn compat(&self, expected: &Ty, got: &Ty) -> bool {
        if expected == got { return true; }
        if matches!(got, Ty::Unknown) { return true; }
        
        // Permisos especiales:
        match (expected, got) {
            (Ty::Mass, Ty::AtomNum) => true, // promoción implícita a float
            _ => false,
        }
    }

    fn unary_type(&mut self, op: &TokenKind, rt: Ty) -> Ty {
        match op {
            TokenKind::Not => {
                if rt == Ty::Polarized { Ty::Polarized }
                else { 
                    self.ctx.error("Operador 'not' requiere polarized (bool)", 0, 0);
                    Ty::Unknown 
                }
            }
            TokenKind::Minus => {
                if rt == Ty::AtomNum || rt == Ty::Mass { rt }
                else { 
                    self.ctx.error("Operador '-' requiere número", 0, 0);
                    Ty::Unknown 
                }
            }
            TokenKind::Plus => {
                if rt == Ty::AtomNum || rt == Ty::Mass { rt }
                else { 
                    self.ctx.error("Operador '+' unario requiere número", 0, 0);
                    Ty::Unknown 
                }
            }
            _ => {
                self.ctx.error("Operador unario no soportado", 0, 0);
                Ty::Unknown
            }
        }
    }

    fn binary_type(&mut self, op: &TokenKind, lt: Ty, rt: Ty) -> Ty {
        use TokenKind::*;
        match op {
            Plus => {
                match (&lt, &rt) {
                    (Ty::Formula, Ty::Formula) => Ty::Formula,                       // "a" + "b"
                    (Ty::AtomNum, Ty::AtomNum) => Ty::AtomNum,                       // 1 + 2
                    (Ty::Mass, Ty::Mass) | (Ty::Mass, Ty::AtomNum) | (Ty::AtomNum, Ty::Mass) => Ty::Mass, // float + int, etc.
                    _ => { 
                        self.ctx.error("Suma con tipos incompatibles", 0, 0); 
                        Ty::Unknown 
                    }
                }
            }
            Minus | Star | Slash => {
                match (&lt, &rt) {
                    (Ty::AtomNum, Ty::AtomNum) => Ty::AtomNum,
                    (Ty::Mass, Ty::Mass) | (Ty::Mass, Ty::AtomNum) | (Ty::AtomNum, Ty::Mass) => Ty::Mass,
                    _ => { 
                        self.ctx.error("Operación aritmética con tipos incompatibles", 0, 0); 
                        Ty::Unknown 
                    }
                }
            }
            And | Or => {
                if lt == Ty::Polarized && rt == Ty::Polarized { Ty::Polarized }
                else { 
                    self.ctx.error("and/or requieren polarized (bool)", 0, 0); 
                    Ty::Unknown 
                }
            }
            Eq | Ne | Lt | Le | Gt | Ge => {
                if self.compat(&lt, &rt) { Ty::Polarized }
                else { 
                    self.ctx.error("Comparación entre tipos incompatibles", 0, 0); 
                    Ty::Unknown 
                }
            }
            _ => {
                self.ctx.error("Operador binario no soportado", 0, 0);
                Ty::Unknown
            }
        }
    }

    fn map_typename_to_ty(&self, tn: &TypeName) -> Ty {
        match tn {
            TypeName::AtomNum => Ty::AtomNum,
            TypeName::Mass => Ty::Mass,
            TypeName::Polarized => Ty::Polarized,
            TypeName::VoidState => Ty::VoidState,
            TypeName::Formula => Ty::Formula,
            TypeName::Symbol => Ty::Formula, // Symbol se trata como string por ahora
            TypeName::Ion => Ty::Mass,       // Ion se trata como float por ahora
            TypeName::Custom(name) => {
                // Para tipos custom, busca en la tabla de símbolos o usa Unknown
                match self.ctx.lookup(name) {
                    Some(sym) => sym.ty.clone(),
                    None => {
                        //self.ctx.error(format!("Tipo custom no definido: {}", name), 0, 0);
                        Ty::Unknown
                    }
                }
            }
        }
    }
}