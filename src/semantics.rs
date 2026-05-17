use std::collections::HashMap;

use crate::utils::enums::TokenKind;
use crate::utils::enums::{Block, Expr, Stmt, Ty, TypeName};

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
        Self {
            scopes: vec![Scope::new()],
            errors: vec![],
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, sym: Symbol) {
        let top = self.scopes.last_mut().unwrap();
        if top.contains_key(&sym.name) {
            self.error(format!("Símbolo duplicado: {}", sym.name), 0, 0);
        } else {
            top.insert(sym.name.clone(), sym);
        }
    }

    fn lookup(&self, name: &str) -> Option<&Symbol> {
        for s in self.scopes.iter().rev() {
            if let Some(sym) = s.get(name) {
                return Some(sym);
            }
        }
        None
    }

    fn error<S: Into<String>>(&mut self, msg: S, line: usize, col: usize) {
        self.errors.push(SemError {
            msg: msg.into(),
            line,
            col,
        });
    }
}

#[derive(Debug, Clone)]
struct FnEnv {
    ret: Ty,
}

// ---------------------- Pase semántico -----------------------------
pub struct SemanticPass<'a> {
    ctx: &'a mut SemCtx,
    fn_stack: Vec<FnEnv>,
    loop_nesting_level: usize,
}

impl<'a> SemanticPass<'a> {
    pub fn new(ctx: &'a mut SemCtx) -> Self {
        Self {
            ctx,
            fn_stack: vec![],
            loop_nesting_level: 0,
        }
    }

    fn get_loop_variable_name(level: usize) -> String {
        format!("loop_{}", level + 1)
    }

    fn enter_loop_scope(&mut self) {
        self.ctx.push_scope();

        for level in 0..=self.loop_nesting_level {
            let loop_var_name = Self::get_loop_variable_name(level);
            self.ctx.declare(Symbol {
                name: loop_var_name,
                ty: Ty::AtomNum,
                is_const: true,
                mutable: false,
            });
        }

        self.loop_nesting_level += 1;
    }

    fn exit_loop_scope(&mut self) {
        self.loop_nesting_level = self.loop_nesting_level.saturating_sub(1);
        self.ctx.pop_scope();
    }

    pub fn check_program(&mut self, program: &Block) {
        for stmt in program {
            if let Stmt::ReactionDecl {
                name,
                params,
                return_type,
                ..
            } = stmt
            {
                let param_tys = params
                    .iter()
                    .map(|(_, ty)| self.map_typename_to_ty(ty))
                    .collect();
                let ret = self.map_typename_to_ty(return_type);
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

    fn check_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Chain { start, end, body } => match (start, end) {
                (Some(n), None) => {
                    if *n < 0 {
                        self.ctx.error("El conteo de chain debe ser ≥ 0", 0, 0);
                        return;
                    }
                    self.enter_loop_scope();
                    self.check_block(body);
                    self.exit_loop_scope();
                }

                (Some(_a), Some(_b)) => {
                    self.enter_loop_scope();
                    self.check_block(body);
                    self.exit_loop_scope();
                }

                _ => {
                    self.ctx
                        .error("Chain inválido: use `chain N {}` o `chain A to B {}`", 0, 0);
                }
            },

            Stmt::Orbite { condition, body } => {
                let cty = self.check_expr(condition);
                if cty != Ty::Polarized && cty != Ty::Unknown {
                    self.ctx.error("orbite requiere condición polarized (bool)", 0, 0);
                }
                self.ctx.push_scope();
                self.check_block(body);
                self.ctx.pop_scope();
            }

            Stmt::VarDecl { name, ty, init, .. } => {
                let init_ty = if let Some(e) = init {
                    self.check_expr(e)
                } else {
                    Ty::Unknown
                };

                let declared_ty = self.map_typename_to_ty(ty);

                if init.is_some() {
                    if !self.compat(&declared_ty, &init_ty) {
                        self.ctx.error(
                            format!("Tipo incompatible en inicialización de '{}': se esperaba {:?}, obtuviste {:?}", name, declared_ty, init_ty),
                            0, 0
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
            Stmt::ConstDecl {
                name, ty, value, ..
            } => {
                let vty = self.check_expr(value);
                let declared_ty = self.map_typename_to_ty(ty);

                if !self.compat(&declared_ty, &vty) {
                    self.ctx.error(
                        format!(
                            "Const '{}' con tipo incompatible: se esperaba {:?}, obtuviste {:?}",
                            name, declared_ty, vty
                        ),
                        0,
                        0,
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
                let sym_opt = self.ctx.lookup(name).cloned();
                match sym_opt {
                    Some(sym) => {
                        if sym.is_const || !sym.mutable {
                            self.ctx.error(
                                format!("'{}' es constante y no puede reasignarse", name),
                                0,
                                0,
                            );
                        }
                        if !self.compat(&sym.ty, &vty) {
                            self.ctx.error(
                                format!(
                                    "Asignación incompatible a '{}': {:?} ← {:?}",
                                    name, sym.ty, vty
                                ),
                                0,
                                0,
                            );
                        }
                    }
                    None => self
                        .ctx
                        .error(format!("Símbolo no declarado: '{}'", name), 0, 0),
                }
            }

            // emit/emitln
            Stmt::EmitLn(args) | Stmt::Emit(args) => {
                for arg in args {
                    self.check_expr(arg);
                }
            }

            // capture
            Stmt::Capture { var_name } => {
                if self.ctx.lookup(var_name).is_none() {
                    self.ctx.error(
                        format!("Variable '{}' no declarada para capture", var_name),
                        0,
                        0,
                    );
                }
            }

            // itest (cond) { ... } notest { ... }
            Stmt::If {
                arms, else_block, ..
            } => {
                for (cond, _) in arms {
                    let cty = self.check_expr(cond);
                    if cty != Ty::Polarized && cty != Ty::Unknown {
                        self.ctx
                            .error("itest requiere condición polarized (bool)", 0, 0);
                    }
                }

                for (_, then_blk) in arms {
                    self.ctx.push_scope();
                    self.check_block(then_blk);
                    self.ctx.pop_scope();
                }

                if let Some(else_blk) = else_block {
                    self.ctx.push_scope();
                    self.check_block(else_blk);
                    self.ctx.pop_scope();
                }
            }

            // reaction
            Stmt::ReactionDecl {
                params,
                return_type,
                body,
                ..
            } => {
                let ret_ty = self.map_typename_to_ty(return_type);
                self.fn_stack.push(FnEnv {
                    ret: ret_ty.clone(),
                });

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

            // release expr;
            Stmt::Release(expr) => {
                let expr_ty = self.check_expr(expr);

                if let Some(fn_env) = self.fn_stack.last() {
                    if !self.compat(&fn_env.ret, &expr_ty) {
                        self.ctx.error(
                            format!(
                                "Tipo de retorno incompatible: se esperaba {:?}, obtuviste {:?}",
                                fn_env.ret, expr_ty
                            ),
                            0,
                            0,
                        );
                    }
                } else {
                    self.ctx.error(
                        "'release' solo puede usarse dentro de una función".to_string(),
                        0,
                        0,
                    );
                }
            }

            // { ... }
            Stmt::Block(b) => {
                self.ctx.push_scope();
                self.check_block(b);
                self.ctx.pop_scope();
            }

            Stmt::ExprStmt(expr) => {
                self.check_expr(expr); // Solo verificar la expresión
            }
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
                if s.contains('.') {
                    Ty::Mass
                } else {
                    Ty::AtomNum
                }
            }
            Expr::LitString(_) => Ty::Formula,
            Expr::LitTrue | Expr::LitFalse => Ty::Polarized,
            Expr::LitChar(_) => Ty::Symbol,
            Expr::Ident(name) => match self.ctx.lookup(name) {
                Some(sym) => sym.ty.clone(),
                None => {
                    self.ctx
                        .error(format!("Ident no declarado: '{}'", name), 0, 0);
                    Ty::Unknown
                }
            },

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

            Expr::VecLiteral(items) => self.type_of_vec_literal(items),

            Expr::ListLiteral(items) => self.type_of_list_literal(items),

            Expr::Index { target, index } => self.type_of_index(target, index),

            Expr::MethodCall {
                receiver,
                name,
                args,
            } => self.type_of_solution_method(receiver, name, args),

            Expr::FunctionCall { name, args } => match self.ctx.lookup(name) {
                Some(sym) => {
                    if let Ty::Function(param_tys, ret_ty) = sym.ty.clone() {
                        if args.len() != param_tys.len() {
                            self.ctx.error(
                                format!(
                                    "Función '{}' espera {} argumentos, recibió {}",
                                    name,
                                    param_tys.len(),
                                    args.len()
                                ),
                                0,
                                0,
                            );
                            return Ty::Unknown;
                        }

                        for (i, (arg, expected_ty)) in args.iter().zip(param_tys.iter()).enumerate()
                        {
                            let arg_ty = self.check_expr(arg);
                            if !self.compat(&expected_ty, &arg_ty) {
                                self.ctx.error(
                                        format!("Argumento {} de '{}': tipo incompatible (esperado {:?}, recibido {:?})", 
                                            i + 1, name, expected_ty, arg_ty),
                                        0, 0
                                    );
                            }
                        }

                        (*ret_ty).clone()
                    } else {
                        self.ctx
                            .error(format!("'{}' no es una función", name), 0, 0);
                        Ty::Unknown
                    }
                }
                None => {
                    self.ctx
                        .error(format!("Función '{}' no declarada", name), 0, 0);
                    Ty::Unknown
                }
            },
        }
    }

    fn compat(&self, expected: &Ty, got: &Ty) -> bool {
        if expected == got {
            return true;
        }
        if matches!(got, Ty::Unknown) {
            return true;
        }

        match (expected, got) {
            (Ty::Mass, Ty::AtomNum) => true,
            (Ty::Solution(te), Ty::Solution(tg)) => self.compat(te, tg),
            (Ty::Sample(te), Ty::Sample(tg)) => self.compat(te, tg),
            (Ty::Sample(te), Ty::Solution(tg)) => self.compat(te, tg),

            _ => false,
        }
    }

    fn unary_type(&mut self, op: &TokenKind, rt: Ty) -> Ty {
        match op {
            TokenKind::Not => {
                if rt == Ty::Polarized {
                    Ty::Polarized
                } else {
                    self.ctx
                        .error("Operador 'not' requiere polarized (bool)", 0, 0);
                    Ty::Unknown
                }
            }
            TokenKind::Minus => {
                if rt == Ty::AtomNum || rt == Ty::Mass {
                    rt
                } else {
                    self.ctx.error("Operador '-' requiere número", 0, 0);
                    Ty::Unknown
                }
            }
            TokenKind::Plus => {
                if rt == Ty::AtomNum || rt == Ty::Mass {
                    rt
                } else {
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
            Plus => match (&lt, &rt) {
                (Ty::Formula, Ty::Formula) => Ty::Formula,
                (Ty::AtomNum, Ty::AtomNum) => Ty::AtomNum,
                (Ty::Mass, Ty::Mass) | (Ty::Mass, Ty::AtomNum) | (Ty::AtomNum, Ty::Mass) => {
                    Ty::Mass
                }
                _ => {
                    self.ctx.error("Suma con tipos incompatibles", 0, 0);
                    Ty::Unknown
                }
            },
            Minus | Star | Slash | Percent => match (&lt, &rt) {
                (Ty::AtomNum, Ty::AtomNum) => Ty::AtomNum,
                (Ty::Mass, Ty::Mass) | (Ty::Mass, Ty::AtomNum) | (Ty::AtomNum, Ty::Mass) => {
                    Ty::Mass
                }
                _ => {
                    self.ctx
                        .error("Operación aritmética con tipos incompatibles", 0, 0);
                    Ty::Unknown
                }
            },
            And | Or => {
                if lt == Ty::Polarized && rt == Ty::Polarized {
                    Ty::Polarized
                } else {
                    self.ctx.error("and/or requieren polarized (bool)", 0, 0);
                    Ty::Unknown
                }
            }
            Eq | Ne | Lt | Le | Gt | Ge => {
                if self.compat(&lt, &rt) {
                    Ty::Polarized
                } else {
                    self.ctx
                        .error("Comparación entre tipos incompatibles", 0, 0);
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
            TypeName::Symbol => Ty::Symbol,
            TypeName::Ion => Ty::Unknown,
            TypeName::Custom(name) => match self.ctx.lookup(name) {
                Some(sym) => sym.ty.clone(),
                None => Ty::Unknown,
            },
            TypeName::Solution(inner) => Ty::Solution(Box::new(self.map_typename_to_ty(inner))),
            TypeName::Sample(inner) => Ty::Sample(Box::new(self.map_typename_to_ty(inner))),
        }
    }

    fn type_of_vec_literal(&mut self, items: &[Expr]) -> Ty {
        if items.is_empty() {
            self.ctx.error(
                "No se puede inferir el tipo de [ ] vacío; anote con solution<T>",
                0,
                0,
            );
            return Ty::Unknown;
        }
        let first = self.check_expr(&items[0]);
        if matches!(first, Ty::Unknown) {
            return Ty::Unknown;
        }
        for e in &items[1..] {
            let t = self.check_expr(e);
            if !self.compat(&first, &t) || !self.compat(&t, &first) {
                self.ctx
                    .error("Los literales de vector deben ser homogéneos", 0, 0);
                return Ty::Unknown;
            }
        }
        Ty::Solution(Box::new(first))
    }

    fn type_of_list_literal(&mut self, items: &[Expr]) -> Ty {
        if items.is_empty() {
            self.ctx.error(
                "No se puede inferir el tipo de [ ] vacío; anote con sample<T>",
                0,
                0,
            );
            return Ty::Unknown;
        }
        let first = self.check_expr(&items[0]);
        if matches!(first, Ty::Unknown) {
            return Ty::Unknown;
        }
        for e in &items[1..] {
            let t = self.check_expr(e);
            if !self.compat(&first, &t) || !self.compat(&t, &first) {
                self.ctx
                    .error("Los literales de lista deben ser homogéneos", 0, 0);
                return Ty::Unknown;
            }
        }
        Ty::Sample(Box::new(first))
    }

    fn type_of_index(&mut self, target: &Expr, index: &Expr) -> Ty {
        let tt = self.check_expr(target);
        let ti = self.check_expr(index);
        if ti != Ty::AtomNum {
            self.ctx
                .error("El índice debe ser atom_num (entero ≥ 0)", 0, 0);
            return Ty::Unknown;
        }
        match tt {
            Ty::Solution(inner) => *inner,
            Ty::Sample(inner) => *inner,
            _ => {
                self.ctx
                    .error("La indexación solo aplica a solution<T> o sample<T>", 0, 0);
                Ty::Unknown
            }
        }
    }

    fn type_of_solution_method(&mut self, recv: &Expr, name: &str, args: &[Expr]) -> Ty {
        let rt = self.check_expr(recv);
        match rt {
            Ty::Solution(inner) => match name {
                "len" => {
                    if !args.is_empty() {
                        self.ctx.error("len() espera 0 argumentos", 0, 0);
                    }
                    Ty::AtomNum
                }
                "push" => {
                    if args.len() != 1 {
                        self.ctx.error("push(x) espera 1 argumento", 0, 0);
                        return Ty::Unknown;
                    }
                    let at = self.check_expr(&args[0]);
                    if !self.compat(&inner, &at) || !self.compat(&at, &inner) {
                        self.ctx
                            .error("push(x): el tipo de x no coincide con T", 0, 0);
                        return Ty::Unknown;
                    }
                    Ty::VoidState
                }
                "set" => {
                    if args.len() != 2 {
                        self.ctx.error("set(i, x) espera 2 argumentos", 0, 0);
                        return Ty::Unknown;
                    }
                    let i = self.check_expr(&args[0]);
                    let x = self.check_expr(&args[1]);
                    if i != Ty::AtomNum {
                        self.ctx.error("set(i, x): i debe ser atom_num", 0, 0);
                        return Ty::Unknown;
                    }
                    if !self.compat(&inner, &x) || !self.compat(&x, &inner) {
                        self.ctx.error("set(i, x): x no coincide con T", 0, 0);
                        return Ty::Unknown;
                    }
                    Ty::VoidState
                }
                "pop" => {
                    if !args.is_empty() {
                        self.ctx.error("pop() espera 0 argumentos", 0, 0);
                    }
                    *inner
                }
                _ => {
                    self.ctx.error("Método desconocido para solution<T>", 0, 0);
                    Ty::Unknown
                }
            },
            Ty::Sample(inner) => match name {
                "len" => {
                    if !args.is_empty() {
                        self.ctx.error("len() espera 0 argumentos", 0, 0);
                    }
                    Ty::AtomNum
                }
                "push_front" | "push_back" => {
                    if args.len() != 1 {
                        self.ctx
                            .error(&format!("{}(x) espera 1 argumento", name), 0, 0);
                        return Ty::Unknown;
                    }
                    let at = self.check_expr(&args[0]);
                    if !self.compat(&inner, &at) || !self.compat(&at, &inner) {
                        self.ctx.error(
                            &format!("{}(x): el tipo de x no coincide con T", name),
                            0,
                            0,
                        );
                        return Ty::Unknown;
                    }
                    Ty::VoidState
                }
                "pop_front" | "pop_back" => {
                    if !args.is_empty() {
                        self.ctx
                            .error(&format!("{}() espera 0 argumentos", name), 0, 0);
                    }
                    *inner
                }
                "insert" => {
                    if args.len() != 2 {
                        self.ctx.error("insert(i, x) espera 2 argumentos", 0, 0);
                        return Ty::Unknown;
                    }
                    let i = self.check_expr(&args[0]);
                    let x = self.check_expr(&args[1]);
                    if i != Ty::AtomNum {
                        self.ctx.error("insert(i, x): i debe ser atom_num", 0, 0);
                        return Ty::Unknown;
                    }
                    if !self.compat(&inner, &x) || !self.compat(&x, &inner) {
                        self.ctx.error("insert(i, x): x no coincide con T", 0, 0);
                        return Ty::Unknown;
                    }
                    Ty::VoidState
                }
                "remove" => {
                    if args.len() != 1 {
                        self.ctx.error("remove(i) espera 1 argumento", 0, 0);
                        return Ty::Unknown;
                    }
                    let i = self.check_expr(&args[0]);
                    if i != Ty::AtomNum {
                        self.ctx.error("remove(i): i debe ser atom_num", 0, 0);
                        return Ty::Unknown;
                    }
                    *inner
                }
                _ => {
                    self.ctx.error("Método desconocido para sample<T>", 0, 0);
                    Ty::Unknown
                }
            },
            _ => {
                self.ctx.error(
                    "Llamada de método: receptor no es solution<T> ni sample<T>",
                    0,
                    0,
                );
                Ty::Unknown
            }
        }
    }
}
