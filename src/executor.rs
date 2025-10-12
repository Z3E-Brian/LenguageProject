use crate::utils::enums::{Instruction, Value};
use std::collections::HashMap;

// ===================== MAQUINA VIRTUAL / EJECUTOR =====================

pub struct Executor {
    stack: Vec<Value>,                      // Pila de valores
    variables: Vec<HashMap<String, Value>>, // Stack de scopes de variables
    output: String,                         // Salida del programa
    pc: usize,                              // Program counter (instruction pointer)
    call_stack: Vec<usize>,                 // Pila de llamadas (para funciones)
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
            variables: vec![HashMap::new()], // Scope global
            output: String::new(),
            pc: 0,
            call_stack: Vec::new(),
        }
    }

    // Función principal: ejecutar todas las instrucciones
    pub fn execute(&mut self, instructions: &[Instruction]) -> Result<String, ExecutionError> {
        self.pc = 0;
        self.output.clear();

        while self.pc < instructions.len() {
            match self.execute_instruction(&instructions[self.pc]) {
                Ok(should_continue) => {
                    if !should_continue {
                        break; // Return o halt
                    }
                    self.pc += 1;
                }
                Err(e) => return Err(e),
            }
        }

        Ok(self.output.clone())
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
