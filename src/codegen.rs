use crate::utils::enums::{Expr, Stmt, TokenKind, Value, Instruction};
use crate::parser::Program;
use std::collections::HashMap;

// ===================== INSTRUCCIONES DEL CODIGO INTERMEDIO =====================

// ===================== GENERADOR DE CODIGO =====================

pub struct CodeGenerator {
    instructions: Vec<Instruction>,
    label_counter: usize,
    function_table: HashMap<String, usize>, // nombre -> posición en instrucciones
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            label_counter: 0,
            function_table: HashMap::new(),
        }
    }

    fn emit_call_builtin<S: Into<String>>(&mut self, name: S, argc: usize) {
        self.emit(Instruction::Call(name.into(), argc));
    }
    
    // Función principal: generar código para todo el programa
    pub fn generate(&mut self, program: &Program) -> Result<Vec<Instruction>, String> {
        self.instructions.clear();
        
        // Primer pase: registrar todas las funciones
        for stmt in program {
            if let Stmt::ReactionDecl { name, .. } = stmt {
                // Reservar posición para la función (se llenará después)
                self.function_table.insert(name.clone(), self.instructions.len());
            }
        }
        
        // Segundo pase: generar código
        for stmt in program {
            self.generate_stmt(stmt)?;
        }
        
        Ok(self.instructions.clone())
    }
    
    // ===== GENERACION DE STATEMENTS =====
    
    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::VarDecl { name, ty: _, init } => {
                if let Some(expr) = init {
                    self.generate_expr(expr)?;
                    self.emit(Instruction::StoreVar(name.clone()));
                }
                Ok(())
            }
            
            Stmt::ConstDecl { name, ty: _, value } => {
                self.generate_expr(value)?;
                self.emit(Instruction::StoreVar(name.clone()));
                Ok(())
            }
            
            Stmt::Assign { name, value } => {
                self.generate_expr(value)?;
                self.emit(Instruction::StoreVar(name.clone()));
                Ok(())
            }
            
            Stmt::EmitLn(args) => {
                // Generar código para cada argumento
                for arg in args {
                    self.generate_expr(arg)?;
                }
                self.emit(Instruction::EmitLn(args.len()));
                Ok(())
            }
            
            Stmt::Emit(args) => {
                // Generar código para cada argumento
                for arg in args {
                    self.generate_expr(arg)?;
                }
                self.emit(Instruction::Emit(args.len()));
                Ok(())
            }
            
            Stmt::If { arms, else_block } => {
                self.generate_if(arms, else_block)
            }
            
            Stmt::ReactionDecl { name, params, body } => {
                self.generate_function(name, params, body)
            }
            
            Stmt::Block(stmts) => {
                self.emit(Instruction::PushScope);
                for stmt in stmts {
                    self.generate_stmt(stmt)?;
                }
                self.emit(Instruction::PopScope);
                Ok(())
            }
            
            Stmt::ExprStmt(expr) => {
                self.generate_expr(expr)?;
                self.emit(Instruction::Pop); // Descartar resultado
                Ok(())
            }
            
            Stmt::Chain { start, end, body } => {
                self.generate_chain(start, end, body)
            }
        }
    }
    
    // ===== GENERACION DE EXPRESIONES =====
    
    fn generate_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::LitNumber(num_str) => {
                let value = num_str.parse::<f64>()
                    .map_err(|_| format!("Invalid number: {}", num_str))?;
                self.emit(Instruction::LoadConst(Value::Number(value)));
                Ok(())
            }
            
            Expr::LitString(s) => {
                self.emit(Instruction::LoadConst(Value::String(s.clone())));
                Ok(())
            }
            
            Expr::Ident(name) => {
                self.emit(Instruction::LoadVar(name.clone()));
                Ok(())
            }
            
            Expr::Binary { lhs, op, rhs } => {
                // Generar código para operandos
                self.generate_expr(lhs)?;
                self.generate_expr(rhs)?;
                
                // Generar instrucción de operación
                let instruction = match op {
                    TokenKind::Plus => Instruction::Add,
                    TokenKind::Minus => Instruction::Sub,
                    TokenKind::Star => Instruction::Mul,
                    TokenKind::Slash => Instruction::Div,
                    TokenKind::Eq => Instruction::Equal,
                    TokenKind::Ne => Instruction::NotEqual,
                    TokenKind::Lt => Instruction::Less,
                    TokenKind::Le => Instruction::LessEqual,
                    TokenKind::Gt => Instruction::Greater,
                    TokenKind::Ge => Instruction::GreaterEqual,
                    TokenKind::And => Instruction::And,
                    TokenKind::Or => Instruction::Or,
                    _ => return Err(format!("Unsupported binary operator: {:?}", op)),
                };
                
                self.emit(instruction);
                Ok(())
            }
            
            Expr::Unary { op, rhs } => {
                self.generate_expr(rhs)?;
                
                let instruction = match op {
                    TokenKind::Minus => Instruction::Neg,
                    TokenKind::Not => Instruction::Not,
                    _ => return Err(format!("Unsupported unary operator: {:?}", op)),
                };
                
                self.emit(instruction);
                Ok(())
            }
            Expr::VecLiteral(items) => {
                // Empuja cada elemento en orden de evaluación
                for it in items {
                    self.generate_expr(it)?;
                }
                // Construye el vector con N elementos del stack
                self.emit_call_builtin("__vec_from", items.len());
                Ok(())
            }

            Expr::Index { target, index } => {
                // Stack: target, index  → __vec_get
                self.generate_expr(target)?;
                self.generate_expr(index)?;
                self.emit_call_builtin("__vec_get", 2);
                Ok(())
            }

            Expr::MethodCall { receiver, name, args } => {
                // Empuja receptor y argumentos
                self.generate_expr(receiver)?;
                for a in args {
                    self.generate_expr(a)?;
                }
                let argc = 1 + args.len(); // receiver + args
                match name.as_str() {
                    "len"  => self.emit_call_builtin("__vec_len", argc),
                    "push" => self.emit_call_builtin("__vec_push", argc),
                    "set"  => self.emit_call_builtin("__vec_set", argc),
                    _ => return Err(format!("Método no soportado en solution<T>: {}", name)),
                }
                Ok(())
            }
        }
    }
    
    // ===== GENERACION DE ESTRUCTURAS DE CONTROL =====
    
    fn generate_if(&mut self, arms: &[(Expr, Vec<Stmt>)], else_block: &Option<Vec<Stmt>>) -> Result<(), String> {
        let mut jump_to_end_labels = Vec::new();
        
        for (i, (condition, then_block)) in arms.iter().enumerate() {
            // Generar condición
            self.generate_expr(condition)?;
            
            // Si es falsa, saltar al siguiente brazo o al else
            let next_label = self.create_label();
            self.emit(Instruction::JumpIfFalse(0)); // Se actualizará después
            let jump_pos = self.instructions.len() - 1;
            
            // Generar código del bloque then
            for stmt in then_block {
                self.generate_stmt(stmt)?;
            }
            
            // Saltar al final (evitar ejecutar otros brazos)
            let end_label = self.create_label();
            self.emit(Instruction::Jump(0)); // Se actualizará después
            jump_to_end_labels.push(self.instructions.len() - 1);
            
            // Actualizar el salto condicional para apuntar aquí
            let current_pos = self.instructions.len();
            if let Instruction::JumpIfFalse(target) = &mut self.instructions[jump_pos] {
                *target = current_pos;
            }
        }
        
        // Generar bloque else si existe
        if let Some(else_stmts) = else_block {
            for stmt in else_stmts {
                self.generate_stmt(stmt)?;
            }
        }
        
        // Actualizar todos los saltos al final
        let end_pos = self.instructions.len();
        for jump_pos in jump_to_end_labels {
            if let Instruction::Jump(target) = &mut self.instructions[jump_pos] {
                *target = end_pos;
            }
        }
        
        Ok(())
    }
    
    fn generate_chain(&mut self, start: &Option<i32>, end: &Option<i32>, body: &[Stmt]) -> Result<(), String> {
        match (start, end) {
            // chain N {} - ejecuta N veces
            (Some(count), None) => {
                if *count <= 0 {
                    return Ok(()); // No ejecutar si count <= 0
                }
                
                // Iniciar captura del bucle: de 0 a count-1
                self.emit(Instruction::StartLoopCapture(0, *count - 1, true));
                
                // Generar código del cuerpo (se capturará en el buffer)
                for stmt in body {
                    self.generate_stmt(stmt)?;
                }
                
                // Terminar captura y empezar ejecución cíclica
                self.emit(Instruction::EndLoopCapture);
            }
            
            // chain N to M {} - ejecuta desde N hasta M 
            (Some(start_val), Some(end_val)) => {
                // Determinar si es ascendente o descendente
                let ascending = *start_val < *end_val;
                
                // 🆕 Iniciar captura del bucle
                self.emit(Instruction::StartLoopCapture(*start_val, *end_val, ascending));
                
                // Generar código del cuerpo
                for stmt in body {
                    self.generate_stmt(stmt)?;
                }
                
                // Terminar captura y empezar ejecución cíclica
                self.emit(Instruction::EndLoopCapture);
            }
            
            _ => {
                return Err("Chain inválido: debe especificar al menos un valor inicial".to_string());
            }
        }
        
        Ok(())
    }
    
    fn generate_function(&mut self, name: &str, params: &[(String, crate::utils::enums::TypeName)], body: &[Stmt]) -> Result<(), String> {
        // Marcar inicio de función
        let function_start = self.instructions.len();
        self.function_table.insert(name.to_string(), function_start);
        
        // Crear scope para parámetros
        self.emit(Instruction::PushScope);
        
        // Los parámetros ya estarán en el stack cuando se llame la función
        // Solo necesitamos almacenarlos en variables locales (en orden inverso)
        for (param_name, _) in params.iter().rev() {
            self.emit(Instruction::StoreVar(param_name.clone()));
        }
        
        // Generar código del cuerpo
        for stmt in body {
            self.generate_stmt(stmt)?;
        }
        
        // Si no hay return explícito, retornar void
        self.emit(Instruction::LoadConst(Value::Void));
        self.emit(Instruction::Return);
        self.emit(Instruction::PopScope);
        
        Ok(())
    }
    
    // ===== UTILIDADES =====
    
    fn emit(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }
    
    fn create_label(&mut self) -> String {
        let label = format!("L{}", self.label_counter);
        self.label_counter += 1;
        label
    }
    
    // Función pública para obtener las instrucciones generadas
    pub fn get_instructions(&self) -> &[Instruction] {
        &self.instructions
    }
    
    // Función para debugging: mostrar instrucciones generadas
    pub fn print_instructions(&self) {
        println!("=== CÓDIGO INTERMEDIO GENERADO ===");
        for (i, instruction) in self.instructions.iter().enumerate() {
            println!("{:3}: {:?}", i, instruction);
        }
        println!("==================================");
    }
}

// ===================== FUNCIONES DE UTILIDAD =====================

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Char(c) => write!(f, "{}", c),
            Value::Void => write!(f, "void"),
            Value::Vector { data, .. } => {
                write!(f, "[")?;
                for (i, v) in data.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", v)?; // usa Display de Value para cada elemento
                }
                write!(f, "]")
            },
        }
    }
}

impl Value {
    // Conversión a booleano para condicionales
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Char(_) => true,
            Value::Void => false,
            Value::Vector { data, .. } => !data.is_empty(),
        }
    }
    
    // Conversión a string para output
    pub fn to_string(&self) -> String {
        match self {
            Value::Number(n) => {
                // Si es entero, mostrar sin decimales
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::String(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::Char(c) => c.to_string(),
            Value::Void => "void".to_string(),
            Value::Vector { data, .. } => {
                let mut s = String::from("[");
                for (i, v) in data.iter().enumerate() {
                    if i > 0 { s.push_str(", "); }
                    s.push_str(&v.to_string());
                }
                s.push(']');
                s
            },
        }
    }
}