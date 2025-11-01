use crate::utils::enums::{Instruction, Value};
use std::collections::HashMap;

// ===================== MAQUINA VIRTUAL / EJECUTOR =====================

pub struct Executor {
    stack: Vec<Value>,                      // Pila de valores
    variables: Vec<HashMap<String, Value>>, // Stack de scopes de variables
    output: String,                         // Salida del programa
    pc: usize,                              // Program counter (instruction pointer)
    call_stack: Vec<usize>,                 // Pila de llamadas (para funciones)
    
    // 🆕 SISTEMA DINÁMICO DE BUCLES ANIDADOS (EJECUCIÓN INMEDIATA)
    loop_counter_stack: Vec<i32>,               // Stack de contadores [exterior -> interior]
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
            variables: vec![HashMap::new()], // Iniciamos con un scope global
            output: String::new(),
            pc: 0,
            call_stack: Vec::new(),
            
            // 🆕 INICIALIZACIÓN DEL SISTEMA DINÁMICO DE BUCLES
            loop_counter_stack: Vec::new(),
        }
    }

    // Función principal: ejecutar todas las instrucciones
    pub fn execute(&mut self, instructions: &[Instruction]) -> Result<String, ExecutionError> {
        self.pc = 0;
        self.output.clear();
        self.execute_instructions(instructions)?;
        Ok(self.output.clone())
    }

    // 🎯 NUEVA FUNCIÓN: Ejecutar instrucciones de forma inmediata (como C++)
    fn execute_instructions(&mut self, instructions: &[Instruction]) -> Result<(), ExecutionError> {
        while self.pc < instructions.len() {
            let current_instruction = &instructions[self.pc];
            
            match current_instruction {
                Instruction::StartLoopCapture(initial, limit, ascending) => {
                    // 🚀 EJECUTAR BUCLE INMEDIATAMENTE 
                    self.execute_loop_immediately_with_instructions(*initial, *limit, *ascending, instructions)?;
                    // El PC ya fue actualizado dentro de la función del bucle
                }
                _ => {
                    match self.execute_instruction(current_instruction) {
                        Ok(should_continue) => {
                            if !should_continue {
                                break; // Return o halt
                            }
                            self.pc += 1; // Solo avanzar aquí para instrucciones normales
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
        }
        Ok(())
    }

    // Ejecutar una instrucción individual
    fn execute_instruction(&mut self, instruction: &Instruction) -> Result<bool, ExecutionError> {
        match instruction {
            // ===== MANEJO DE VALORES =====
            Instruction::LoadConst(value) => {
                self.stack.push(value.clone());
                Ok(true)
            }

            Instruction::LoadVar(name) => {
                // Buscar variable en scopes (desde el más reciente)
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
                    // Almacenar en el scope más reciente
                    if let Some(current_scope) = self.variables.last_mut() {
                        current_scope.insert(name.clone(), value);
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
                Ok(false) // No incrementar PC automáticamente
            }

            Instruction::JumpIfFalse(target) => {
                if let Some(condition) = self.stack.pop() {
                    if !condition.is_truthy() {
                        self.pc = *target;
                        Ok(false) // No incrementar PC automáticamente
                    } else {
                        Ok(true) // Continuar normalmente
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
                        Ok(false) // No incrementar PC automáticamente
                    } else {
                        Ok(true) // Continuar normalmente
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
                    // Mantener al menos el scope global
                    self.variables.pop();
                }
                Ok(true)
            }

            // ===== I/O =====
            Instruction::EmitLn(num_args) => {
                let mut output_parts = Vec::new();

                // Recoger argumentos del stack (en orden inverso)
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

                // Invertir para obtener el orden correcto
                output_parts.reverse();

                // Agregar a la salida
                for part in output_parts {
                    self.output.push_str(&part);
                }
                self.output.push('\n');

                Ok(true)
            }

            Instruction::Emit(num_args) => {
                let mut output_parts = Vec::new();

                // Recoger argumentos del stack (en orden inverso)
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

                // Invertir para obtener el orden correcto
                output_parts.reverse();

                // Agregar a la salida (sin salto de línea)
                for part in output_parts {
                    self.output.push_str(&part);
                }

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

            // ===== 🎯 BUCLES CON EJECUCIÓN INMEDIATA (COMO C++) =====
            Instruction::StartLoopCapture(_, _, _) => {
                // StartLoopCapture se maneja en execute_instructions, no aquí
                Err(ExecutionError {
                    message: "StartLoopCapture debe ser manejado por execute_instructions".to_string(),
                    instruction_index: self.pc,
                })
            }

            Instruction::EndLoopCapture => {
                // EndLoopCapture ya fue procesado por execute_loop_immediately
                // Solo saltar esta instrucción
                Ok(true)
            }

            // ===== NO IMPLEMENTADAS AUN =====
            Instruction::Call(_, _) => Err(ExecutionError {
                message: "Function calls not implemented yet".to_string(),
                instruction_index: self.pc,
            }),

            Instruction::Return => {
                // Por ahora, simplemente terminar ejecución
                Ok(false)
            }

            Instruction::Label(_) => {
                // Las etiquetas no se ejecutan, solo sirven de referencia
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

    // 🎯 CONFIGURAR VARIABLES loop_N AUTOMÁTICAS
    fn setup_loop_variables(&mut self) {
        if let Some(current_scope) = self.variables.last_mut() {
            // Limpiar variables de bucle anteriores
            current_scope.retain(|k, _| !k.starts_with("loop_"));
            
            // Crear variables loop_1, loop_2, loop_3, ... (1-indexed desde exterior)
            for (index, &counter) in self.loop_counter_stack.iter().enumerate() {
                let var_name = format!("loop_{}", index + 1); // 1-indexed
                current_scope.insert(var_name, Value::Number(counter as f64));
            }
            
            // Variables loop_N configuradas correctamente
        }
    }

    // 🎯 EJECUTAR BUCLE INMEDIATAMENTE CON ACCESO A INSTRUCCIONES (COMO C++)
    fn execute_loop_immediately_with_instructions(&mut self, initial: i32, limit: i32, ascending: bool, instructions: &[Instruction]) -> Result<(), ExecutionError> {
        
        // 🔍 ENCONTRAR EL RANGO DEL CUERPO DEL BUCLE (StartLoopCapture -> EndLoopCapture)
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
                        break; // Encontramos nuestro EndLoopCapture
                    } else {
                        nested_count -= 1;
                    }
                }
                _ => {}
            }
            end_pc += 1;
        }
        
        let body_instructions = &instructions[start_pc + 1..end_pc]; // Cuerpo del bucle
        
        // 🔄 BUCLE PRINCIPAL: Iterar desde initial hasta limit
        let mut counter = initial;
        
        while {
            // Verificar condición de continuación
            if ascending {
                counter <= limit
            } else {
                counter >= limit
            }
        } {
            
            // 📍 CONFIGURAR CONTEXTO PARA ESTA ITERACIÓN
            self.loop_counter_stack.push(counter);
            self.variables.push(HashMap::new());
            self.setup_loop_variables();
            
            // 🚀 EJECUTAR CUERPO DEL BUCLE
            let saved_pc = self.pc;
            self.pc = 0; // Reset PC para ejecutar el cuerpo
            
            // Crear un ejecutor temporal para las instrucciones del cuerpo
            self.execute_body_slice(body_instructions)?;
            
            self.pc = saved_pc; // Restaurar PC
            
            // 🧹 LIMPIAR DESPUÉS DE LA ITERACIÓN
            self.loop_counter_stack.pop();
            if self.variables.len() > 1 {
                self.variables.pop();
            }
            
            // Avanzar contador
            if ascending {
                counter += 1;
            } else {
                counter -= 1;
            }
        }
        
        // Posicionar PC después del EndLoopCapture
        self.pc = end_pc;
        
        Ok(())
    }
    
    // 🎯 EJECUTAR UN SLICE DE INSTRUCCIONES (CUERPO DEL BUCLE)
    fn execute_body_slice(&mut self, body_instructions: &[Instruction]) -> Result<(), ExecutionError> {
        let saved_pc = self.pc;
        self.pc = 0;
        
        while self.pc < body_instructions.len() {
            let instruction = &body_instructions[self.pc];
            
            match instruction {
                Instruction::StartLoopCapture(initial, limit, ascending) => {
                    // Bucle anidado: ejecutar con el contexto simplificado
                    self.execute_nested_loop(*initial, *limit, *ascending, body_instructions)?;
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
    
    // 🎯 MANEJAR BUCLE ANIDADO (SIMPLIFICADO)
    fn execute_nested_loop(&mut self, initial: i32, limit: i32, ascending: bool, parent_body: &[Instruction]) -> Result<(), ExecutionError> {
        // Por ahora, implementación simple que encuentra el cuerpo del bucle anidado
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
        
        // Ejecutar bucle anidado
        let mut counter = initial;
        while {
            if ascending { counter <= limit } else { counter >= limit }
        } {
            self.loop_counter_stack.push(counter);
            self.variables.push(HashMap::new());
            self.setup_loop_variables();
            
            // Ejecutar cuerpo del bucle anidado
            let saved_pc = self.pc;
            self.pc = 0;
            
            for instruction in nested_body {
                if let Instruction::StartLoopCapture(_, _, _) | Instruction::EndLoopCapture = instruction {
                    continue; // Saltar instrucciones de bucle en bucles anidados por ahora
                }
                
                match self.execute_instruction(instruction) {
                    Ok(should_continue) => {
                        if !should_continue { break; }
                    }
                    Err(e) => {
                        self.pc = saved_pc;
                        return Err(e);
                    }
                }
            }
            
            self.pc = saved_pc;
            
            self.loop_counter_stack.pop();
            if self.variables.len() > 1 {
                self.variables.pop();
            }
            
            if ascending { counter += 1; } else { counter -= 1; }
        }
        
        // Posicionar después del EndLoopCapture del bucle anidado
        self.pc = end_pc;
        
        Ok(())
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
