use crate::utils::enums::{Instruction, Value, Ty};
use std::collections::HashMap;

// ===================== ESTADO DE EJECUCIÓN =====================
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStatus {
    Finished,
    WaitingForInput(String, Ty), // nombre de la variable y su tipo esperado
}

// ===================== CALLBACK PARA CAPTURA DE ENTRADA =====================
pub type InputCallback = Box<dyn Fn(&str) -> Option<String>>;

// ===================== MAQUINA VIRTUAL / EJECUTOR =====================

pub struct Executor {
    stack: Vec<Value>,                      // Pila de valores
    variables: Vec<HashMap<String, Value>>, // Stack de scopes de variables
    variable_types: HashMap<String, Ty>,    // Tipos de las variables declaradas
    output: String,                         // Salida del programa
    pc: usize,                              // Program counter (instruction pointer)
    call_stack: Vec<usize>,                 // Pila de llamadas (para funciones)
    function_table: HashMap<String, usize>, // Tabla de funciones (nombre -> posición)
    loop_counter_stack: Vec<i32>,               // Stack de contadores [exterior -> interior]
    pub input_callback: Option<InputCallback>,  // Callback para entrada del usuario
    paused_for_input: bool,                 // Flag para indicar pausa por input
}

#[derive(Debug)]
pub struct ExecutionError {
    pub message: String,
    pub instruction_index: usize,
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Runtime Error at instruction {}: {}",
            self.instruction_index, self.message
        )
    }
}

impl std::error::Error for ExecutionError {}

impl Executor {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            variables: vec![HashMap::new()], // scope global
            variable_types: HashMap::new(),
            output: String::new(),
            pc: 0,
            call_stack: Vec::new(),
            function_table: HashMap::new(),
            loop_counter_stack: Vec::new(),
            input_callback: None,
            paused_for_input: false,
        }
    }
    
    pub fn register_function(&mut self, name: String, address: usize) {
        self.function_table.insert(name, address);
    }

    pub fn register_variable_type(&mut self, name: &str, ty: Ty) {
        self.variable_types.insert(name.to_string(), ty);
    }

    fn get_variable_type(&self, name: &str) -> Ty {
        self.variable_types.get(name).cloned().unwrap_or(Ty::Unknown)
    }

    pub fn get_output(&self) -> &str {
        &self.output
    }

    pub fn store_captured_value(&mut self, var_name: &str, value: Value) {
        self.output.push_str(&format!("{}\n", value.to_string()));
        
        for scope in self.variables.iter_mut().rev() {
            if scope.contains_key(var_name) {
                scope.insert(var_name.to_string(), value);
                return;
            }
        }
        if let Some(current_scope) = self.variables.last_mut() {
            current_scope.insert(var_name.to_string(), value);
        }
    }

    pub fn execute_until_capture(&mut self, instructions: &[Instruction]) -> Result<ExecutionStatus, ExecutionError> {
        self.paused_for_input = false;
        
        while self.pc < instructions.len() {
            let current_instruction = &instructions[self.pc];
            
            if let Instruction::Capture(var_name) = current_instruction {
                self.output.push_str(&format!(">> Ingrese valor para '{}': ", var_name));
                self.paused_for_input = true;
                self.pc += 1;
                let var_type = self.get_variable_type(var_name);
                return Ok(ExecutionStatus::WaitingForInput(var_name.clone(), var_type));
            }
            
            match current_instruction {
                Instruction::StartLoopCapture(initial, limit, ascending) => {
                    self.execute_loop_immediately_with_instructions(*initial, *limit, *ascending, instructions)?;
                }
                _ => {
                    match self.execute_instruction(current_instruction) {
                        Ok(should_increment_pc) => {
                            if should_increment_pc {
                                self.pc += 1;
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
        }
        
        Ok(ExecutionStatus::Finished)
    }

    pub fn execute(&mut self, instructions: &[Instruction]) -> Result<String, ExecutionError> {
        self.pc = 0;
        self.output.clear();
        self.execute_instructions(instructions)?;
        Ok(self.output.clone())
    }

    fn execute_instructions(&mut self, instructions: &[Instruction]) -> Result<(), ExecutionError> {
        while self.pc < instructions.len() {
            let current_instruction = &instructions[self.pc];
            
            match current_instruction {
                Instruction::StartLoopCapture(initial, limit, ascending) => {
                    self.execute_loop_immediately_with_instructions(*initial, *limit, *ascending, instructions)?;
                }
                _ => {
                    match self.execute_instruction(current_instruction) {
                        Ok(should_increment_pc) => {
                            if should_increment_pc {
                                self.pc += 1;
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
        }
        Ok(())
    }

    fn execute_instruction(&mut self, instruction: &Instruction) -> Result<bool, ExecutionError> {
        match instruction {
            Instruction::LoadConst(value) => {
                self.stack.push(value.clone());
                Ok(true)
            }

            Instruction::LoadVar(name) => {
                for scope in self.variables.iter().rev() {
                    if let Some(value) = scope.get(name) {
                        self.stack.push(value.clone());
                        return Ok(true);
                    }
                }
                Err(ExecutionError {
                    message: format!("Variable '{}' not found", name),
                    instruction_index: self.pc,
                })
            }

            Instruction::StoreVar(name) => {
                if let Some(value) = self.stack.pop() {
                    let mut found = false;
                    for scope in self.variables.iter_mut().rev() {
                        if scope.contains_key(name) {
                            scope.insert(name.clone(), value.clone());
                            found = true;
                            break;
                        }
                    }
                    
                    if !found {
                        if let Some(current_scope) = self.variables.last_mut() {
                            current_scope.insert(name.clone(), value);
                        }
                    }
                    Ok(true)
                } else {
                    Err(ExecutionError {
                        message: "Stack underflow on StoreVar".to_string(),
                        instruction_index: self.pc,
                    })
                }
            }

            // ===== OPERACIONES ARITMETICAS =====
            Instruction::Add => self.binary_op(|a, b| match (a, b) {
                (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x + y)),
                (Value::String(x), Value::String(y)) => Ok(Value::String(x.clone() + y)),
                (Value::String(x), Value::Number(y)) => {
                    Ok(Value::String(x.clone() + &y.to_string()))
                }
                (Value::Number(x), Value::String(y)) => Ok(Value::String(x.to_string() + y)),
                _ => Err("Invalid operands for addition".to_string()),
            }),

            Instruction::Sub => self.binary_op(|a, b| match (a, b) {
                (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x - y)),
                _ => Err("Invalid operands for subtraction".to_string()),
            }),

            Instruction::Mul => self.binary_op(|a, b| match (a, b) {
                (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x * y)),
                _ => Err("Invalid operands for multiplication".to_string()),
            }),

            Instruction::Div => self.binary_op(|a, b| match (a, b) {
                (Value::Number(x), Value::Number(y)) => {
                    if *y == 0.0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(Value::Number(x / y))
                    }
                }
                _ => Err("Invalid operands for division".to_string()),
            }),

            Instruction::Mod => self.binary_op(|a, b| match (a, b) {
                (Value::Number(x), Value::Number(y)) => {
                    if *y == 0.0 {
                        Err("Modulo by zero".to_string())
                    } else {
                        Ok(Value::Number(x % y))
                    }
                }
                _ => Err("Invalid operands for modulo".to_string()),
            }),

            // ===== OPERACIONES UNARIAS =====
            Instruction::Neg => {
                if let Some(value) = self.stack.pop() {
                    match value {
                        Value::Number(n) => {
                            self.stack.push(Value::Number(-n));
                            Ok(true)
                        }
                        _ => Err(ExecutionError {
                            message: "Invalid operand for negation".to_string(),
                            instruction_index: self.pc,
                        }),
                    }
                } else {
                    Err(ExecutionError {
                        message: "Stack underflow on Neg".to_string(),
                        instruction_index: self.pc,
                    })
                }
            }

            Instruction::Not => {
                if let Some(value) = self.stack.pop() {
                    self.stack.push(Value::Bool(!value.is_truthy()));
                    Ok(true)
                } else {
                    Err(ExecutionError {
                        message: "Stack underflow on Not".to_string(),
                        instruction_index: self.pc,
                    })
                }
            }

            // ===== OPERACIONES DE COMPARACION =====
            Instruction::Equal => self.comparison_op(|a, b| a == b),
            Instruction::NotEqual => self.comparison_op(|a, b| a != b),
            Instruction::Less => self.numeric_comparison(|a, b| a < b),
            Instruction::Greater => self.numeric_comparison(|a, b| a > b),
            Instruction::LessEqual => self.numeric_comparison(|a, b| a <= b),
            Instruction::GreaterEqual => self.numeric_comparison(|a, b| a >= b),

            // ===== OPERACIONES LOGICAS =====
            Instruction::And => {
                if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                    let result = a.is_truthy() && b.is_truthy();
                    self.stack.push(Value::Bool(result));
                    Ok(true)
                } else {
                    Err(ExecutionError {
                        message: "Stack underflow on And".to_string(),
                        instruction_index: self.pc,
                    })
                }
            }

            Instruction::Or => {
                if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                    let result = a.is_truthy() || b.is_truthy();
                    self.stack.push(Value::Bool(result));
                    Ok(true)
                } else {
                    Err(ExecutionError {
                        message: "Stack underflow on Or".to_string(),
                        instruction_index: self.pc,
                    })
                }
            }

            // ===== CONTROL DE FLUJO =====
            Instruction::Jump(target) => {
                self.pc = *target;
                Ok(false)
            }

            Instruction::JumpIfFalse(target) => {
                if let Some(condition) = self.stack.pop() {
                    if !condition.is_truthy() {
                        self.pc = *target;
                        Ok(false)
                    } else {
                        Ok(true)
                    }
                } else {
                    Err(ExecutionError {
                        message: "Stack underflow on JumpIfFalse".to_string(),
                        instruction_index: self.pc,
                    })
                }
            }

            Instruction::JumpIfTrue(target) => {
                if let Some(condition) = self.stack.pop() {
                    if condition.is_truthy() {
                        self.pc = *target;
                        Ok(false)
                    } else {
                        Ok(true)
                    }
                } else {
                    Err(ExecutionError {
                        message: "Stack underflow on JumpIfTrue".to_string(),
                        instruction_index: self.pc,
                    })
                }
            }

            // ===== MANEJO DE SCOPES =====
            Instruction::PushScope => {
                self.variables.push(HashMap::new());
                Ok(true)
            }

            Instruction::PopScope => {
                if self.variables.len() > 1 {
                    self.variables.pop();
                }
                Ok(true)
            }

            // ===== I/O =====
            Instruction::EmitLn(num_args) => {
                let mut output_parts = Vec::new();

                for _ in 0..*num_args {
                    if let Some(value) = self.stack.pop() {
                        output_parts.push(value.to_string());
                    } else {
                        return Err(ExecutionError {
                            message: "Stack underflow on EmitLn".to_string(),
                            instruction_index: self.pc,
                        });
                    }
                }

                output_parts.reverse();

                for part in output_parts {
                    self.output.push_str(&part);
                }
                self.output.push('\n');

                Ok(true)
            }

            Instruction::Emit(num_args) => {
                let mut output_parts = Vec::new();

                for _ in 0..*num_args {
                    if let Some(value) = self.stack.pop() {
                        output_parts.push(value.to_string());
                    } else {
                        return Err(ExecutionError {
                            message: "Stack underflow on Emit".to_string(),
                            instruction_index: self.pc,
                        });
                    }
                }

                output_parts.reverse();

                for part in output_parts {
                    self.output.push_str(&part);
                }

                Ok(true)
            }

            Instruction::Capture(_var_name) => {
                Ok(true)
            }

            Instruction::RegisterVarType(name, ty) => {
                self.register_variable_type(name, ty.clone());
                Ok(true)
            }

            // ===== UTILIDADES =====
            Instruction::Pop => {
                self.stack.pop();
                Ok(true)
            }

            Instruction::Dup => {
                if let Some(value) = self.stack.last() {
                    self.stack.push(value.clone());
                    Ok(true)
                } else {
                    Err(ExecutionError {
                        message: "Stack underflow on Dup".to_string(),
                        instruction_index: self.pc,
                    })
                }
            }

            Instruction::StartLoopCapture(_, _, _) => {
                Err(ExecutionError {
                    message: "StartLoopCapture debe ser manejado por execute_instructions".to_string(),
                    instruction_index: self.pc,
                })
            }

            Instruction::EndLoopCapture => {
                Ok(true)
            }

            // ===== LLAMADAS A FUNCIONES Y BUILTINS =====
            Instruction::Call(name, argc) => {
                if name.starts_with("__vec_") || name.starts_with("__list_") {
                    self.execute_builtin(name, *argc)?;
                    Ok(true)
                } else {
                    if let Some(&func_addr) = self.function_table.get(name) {
                        self.call_stack.push(self.pc);
                        
                        self.pc = func_addr;
                        Ok(true)
                    } else {
                        Err(ExecutionError {
                            message: format!("Función '{}' no encontrada", name),
                            instruction_index: self.pc,
                        })
                    }
                }
            }

            Instruction::Return => {
                if let Some(return_addr) = self.call_stack.pop() {
                    self.pc = return_addr;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }

            Instruction::Label(_) => {
                Ok(true)
            }

            _ => Err(ExecutionError {
                message: format!("Unimplemented instruction: {:?}", instruction),
                instruction_index: self.pc,
            }),
        }
    }

    // ===== FUNCIONES AUXILIARES =====

    fn binary_op<F>(&mut self, op: F) -> Result<bool, ExecutionError>
    where
        F: Fn(&Value, &Value) -> Result<Value, String>,
    {
        if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
            match op(&a, &b) {
                Ok(result) => {
                    self.stack.push(result);
                    Ok(true)
                }
                Err(msg) => Err(ExecutionError {
                    message: msg,
                    instruction_index: self.pc,
                }),
            }
        } else {
            Err(ExecutionError {
                message: "Stack underflow on binary operation".to_string(),
                instruction_index: self.pc,
            })
        }
    }

    fn comparison_op<F>(&mut self, op: F) -> Result<bool, ExecutionError>
    where
        F: Fn(&Value, &Value) -> bool,
    {
        if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
            let result = op(&a, &b);
            self.stack.push(Value::Bool(result));
            Ok(true)
        } else {
            Err(ExecutionError {
                message: "Stack underflow on comparison".to_string(),
                instruction_index: self.pc,
            })
        }
    }

    fn numeric_comparison<F>(&mut self, op: F) -> Result<bool, ExecutionError>
    where
        F: Fn(f64, f64) -> bool,
    {
        if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
            match (a, b) {
                (Value::Number(x), Value::Number(y)) => {
                    let result = op(x, y);
                    self.stack.push(Value::Bool(result));
                    Ok(true)
                }
                _ => Err(ExecutionError {
                    message: "Invalid operands for numeric comparison".to_string(),
                    instruction_index: self.pc,
                }),
            }
        } else {
            Err(ExecutionError {
                message: "Stack underflow on numeric comparison".to_string(),
                instruction_index: self.pc,
            })
        }
    }

    fn setup_loop_variables(&mut self) {
        if let Some(current_scope) = self.variables.last_mut() {
            current_scope.retain(|k, _| !k.starts_with("loop_"));
            for (index, &counter) in self.loop_counter_stack.iter().enumerate() {
                let var_name = format!("loop_{}", index + 1); // 1-indexed
                current_scope.insert(var_name, Value::Number(counter as f64));
            }
        }
    }

    fn execute_loop_immediately_with_instructions(&mut self, initial: i32, limit: i32, ascending: bool, instructions: &[Instruction]) -> Result<(), ExecutionError> {
        let start_pc = self.pc; // StartLoopCapture actual
        let mut end_pc = start_pc + 1;
        let mut nested_count = 0;
        
        // Buscar el EndLoopCapture correspondiente
        while end_pc < instructions.len() {
            match &instructions[end_pc] {
                Instruction::StartLoopCapture(_, _, _) => {
                    nested_count += 1;
                }
                Instruction::EndLoopCapture => {
                    if nested_count == 0 {
                        break; // Encontramos EndLoopCapture
                    } else {
                        nested_count -= 1;
                    }
                }
                _ => {}
            }
            end_pc += 1;
        }
        
        let body_instructions = &instructions[start_pc + 1..end_pc]; // Cuerpo del bucle
        let mut counter = initial;
        
        while {
            if ascending {
                counter <= limit
            } else {
                counter >= limit
            }
        } {
            self.loop_counter_stack.push(counter);
            self.variables.push(HashMap::new());
            self.setup_loop_variables();
            
            let saved_pc = self.pc;
            self.pc = 0;
            
            // Crear un ejecutor temporal para las instrucciones del cuerpo
            self.execute_body_slice(body_instructions)?;
            
            self.pc = saved_pc;
            
            self.loop_counter_stack.pop();
            if self.variables.len() > 1 {
                self.variables.pop();
            }
            
            if ascending {
                counter += 1;
            } else {
                counter -= 1;
            }
        }
        
        self.pc = end_pc;
        
        Ok(())
    }
    
    fn execute_body_slice(&mut self, body_instructions: &[Instruction]) -> Result<(), ExecutionError> {
        let saved_pc = self.pc;
        self.pc = 0;
        
        while self.pc < body_instructions.len() {
            let instruction = &body_instructions[self.pc];
            
            match instruction {
                Instruction::StartLoopCapture(initial, limit, ascending) => {
                    let start_idx = self.pc;
                    let mut end_idx = start_idx + 1;
                    let mut nested_count = 0;
                    
                    while end_idx < body_instructions.len() {
                        match &body_instructions[end_idx] {
                            Instruction::StartLoopCapture(_, _, _) => nested_count += 1,
                            Instruction::EndLoopCapture => {
                                if nested_count == 0 { break; }
                                nested_count -= 1;
                            }
                            _ => {}
                        }
                        end_idx += 1;
                    }
                    
                    let nested_body = &body_instructions[start_idx + 1..end_idx];
                    
                    let mut counter = *initial;
                    while {
                        if *ascending { counter <= *limit } else { counter >= *limit }
                    } {
                        self.loop_counter_stack.push(counter);
                        self.variables.push(HashMap::new());
                        self.setup_loop_variables();
                        
                        self.execute_body_slice(nested_body)?;
                        
                        self.loop_counter_stack.pop();
                        if self.variables.len() > 1 {
                            self.variables.pop();
                        }
                        
                        if *ascending { counter += 1; } else { counter -= 1; }
                    }
                    
                    self.pc = end_idx;
                }
                Instruction::EndLoopCapture => {
                }
                _ => {
                    match self.execute_instruction(instruction) {
                        Ok(should_continue) => {
                            if !should_continue { 
                                self.pc = saved_pc;
                                return Ok(()); 
                            }
                        }
                        Err(e) => {
                            self.pc = saved_pc;
                            return Err(e);
                        }
                    }
                }
            }
            
            self.pc += 1;
        }
        
        self.pc = saved_pc;
        Ok(())
    }
    
    fn execute_nested_loop(&mut self, initial: i32, limit: i32, ascending: bool, parent_body: &[Instruction]) -> Result<(), ExecutionError> {
        let start_pc = self.pc;
        let mut end_pc = start_pc + 1;
        let mut nested_count = 0;
        
        while end_pc < parent_body.len() {
            match &parent_body[end_pc] {
                Instruction::StartLoopCapture(_, _, _) => nested_count += 1,
                Instruction::EndLoopCapture => {
                    if nested_count == 0 { break; }
                    nested_count -= 1;
                }
                _ => {}
            }
            end_pc += 1;
        }
        
        let nested_body = &parent_body[start_pc + 1..end_pc];
        
        let mut counter = initial;
        while {
            if ascending { counter <= limit } else { counter >= limit }
        } {
            self.loop_counter_stack.push(counter);
            self.variables.push(HashMap::new());
            self.setup_loop_variables();
            
            let saved_pc = self.pc;
            self.pc = 0;
            
            let mut i = 0;
            while i < nested_body.len() {
                let instruction = &nested_body[i];
                
                match instruction {
                    Instruction::StartLoopCapture(inner_initial, inner_limit, inner_ascending) => {
                        let inner_start = i;
                        let mut inner_end = i + 1;
                        let mut inner_nested = 0;
                        
                        while inner_end < nested_body.len() {
                            match &nested_body[inner_end] {
                                Instruction::StartLoopCapture(_, _, _) => inner_nested += 1,
                                Instruction::EndLoopCapture => {
                                    if inner_nested == 0 { break; }
                                    inner_nested -= 1;
                                }
                                _ => {}
                            }
                            inner_end += 1;
                        }
                        
                        let inner_body = &nested_body[inner_start + 1..inner_end];
                        
                        let mut inner_counter = *inner_initial;
                        while {
                            if *inner_ascending { inner_counter <= *inner_limit } else { inner_counter >= *inner_limit }
                        } {
                            self.loop_counter_stack.push(inner_counter);
                            self.variables.push(HashMap::new());
                            self.setup_loop_variables();
                            
                            for inner_instr in inner_body {
                                if matches!(inner_instr, Instruction::EndLoopCapture) {
                                    continue;
                                }
                                if let Err(e) = self.execute_instruction(inner_instr) {
                                    self.pc = saved_pc;
                                    return Err(e);
                                }
                            }
                            
                            self.loop_counter_stack.pop();
                            if self.variables.len() > 1 {
                                self.variables.pop();
                            }
                            
                            if *inner_ascending { inner_counter += 1; } else { inner_counter -= 1; }
                        }
                        
                        i = inner_end + 1;
                        continue;
                    }
                    Instruction::EndLoopCapture => {
                        i += 1;
                        continue;
                    }
                    _ => {
                        match self.execute_instruction(instruction) {
                            Ok(should_continue) => {
                                if !should_continue { 
                                    self.pc = saved_pc;
                                    break;
                                }
                            }
                            Err(e) => {
                                self.pc = saved_pc;
                                return Err(e);
                            }
                        }
                    }
                }
                
                i += 1;
            }
            
            self.pc = saved_pc;
            
            self.loop_counter_stack.pop();
            if self.variables.len() > 1 {
                self.variables.pop();
            }
            
            if ascending { counter += 1; } else { counter -= 1; }
        }
        
        self.pc = end_pc;
        
        Ok(())
    }

    // EJECUTAR FUNCIONES BUILTIN PARA VECTORES Y LISTAS
    fn execute_builtin(&mut self, name: &str, argc: usize) -> Result<(), ExecutionError> {
        use crate::utils::enums::Ty;
        
        match name {
            // ===== OPERACIONES DE VECTORES (solution<T>) =====
            // __vec_from(elem1, elem2, ..., elemN) -> Vector
            "__vec_from" => {
                if argc == 0 {
                    // Vector vacío
                    self.stack.push(Value::Vector {
                        elem: Ty::Unknown,
                        data: Vec::new(),
                    });
                    return Ok(());
                }
                
                // Recolectar elementos del stack
                let mut elements = Vec::with_capacity(argc);
                for _ in 0..argc {
                    if let Some(value) = self.stack.pop() {
                        elements.push(value);
                    } else {
                        return Err(ExecutionError {
                            message: format!("Stack underflow in __vec_from (expected {} elements)", argc),
                            instruction_index: self.pc,
                        });
                    }
                }
                
                // Revertir orden
                elements.reverse();
                
                // Inferir tipo del primer elemento
                let elem_type = if let Some(first) = elements.first() {
                    match first {
                        Value::Number(_) => Ty::AtomNum,
                        Value::String(_) => Ty::Formula,
                        Value::Bool(_) => Ty::Polarized,
                        Value::Char(_) => Ty::Symbol,
                        Value::Void => Ty::VoidState,
                        Value::Vector { elem, .. } => elem.clone(),
                        Value::List { elem, .. } => elem.clone(),
                    }
                } else {
                    Ty::Unknown
                };
                
                // Crear vector
                self.stack.push(Value::Vector {
                    elem: elem_type,
                    data: elements,
                });
                
                Ok(())
            }
            
            // __vec_push(vector, elemento) -> void
            "__vec_push" => {
                if argc != 2 {
                    return Err(ExecutionError {
                        message: format!("__vec_push expects 2 arguments, got {}", argc),
                        instruction_index: self.pc,
                    });
                }
                
                let element = self.stack.pop().ok_or_else(|| ExecutionError {
                    message: "Stack underflow in __vec_push (element)".to_string(),
                    instruction_index: self.pc,
                })?;
                
                let mut vector = self.stack.pop().ok_or_else(|| ExecutionError {
                    message: "Stack underflow in __vec_push (vector)".to_string(),
                    instruction_index: self.pc,
                })?;
                
                match &mut vector {
                    Value::Vector { data, .. } => {
                        data.push(element);
                        self.stack.push(vector); // Devolver vector modificado
                        Ok(())
                    }
                    _ => Err(ExecutionError {
                        message: "Invalid argument for __vec_push (expected Vector)".to_string(),
                        instruction_index: self.pc,
                    }),
                }
            }
            
            // __vec_set(vector, index, elemento) -> void
            "__vec_set" => {
                if argc != 3 {
                    return Err(ExecutionError {
                        message: format!("__vec_set expects 3 arguments, got {}", argc),
                        instruction_index: self.pc,
                    });
                }
                
                let element = self.stack.pop().ok_or_else(|| ExecutionError {
                    message: "Stack underflow in __vec_set (element)".to_string(),
                    instruction_index: self.pc,
                })?;
                
                let index = self.stack.pop().ok_or_else(|| ExecutionError {
                    message: "Stack underflow in __vec_set (index)".to_string(),
                    instruction_index: self.pc,
                })?;
                
                let mut vector = self.stack.pop().ok_or_else(|| ExecutionError {
                    message: "Stack underflow in __vec_set (vector)".to_string(),
                    instruction_index: self.pc,
                })?;
                
                match (&mut vector, index) {
                    (Value::Vector { data, .. }, Value::Number(idx)) => {
                        let idx = idx as usize;
                        if idx >= data.len() {
                            return Err(ExecutionError {
                                message: format!("Index {} out of bounds for vector of length {}", idx, data.len()),
                                instruction_index: self.pc,
                            });
                        }
                        data[idx] = element;
                        self.stack.push(vector);
                        Ok(())
                    }
                    _ => Err(ExecutionError {
                        message: "Invalid arguments for __vec_set (expected Vector, Number, and Value)".to_string(),
                        instruction_index: self.pc,
                    }),
                }
            }
            
            // ===== OPERACIONES DE LISTAS (sample<T>) =====
            
            // __list_from(elem1, elem2, ..., elemN) -> List
            "__list_from" => {
                if argc == 0 {
                    // Lista vacía
                    self.stack.push(Value::List {
                        elem: Ty::Unknown,
                        nodes: Vec::new(),
                    });
                    return Ok(());
                }
                
                // Recolectar elementos del stack
                let mut elements = Vec::with_capacity(argc);
                for _ in 0..argc {
                    if let Some(value) = self.stack.pop() {
                        elements.push(value);
                    } else {
                        return Err(ExecutionError {
                            message: format!("Stack underflow in __list_from (expected {} elements)", argc),
                            instruction_index: self.pc,
                        });
                    }
                }
                
                // Revertir orden
                elements.reverse();
                
                // Inferir tipo del primer elemento
                let elem_type = if let Some(first) = elements.first() {
                    match first {
                        Value::Number(_) => Ty::AtomNum,
                        Value::String(_) => Ty::Formula,
                        Value::Bool(_) => Ty::Polarized,
                        Value::Char(_) => Ty::Symbol,
                        Value::Void => Ty::VoidState,
                        Value::Vector { elem, .. } => elem.clone(),
                        Value::List { elem, .. } => elem.clone(),
                    }
                } else {
                    Ty::Unknown
                };
                
                // Crear lista
                self.stack.push(Value::List {
                    elem: elem_type,
                    nodes: elements,
                });
                
                Ok(())
            }
            
            // __list_push_front(list, elemento) -> void
            "__list_push_front" => {
                if argc != 2 {
                    return Err(ExecutionError {
                        message: format!("__list_push_front expects 2 arguments, got {}", argc),
                        instruction_index: self.pc,
                    });
                }
                
                let element = self.stack.pop().ok_or_else(|| ExecutionError {
                    message: "Stack underflow in __list_push_front (element)".to_string(),
                    instruction_index: self.pc,
                })?;
                
                let mut list = self.stack.pop().ok_or_else(|| ExecutionError {
                    message: "Stack underflow in __list_push_front (list)".to_string(),
                    instruction_index: self.pc,
                })?;
                
                match &mut list {
                    Value::List { nodes, .. } => {
                        nodes.insert(0, element); // Insertar al inicio
                        self.stack.push(list);
                        Ok(())
                    }
                    _ => Err(ExecutionError {
                        message: "Invalid argument for __list_push_front (expected List)".to_string(),
                        instruction_index: self.pc,
                    }),
                }
            }
            
            // __list_push_back(list, elemento) -> void
            "__list_push_back" => {
                if argc != 2 {
                    return Err(ExecutionError {
                        message: format!("__list_push_back expects 2 arguments, got {}", argc),
                        instruction_index: self.pc,
                    });
                }
                
                let element = self.stack.pop().ok_or_else(|| ExecutionError {
                    message: "Stack underflow in __list_push_back (element)".to_string(),
                    instruction_index: self.pc,
                })?;
                
                let mut list = self.stack.pop().ok_or_else(|| ExecutionError {
                    message: "Stack underflow in __list_push_back (list)".to_string(),
                    instruction_index: self.pc,
                })?;
                
                match &mut list {
                    Value::List { nodes, .. } => {
                        nodes.push(element); // Agregar al final
                        self.stack.push(list);
                        Ok(())
                    }
                    _ => Err(ExecutionError {
                        message: "Invalid argument for __list_push_back (expected List)".to_string(),
                        instruction_index: self.pc,
                    }),
                }
            }
            
            // __list_pop_front(list) -> elemento
            "__list_pop_front" => {
                if argc != 1 {
                    return Err(ExecutionError {
                        message: format!("__list_pop_front expects 1 argument, got {}", argc),
                        instruction_index: self.pc,
                    });
                }
                
                let mut list = self.stack.pop().ok_or_else(|| ExecutionError {
                    message: "Stack underflow in __list_pop_front".to_string(),
                    instruction_index: self.pc,
                })?;
                
                match &mut list {
                    Value::List { nodes, .. } => {
                        if nodes.is_empty() {
                            return Err(ExecutionError {
                                message: "Cannot pop from empty list".to_string(),
                                instruction_index: self.pc,
                            });
                        }
                        let element = nodes.remove(0); // Eliminar del inicio
                        self.stack.push(element);
                        Ok(())
                    }
                    _ => Err(ExecutionError {
                        message: "Invalid argument for __list_pop_front (expected List)".to_string(),
                        instruction_index: self.pc,
                    }),
                }
            }
            
            // __list_pop_back(list) -> elemento
            "__list_pop_back" => {
                if argc != 1 {
                    return Err(ExecutionError {
                        message: format!("__list_pop_back expects 1 argument, got {}", argc),
                        instruction_index: self.pc,
                    });
                }
                
                let mut list = self.stack.pop().ok_or_else(|| ExecutionError {
                    message: "Stack underflow in __list_pop_back".to_string(),
                    instruction_index: self.pc,
                })?;
                
                match &mut list {
                    Value::List { nodes, .. } => {
                        if nodes.is_empty() {
                            return Err(ExecutionError {
                                message: "Cannot pop from empty list".to_string(),
                                instruction_index: self.pc,
                            });
                        }
                        let element = nodes.pop().unwrap(); // Eliminar del final
                        self.stack.push(element);
                        Ok(())
                    }
                    _ => Err(ExecutionError {
                        message: "Invalid argument for __list_pop_back (expected List)".to_string(),
                        instruction_index: self.pc,
                    }),
                }
            }
            
            // __list_insert(list, index, elemento) -> void
            "__list_insert" => {
                if argc != 3 {
                    return Err(ExecutionError {
                        message: format!("__list_insert expects 3 arguments, got {}", argc),
                        instruction_index: self.pc,
                    });
                }
                
                let element = self.stack.pop().ok_or_else(|| ExecutionError {
                    message: "Stack underflow in __list_insert (element)".to_string(),
                    instruction_index: self.pc,
                })?;
                
                let index = self.stack.pop().ok_or_else(|| ExecutionError {
                    message: "Stack underflow in __list_insert (index)".to_string(),
                    instruction_index: self.pc,
                })?;
                
                let mut list = self.stack.pop().ok_or_else(|| ExecutionError {
                    message: "Stack underflow in __list_insert (list)".to_string(),
                    instruction_index: self.pc,
                })?;
                
                match (&mut list, index) {
                    (Value::List { nodes, .. }, Value::Number(idx)) => {
                        let idx = idx as usize;
                        if idx > nodes.len() {
                            return Err(ExecutionError {
                                message: format!("Index {} out of bounds for list of length {}", idx, nodes.len()),
                                instruction_index: self.pc,
                            });
                        }
                        nodes.insert(idx, element);
                        self.stack.push(list);
                        Ok(())
                    }
                    _ => Err(ExecutionError {
                        message: "Invalid arguments for __list_insert (expected List, Number, and Value)".to_string(),
                        instruction_index: self.pc,
                    }),
                }
            }
            
            // __list_remove(list, index) -> elemento
            "__list_remove" => {
                if argc != 2 {
                    return Err(ExecutionError {
                        message: format!("__list_remove expects 2 arguments, got {}", argc),
                        instruction_index: self.pc,
                    });
                }
                
                let index = self.stack.pop().ok_or_else(|| ExecutionError {
                    message: "Stack underflow in __list_remove (index)".to_string(),
                    instruction_index: self.pc,
                })?;
                
                let mut list = self.stack.pop().ok_or_else(|| ExecutionError {
                    message: "Stack underflow in __list_remove (list)".to_string(),
                    instruction_index: self.pc,
                })?;
                
                match (&mut list, index) {
                    (Value::List { nodes, .. }, Value::Number(idx)) => {
                        let idx = idx as usize;
                        if idx >= nodes.len() {
                            return Err(ExecutionError {
                                message: format!("Index {} out of bounds for list of length {}", idx, nodes.len()),
                                instruction_index: self.pc,
                            });
                        }
                        let element = nodes.remove(idx);
                        self.stack.push(element);
                        Ok(())
                    }
                    _ => Err(ExecutionError {
                        message: "Invalid arguments for __list_remove (expected List and Number)".to_string(),
                        instruction_index: self.pc,
                    }),
                }
            }
            
            // __list_get (reutiliza __vec_get pero para listas)
            "__list_get" | "__vec_get" => {
                if argc != 2 {
                    return Err(ExecutionError {
                        message: format!("{} expects 2 arguments, got {}", name, argc),
                        instruction_index: self.pc,
                    });
                }
                
                let index = self.stack.pop().ok_or_else(|| ExecutionError {
                    message: format!("Stack underflow in {} (index)", name).to_string(),
                    instruction_index: self.pc,
                })?;
                
                let collection = self.stack.pop().ok_or_else(|| ExecutionError {
                    message: format!("Stack underflow in {} (collection)", name).to_string(),
                    instruction_index: self.pc,
                })?;
                
                match (collection, index) {
                    (Value::Vector { data, .. }, Value::Number(idx)) => {
                        let idx = idx as usize;
                        if idx >= data.len() {
                            return Err(ExecutionError {
                                message: format!("Index {} out of bounds for vector of length {}", idx, data.len()),
                                instruction_index: self.pc,
                            });
                        }
                        self.stack.push(data[idx].clone());
                        Ok(())
                    }
                    (Value::List { nodes, .. }, Value::Number(idx)) => {
                        let idx = idx as usize;
                        if idx >= nodes.len() {
                            return Err(ExecutionError {
                                message: format!("Index {} out of bounds for list of length {}", idx, nodes.len()),
                                instruction_index: self.pc,
                            });
                        }
                        self.stack.push(nodes[idx].clone());
                        Ok(())
                    }
                    _ => Err(ExecutionError {
                        message: format!("Invalid arguments for {} (expected collection and Number)", name).to_string(),
                        instruction_index: self.pc,
                    }),
                }
            }
            
            // __list_len (reutiliza __vec_len pero para listas)
            "__list_len" | "__vec_len" => {
                if argc != 1 {
                    return Err(ExecutionError {
                        message: format!("{} expects 1 argument, got {}", name, argc),
                        instruction_index: self.pc,
                    });
                }
                
                let collection = self.stack.pop().ok_or_else(|| ExecutionError {
                    message: format!("Stack underflow in {}", name).to_string(),
                    instruction_index: self.pc,
                })?;
                
                match collection {
                    Value::Vector { data, .. } => {
                        self.stack.push(Value::Number(data.len() as f64));
                        Ok(())
                    }
                    Value::List { nodes, .. } => {
                        self.stack.push(Value::Number(nodes.len() as f64));
                        Ok(())
                    }
                    _ => Err(ExecutionError {
                        message: format!("Invalid argument for {} (expected collection)", name).to_string(),
                        instruction_index: self.pc,
                    }),
                }
            }
            
            _ => Err(ExecutionError {
                message: format!("Unknown builtin: {}", name),
                instruction_index: self.pc,
            }),
        }
    }


    // Función para debugging: mostrar estado del ejecutor
    pub fn print_state(&self) {
        println!("=== ESTADO DEL EJECUTOR ===");
        println!("PC: {}", self.pc);
        println!("Stack: {:?}", self.stack);
        println!("Variables: {:?}", self.variables);
        println!("Output: {:?}", self.output);
        println!("===========================");
    }
}
