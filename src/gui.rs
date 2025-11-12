use crate::{run_lexer, run_parser, run_semantics};
use eframe::egui;
use eframe::NativeOptions;
use eframe::App;

pub struct IDE {
    pub code: String,
    pub output: String,
    pub light_mode: bool,
    pub show_options: bool,
    pub show_snippets: bool,
    pub selected_tab: SnippetTab,
    pub waiting_for_input: bool,
    pub input_var_name: String,
    pub input_var_type: crate::utils::enums::Ty,
    pub input_buffer: String,
    pub executor_state: Option<ExecutorState>,
}

pub struct ExecutorState {
    pub executor: crate::executor::Executor,
    pub instructions: Vec<crate::utils::enums::Instruction>,
}

impl Default for IDE {
    fn default() -> Self {
        Self {
            code: String::new(),
            output: String::new(),
            light_mode: false,
            show_options: false,
            show_snippets: false,
            selected_tab: SnippetTab::default(),
            waiting_for_input: false,
            input_var_name: String::new(),
            input_var_type: crate::utils::enums::Ty::Unknown,
            input_buffer: String::new(),
            executor_state: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SnippetTab {
    #[default]
    Declarations,
    Control,
    Loops,
    Functions,
    Common,
}

impl IDE {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn show(&mut self, ctx: &egui::Context) {
        if self.light_mode {
            ctx.set_visuals(egui::Visuals::light());
        } else {
            ctx.set_visuals(egui::Visuals::dark());
        }

        // ===== BARRA SUPERIOR (NEGRA) con botones a la DERECHA =====
        egui::TopBottomPanel::top("topbar")
            .frame(egui::Frame::none().fill(egui::Color32::BLACK))
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new("📄 Nuevo").fill(egui::Color32::DARK_GRAY))
                        .clicked() {
                        self.code.clear();
                        self.output.clear();
                    }
                    if ui.add(egui::Button::new("🧩 Snippets").fill(egui::Color32::DARK_GRAY))
                        .clicked() {
                        self.show_snippets = !self.show_snippets;
                    }
                    if ui.add(egui::Button::new("⚙️ Opciones").fill(egui::Color32::DARK_GRAY))
                        .clicked() {
                        self.show_options = !self.show_options;
                    }
                    if ui.add(egui::Button::new("❌ Salir").fill(egui::Color32::from_rgb(80, 0, 0)))
                        .clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    ui.add_space(8.0);
                });
            });

        // Mostrar ventana de opciones si está activado
        if self.show_options {
            egui::Window::new("Opciones")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.checkbox(&mut self.light_mode, "Modo claro");
                    if ui.button("Cerrar").clicked() {
                        self.show_options = false;
                    }
                });
        }

        // Mostrar ventana de snippets si está activado
        if self.show_snippets {
            egui::Window::new("Snippets de Código")
                .collapsible(false)
                .resizable(true)
                .default_width(400.0)
                .default_height(500.0)
                .show(ctx, |ui| {
                    self.show_snippets_content(ui);
                });
        }

        // ===== ÁREA PRINCIPAL (TODO NEGRO) =====
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::BLACK))
            .show(ctx, |ui| {
                let available_height = ui.available_height();
                let available_width = ui.available_width();
                
                // Usar horizontal layout con espaciado
                ui.horizontal(|ui| {
                    // -------- IZQUIERDA: EDITOR --------
                    ui.vertical(|ui| {
                        let panel_width = available_width / 2.0 - 10.0;
                        ui.set_width(panel_width);
                        ui.set_height(available_height);
                        
                        ui.label(egui::RichText::new("EDITOR").color(egui::Color32::WHITE).strong());
                        ui.add_space(4.0);
                        
                        let editor_height = available_height - 28.0; 
                        egui::Frame::group(ui.style())
                            .fill(egui::Color32::from_rgb(30, 30, 30))
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(80)))
                            .show(ui, |ui| {
                                egui::ScrollArea::vertical()
                                    .id_source("editor_scroll")
                                    .max_height(editor_height)
                                    .show(ui, |ui| {
                                        ui.add_sized(
                                            [panel_width - 18.0, editor_height],
                                            egui::TextEdit::multiline(&mut self.code)
                                                .code_editor()
                                                .text_color(egui::Color32::WHITE)
                                                .hint_text("Escribe tu código aquí..."),
                                        );
                                    });
                            });
                    });

                    ui.add_space(8.0);

                    // -------- DERECHA: OUTPUT + BOTONES DEBAJO --------
                    ui.vertical(|ui| {
                        let panel_width = available_width / 2.0 - 10.0;
                        ui.set_width(panel_width);
                        ui.set_height(available_height);
                        
                        ui.label(egui::RichText::new("OUTPUT").color(egui::Color32::WHITE).strong());
                        ui.add_space(4.0);

                        let output_height = available_height - 70.0; 
                        
                        // Terminal estilo C++ - fondo negro puro, texto monoespacio
                        egui::Frame::none()
                            .fill(egui::Color32::BLACK)
                            .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 180, 0)))
                            .inner_margin(egui::Margin::same(8.0))
                            .show(ui, |ui| {
                                egui::ScrollArea::vertical()
                                    .id_source("output_scroll")
                                    .max_height(output_height)
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        ui.set_width(panel_width - 20.0);
                                        
                                        // Mostrar el output con estilo terminal
                                        ui.add(
                                            egui::TextEdit::multiline(&mut self.output)
                                                .interactive(false)
                                                .text_color(egui::Color32::from_rgb(0, 255, 0)) // Verde brillante estilo terminal
                                                .font(egui::FontId::monospace(14.0))
                                                .desired_width(panel_width - 36.0)
                                        );
                                        
                                        // Si está esperando entrada, mostrar prompt estilo terminal
                                        if self.waiting_for_input {
                                            ui.add_space(5.0);
                                            
                                            // Mostrar tipo esperado en color cyan
                                            let type_str = match &self.input_var_type {
                                                crate::utils::enums::Ty::AtomNum => "int",
                                                crate::utils::enums::Ty::Mass => "double",
                                                crate::utils::enums::Ty::Polarized => "bool",
                                                crate::utils::enums::Ty::Formula => "string",
                                                _ => "value",
                                            };
                                            
                                            ui.label(egui::RichText::new(format!("// Esperando entrada: {} (tipo: {})", self.input_var_name, type_str))
                                                .color(egui::Color32::from_rgb(100, 200, 255)) // Cyan para comentarios
                                                .font(egui::FontId::monospace(13.0)));
                                            
                                            // Prompt estilo terminal con cursor parpadeante
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new("> ")
                                                    .color(egui::Color32::from_rgb(255, 255, 0)) // Amarillo para el prompt
                                                    .font(egui::FontId::monospace(14.0))
                                                    .strong());
                                                
                                                // Campo de entrada estilo terminal
                                                let response = ui.add(
                                                    egui::TextEdit::singleline(&mut self.input_buffer)
                                                        .text_color(egui::Color32::WHITE)
                                                        .font(egui::FontId::monospace(14.0))
                                                        .desired_width(panel_width - 60.0)
                                                        .hint_text("")
                                                );
                                                
                                                // Si presiona Enter, procesar la entrada
                                                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                                    self.process_input();
                                                }
                                                
                                                // Mantener el foco en el campo de entrada
                                                if self.waiting_for_input {
                                                    response.request_focus();
                                                }
                                            });
                                            
                                            ui.add_space(5.0);
                                        }
                                    });
                            });

                        ui.add_space(8.0);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("▶️ EJECUTAR").clicked() {
                                self.output = self.ejecutar();
                            }
                            if ui.button("🔨 COMPILAR").clicked() {
                                self.output = self.compilar();
                            }
                        });
                    });
                });
            });
    }
    
    // ====== Implementaciones de compilación y ejecución ======
    fn compilar(&self) -> String {
        if self.code.trim().is_empty() {
            return "Error: archivo vacío".into();
        }

        // 1. Análisis léxico
        let tokens = match run_lexer(&self.code) {
            Ok(toks) => toks,
            Err(e) => return format!("Error léxico: {}", e),
        };

        // 2. Análisis sintáctico
        let ast = match run_parser(tokens) {
            Ok(ast) => ast,
            Err(e) => return format!("Error de parseo: {} @ {}:{}", e.message, e.line, e.col),
        };

        // 3. Análisis semántico
        match run_semantics(&ast) {
            Ok(_) => format!("Compilación exitosa. Código válido."),
            Err(errors) => {
                let mut err_msg = String::from("Errores semánticos:\n");
                for e in errors {
                    err_msg.push_str(&format!("- {} @ {}:{}\n", e.msg, e.line, e.col));
                }
                err_msg
            }
        }
    }

    fn ejecutar(&mut self) -> String {
        if self.code.trim().is_empty() {
            return "Error: archivo vacío".into();
        }

        // Iniciar ejecución (reset estado si había algo anterior)
        self.waiting_for_input = false;
        self.input_var_name.clear();
        self.input_buffer.clear();
        self.output.clear();

        // 1. Compilar el código
        let tokens = match crate::run_lexer(&self.code) {
            Ok(toks) => toks,
            Err(e) => return format!("Error léxico: {}", e),
        };

        let ast = match crate::run_parser(tokens) {
            Ok(ast) => ast,
            Err(e) => return format!("Error de parseo: {} @ {}:{}", e.message, e.line, e.col),
        };

        match crate::run_semantics(&ast) {
            Ok(_) => {},
            Err(errors) => {
                let mut err_msg = String::from("Errores semánticos:\n");
                for e in errors {
                    err_msg.push_str(&format!("- {} @ {}:{}\n", e.msg, e.line, e.col));
                }
                return err_msg;
            }
        }

        let instructions = match crate::run_codegen(&ast) {
            Ok(instrs) => instrs,
            Err(e) => return format!("Error de generación de código: {}", e),
        };

        // 2. Crear ejecutor y comenzar ejecución
        let mut executor = crate::executor::Executor::new();
        
        // Ejecutar hasta encontrar un Capture o terminar
        match self.execute_until_input(&mut executor, &instructions) {
            Ok(finished) => {
                if finished {
                    // Programa terminado
                    self.output = executor.get_output().to_string();
                    self.executor_state = None;
                    if self.output.is_empty() {
                        "Programa ejecutado sin salida.".into()
                    } else {
                        self.output.clone()
                    }
                } else {
                    // Esperando entrada
                    self.output = executor.get_output().to_string();
                    self.executor_state = Some(ExecutorState { executor, instructions });
                    self.output.clone()
                }
            }
            Err(e) => format!("Error de ejecución: {}", e),
        }
    }
    
    fn execute_until_input(
        &mut self,
        executor: &mut crate::executor::Executor,
        instructions: &[crate::utils::enums::Instruction],
    ) -> Result<bool, String> {
        // Ejecutar instrucciones hasta encontrar Capture o terminar
        executor.execute_until_capture(instructions)
            .map(|status| {
                match status {
                    crate::executor::ExecutionStatus::WaitingForInput(var_name, var_type) => {
                        self.waiting_for_input = true;
                        self.input_var_name = var_name;
                        self.input_var_type = var_type;
                        false // No terminado
                    }
                    crate::executor::ExecutionStatus::Finished => {
                        self.waiting_for_input = false;
                        true // Terminado
                    }
                }
            })
            .map_err(|e| e.to_string())
    }
    
    fn process_input(&mut self) {
        if let Some(mut state) = self.executor_state.take() {
            let input_text = self.input_buffer.trim();
            
            // Agregar el input al output (eco estilo terminal)
            self.output.push_str(&format!("> {}\n", self.input_buffer));
            
            // Validar y parsear el input según el tipo esperado
            let parse_result = match &self.input_var_type {
                crate::utils::enums::Ty::AtomNum => {
                    // Debe ser un número entero
                    match input_text.parse::<f64>() {
                        Ok(num) if num.fract() == 0.0 => {
                            Ok(crate::utils::enums::Value::Number(num))
                        }
                        Ok(_) => Err(format!("❌ Error de tipo: '{}' requiere un número entero (int)", self.input_var_name)),
                        Err(_) => Err(format!("❌ Error de tipo: '{}' requiere un número entero (int)", self.input_var_name)),
                    }
                }
                crate::utils::enums::Ty::Mass => {
                    // Debe ser un número (puede ser decimal)
                    match input_text.parse::<f64>() {
                        Ok(num) => Ok(crate::utils::enums::Value::Number(num)),
                        Err(_) => Err(format!("❌ Error de tipo: '{}' requiere un número decimal (double)", self.input_var_name)),
                    }
                }
                crate::utils::enums::Ty::Polarized => {
                    // Debe ser true o false
                    if input_text.eq_ignore_ascii_case("true") {
                        Ok(crate::utils::enums::Value::Bool(true))
                    } else if input_text.eq_ignore_ascii_case("false") {
                        Ok(crate::utils::enums::Value::Bool(false))
                    } else {
                        Err(format!("❌ Error de tipo: '{}' requiere un booleano (bool: true/false)", self.input_var_name))
                    }
                }
                crate::utils::enums::Ty::Formula => {
                    // Cualquier texto es válido
                    Ok(crate::utils::enums::Value::String(self.input_buffer.trim().to_string()))
                }
                _ => {
                    // Para tipos desconocidos, intentar parsear como número o usar como string
                    if let Ok(num) = input_text.parse::<f64>() {
                        Ok(crate::utils::enums::Value::Number(num))
                    } else if input_text.eq_ignore_ascii_case("true") {
                        Ok(crate::utils::enums::Value::Bool(true))
                    } else if input_text.eq_ignore_ascii_case("false") {
                        Ok(crate::utils::enums::Value::Bool(false))
                    } else {
                        Ok(crate::utils::enums::Value::String(self.input_buffer.trim().to_string()))
                    }
                }
            };
            
            // Limpiar el buffer de entrada
            self.input_buffer.clear();
            
            match parse_result {
                Ok(value) => {
                    // Almacenar el valor capturado en el ejecutor
                    state.executor.store_captured_value(&self.input_var_name, value);
                    
                    // Continuar ejecución
                    match self.execute_until_input(&mut state.executor, &state.instructions) {
                        Ok(finished) => {
                            if finished {
                                // Programa terminado - limpiar todo
                                self.output = state.executor.get_output().to_string();
                                self.executor_state = None;
                                self.waiting_for_input = false;
                            } else {
                                // Todavía esperando más entrada - mantener el estado de espera
                                self.output = state.executor.get_output().to_string();
                                self.executor_state = Some(state);
                                // waiting_for_input ya fue configurado a true en execute_until_input
                            }
                        }
                        Err(e) => {
                            self.output.push_str(&format!("\n❌ Error: {}\n", e));
                            self.executor_state = None;
                            self.waiting_for_input = false;
                        }
                    }
                }
                Err(error_msg) => {
                    // Error de validación de tipo - mostrar mensaje y mantener esperando
                    self.output.push_str(&format!("{}\n", error_msg));
                    // Restaurar el estado y seguir esperando
                    self.executor_state = Some(state);
                    // waiting_for_input sigue siendo true
                }
            }
        } else {
            // No hay estado de ejecución, solo limpiar
            self.input_buffer.clear();
            self.waiting_for_input = false;
        }
    }

    // Las funciones de ejecución ahora están en codegen.rs y executor.rs

    // ====== Funcionalidad de Snippets ======
    fn show_snippets_content(&mut self, ui: &mut egui::Ui) {
        // Pestañas para diferentes categorías
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.selected_tab, SnippetTab::Declarations, "📝 Declaraciones");
            ui.selectable_value(&mut self.selected_tab, SnippetTab::Control, "🔀 Control");
            ui.selectable_value(&mut self.selected_tab, SnippetTab::Loops, "🔄 Bucles");
            ui.selectable_value(&mut self.selected_tab, SnippetTab::Functions, "⚙️ Funciones");
            ui.selectable_value(&mut self.selected_tab, SnippetTab::Common, "🔧 Común");
        });
        
        ui.separator();
        
        match self.selected_tab {
            SnippetTab::Declarations => self.show_declarations_snippets(ui),
            SnippetTab::Control => self.show_control_snippets(ui),
            SnippetTab::Loops => self.show_loops_snippets(ui),
            SnippetTab::Functions => self.show_functions_snippets(ui),
            SnippetTab::Common => self.show_common_snippets(ui),
        }
        
        ui.separator();
        if ui.button("❌ Cerrar").clicked() {
            self.show_snippets = false;
        }
    }
    
    fn show_declarations_snippets(&mut self, ui: &mut egui::Ui) {
        ui.heading("📝 Declaraciones de Variables");
        
        ui.horizontal(|ui| {
            if ui.button("🔢 Variable Entero").clicked() {
                self.insert_snippet("atom variable_name : atom_num = 0;");
            }
            if ui.button("🌊 Variable Decimal").clicked() {
                self.insert_snippet("atom variable_name : mass = 0.0;");
            }
        });
        
        ui.horizontal(|ui| {
            if ui.button("📝 Variable String").clicked() {
                self.insert_snippet("atom variable_name : formula = \"texto\";");
            }
            if ui.button("✅ Variable Booleana").clicked() {
                self.insert_snippet("atom variable_name : polarized = true;");
            }
        });
        
        ui.separator();
        ui.heading("🔒 Constantes");
        
        ui.horizontal(|ui| {
            if ui.button("🔢 Constante Entero").clicked() {
                self.insert_snippet("ion CONSTANT_NAME : atom_num = 42;");
            }
            if ui.button("🌊 Constante Decimal").clicked() {
                self.insert_snippet("ion CONSTANT_NAME : mass = 3.14159;");
            }
        });
        
        if ui.button("📝 Constante String").clicked() {
            self.insert_snippet("ion CONSTANT_NAME : formula = \"valor constante\";");
        }
    }
    
    fn show_control_snippets(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔀 Estructuras de Control");
        
        if ui.button("🔍 If Simple").clicked() {
            self.insert_snippet("itest (condicion) {\n    // código aquí\n}");
        }
        
        if ui.button("🔍 If-Else").clicked() {
            self.insert_snippet("itest (condicion) {\n    // código si verdadero\n} notest {\n    // código si falso\n}");
        }
        
        if ui.button("🔍 If-ElseIf-Else").clicked() {
            self.insert_snippet("itest (condicion1) {\n    // código condición 1\n} inotest (condicion2) {\n    // código condición 2\n} notest {\n    // código por defecto\n}");
        }
        
        ui.separator();
        ui.heading("💬 Salida");
        
        ui.horizontal(|ui| {
            if ui.button("📝 Emit (sin salto)").clicked() {
                self.insert_snippet("emit(\"mensaje\");");
            }
            if ui.button("📝 Emitln (con salto)").clicked() {
                self.insert_snippet("emitln(\"mensaje\");");
            }
        });
        
        ui.horizontal(|ui| {
            if ui.button("⌨️ Capture Variable").clicked() {
                self.insert_snippet("!! Declarar variable primero\natom entrada : formula = \"\";\n!! Capturar entrada del usuario\ncapture(entrada);\nemitln(\"Entrada capturada: \", entrada);");
            }
        });
        
        ui.horizontal(|ui| {
            if ui.button("🔢 Emit con Variable").clicked() {
                self.insert_snippet("emitln(\"Valor: \", variable_name);");
            }
            if ui.button("🧮 Emit con Operación").clicked() {
                self.insert_snippet("emitln(\"Resultado: \", a + b);");
            }
        });
    }
    
    fn show_loops_snippets(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔄 Bucles Chain");
        
        ui.label("📋 Bucles Básicos:");
        ui.horizontal(|ui| {
            if ui.button("🔄 Bucle Simple").clicked() {
                self.insert_snippet("chain 3 {\n    emit(\"Iteración: \", i, \"\\n\");\n}");
            }
            if ui.button("⬆️ Bucle Ascendente").clicked() {
                self.insert_snippet("chain 1 to 10 {\n    emit(\"Número: \", i, \"\\n\");\n}");
            }
        });
        
        ui.horizontal(|ui| {
            if ui.button("⬇️ Bucle Descendente").clicked() {
                self.insert_snippet("chain 10 to 1 {\n    emit(\"Countdown: \", i, \"\\n\");\n}");
            }
            if ui.button("🎯 Bucle Personalizado").clicked() {
                self.insert_snippet("chain 5 to 15 {\n    // Usar variable 'i' aquí\n    emit(\"Valor actual: \", i, \"\\n\");\n}");
            }
        });
        
        ui.separator();
        ui.label("🧮 Bucles con Matemáticas:");
        
        ui.horizontal(|ui| {
            if ui.button("✖️ Tabla de Multiplicar").clicked() {
                self.insert_snippet("chain 1 to 10 {\n    atom resultado : atom_num = 5 * i;\n    emit(\"5 x \", i, \" = \", resultado, \"\\n\");\n}");
            }
            if ui.button("🔢 Potencias").clicked() {
                self.insert_snippet("chain 1 to 5 {\n    atom cuadrado : atom_num = i * i;\n    atom cubo : atom_num = i * i * i;\n    emit(\"i=\", i, \" i²=\", cuadrado, \" i³=\", cubo, \"\\n\");\n}");
            }
        });
        
        ui.horizontal(|ui| {
            if ui.button("➕ Suma Acumulativa").clicked() {
                self.insert_snippet("atom suma : atom_num = 0;\nchain 1 to 100 {\n    suma = suma + i;\n    emit(\"Suma hasta \", i, \": \", suma, \"\\n\");\n}\nemit(\"Suma total: \", suma, \"\\n\");");
            }
            if ui.button("📊 Factorial").clicked() {
                self.insert_snippet("atom factorial : atom_num = 1;\nchain 1 to 5 {\n    factorial = factorial * i;\n    emit(\"Factorial de \", i, \": \", factorial, \"\\n\");\n}");
            }
        });
        
        ui.separator();
        ui.label("🎯 Bucles con Condicionales:");
        
        ui.horizontal(|ui| {
            if ui.button("🔢 Par/Impar").clicked() {
                self.insert_snippet("chain 1 to 20 {\n    itest (i % 2 == 0) {\n        emit(i, \" es par\\n\");\n    } notest {\n        emit(i, \" es impar\\n\");\n    }\n}");
            }
            if ui.button("🔍 Búsqueda").clicked() {
                self.insert_snippet("atom encontrado : polarized = false;\nchain 1 to 100 {\n    itest (i == 42) {\n        emit(\"¡Encontrado en posición: \", i, \"!\\n\");\n        encontrado = true;\n    }\n}");
            }
        });
        
        ui.horizontal(|ui| {
            if ui.button("📈 Números Primos").clicked() {
                self.insert_snippet("chain 2 to 50 {\n    atom es_primo : polarized = true;\n    // Verificar si i es primo\n    itest (i > 2) {\n        atom divisor : atom_num = 2;\n        // Simplificado: verificar solo algunos divisores\n        itest (i % divisor == 0) {\n            es_primo = false;\n        }\n    }\n    itest (es_primo) {\n        emit(i, \" es primo\\n\");\n    }\n}");
            }
            if ui.button("🎲 Filtro de Rango").clicked() {
                self.insert_snippet("chain 1 to 100 {\n    itest (i >= 10 && i <= 20) {\n        emit(\"En rango: \", i, \"\\n\");\n    } inotest (i < 10) {\n        emit(\"Muy pequeño: \", i, \"\\n\");\n    } notest {\n        emit(\"Muy grande: \", i, \"\\n\");\n    }\n}");
            }
        });
        
        ui.separator();
        ui.label("🎨 Bucles de Patrones:");
        
        ui.horizontal(|ui| {
            if ui.button("📐 Triángulo").clicked() {
                self.insert_snippet("chain 1 to 5 {\n    chain 1 to i {\n        emit(\"* \");\n    }\n    emit(\"\\n\");\n}");
            }
            if ui.button("🔴 Círculos").clicked() {
                self.insert_snippet("chain 1 to 3 {\n    emit(\"Círculo \", i, \": \");\n    chain 1 to i {\n        emit(\"O \");\n    }\n    emit(\"\\n\");\n}");
            }
        });
        
        ui.horizontal(|ui| {
            if ui.button("📏 Tabla Formateada").clicked() {
                self.insert_snippet("emit(\"Tabla de Valores:\\n\");\nemit(\"i\\ti²\\ti³\\n\");\nemit(\"---\\t---\\t---\\n\");\nchain 1 to 10 {\n    atom cuadrado : atom_num = i * i;\n    atom cubo : atom_num = i * i * i;\n    emit(i, \"\\t\", cuadrado, \"\\t\", cubo, \"\\n\");\n}");
            }
            if ui.button("🎯 Programa Completo").clicked() {
                self.insert_snippet("!! Programa de demostración de bucles\nemit(\"=== DEMOSTRACIÓN DE BUCLES CHAIN ===\\n\\n\");\n\n!! 1. Bucle simple\nemit(\"1. Conteo del 0 al 4:\\n\");\nchain 5 {\n    emit(\"- Iteración \", i, \"\\n\");\n}\n\n!! 2. Bucle con rango\nemit(\"\\n2. Números del 10 al 15:\\n\");\nchain 10 to 15 {\n    emit(\"- Número: \", i, \"\\n\");\n}\n\n!! 3. Bucle descendente\nemit(\"\\n3. Countdown del 5 al 1:\\n\");\nchain 5 to 1 {\n    emit(\"- \", i, \"...\\n\");\n}\n\nemit(\"\\n¡Programa completado!\\n\");");
            }
        });
    }
    
    fn show_functions_snippets(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚙️ Funciones (Reactions)");
        
        if ui.button("⚙️ Función Sin Parámetros").clicked() {
            self.insert_snippet("reaction function_name() {\n    // código de la función\n}");
        }
        
        if ui.button("⚙️ Función Con 1 Parámetro").clicked() {
            self.insert_snippet("reaction function_name(param1: atom_num) {\n    // código de la función\n}");
        }
        
        if ui.button("⚙️ Función Con 2 Parámetros").clicked() {
            self.insert_snippet("reaction function_name(param1: atom_num, param2: formula) {\n    // código de la función\n}");
        }
        
        if ui.button("⚙️ Función Con 3 Parámetros").clicked() {
            self.insert_snippet("reaction function_name(param1: atom_num, param2: mass, param3: formula) {\n    // código de la función\n}");
        }
        
        ui.separator();
        ui.heading("🏗️ Plantillas Comunes");
        
        if ui.button("🧮 Función Calculadora").clicked() {
            self.insert_snippet("reaction calcular(a: atom_num, b: atom_num) {\n    atom resultado : atom_num = a + b;\n    emitln(\"Resultado: \", resultado);\n}");
        }
        
        if ui.button("🔄 Función Procesadora").clicked() {
            self.insert_snippet("reaction procesar(entrada: formula) {\n    emitln(\"Procesando: \", entrada);\n    // lógica de procesamiento aquí\n    emitln(\"Procesamiento completado\");\n}");
        }
    }
    
    fn show_common_snippets(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔧 Patrones Comunes");
        
        if ui.button("⌨️ Programa con Entrada de Usuario").clicked() {
            self.insert_snippet("!! Programa interactivo con capture()\natom nombre : formula = \"\";\natom edad : atom_num = 0;\n\nemitln(\"=== Bienvenido ===\");\nemitln(\"Por favor, ingrese su nombre:\");\ncapture(nombre);\n\nemitln(\"Ingrese su edad:\");\ncapture(edad);\n\nemitln(\"\");\nemitln(\"Hola, \", nombre, \"!\");\nemitln(\"Tienes \", edad, \" años.\");");
        }
        
        if ui.button("🧮 Calculadora Interactiva").clicked() {
            self.insert_snippet("!! Calculadora simple\natom num1 : atom_num = 0;\natom num2 : atom_num = 0;\n\nemitln(\"=== CALCULADORA ===\");\nemitln(\"Ingrese el primer número:\");\ncapture(num1);\n\nemitln(\"Ingrese el segundo número:\");\ncapture(num2);\n\natom suma : atom_num = num1 + num2;\natom resta : atom_num = num1 - num2;\natom mult : atom_num = num1 * num2;\n\nemitln(\"\");\nemitln(\"Resultados:\");\nemitln(\"Suma: \", suma);\nemitln(\"Resta: \", resta);\nemitln(\"Multiplicación: \", mult);");
        }
        
        if ui.button("🔢 Contador Simple").clicked() {
            self.insert_snippet("atom contador : atom_num = 0;\ncontador = contador + 1;\nemitln(\"Contador: \", contador);");
        }
        
        if ui.button("✅ Verificación de Condición").clicked() {
            self.insert_snippet("itest (valor > 0) {\n    emitln(\"Valor positivo\");\n} inotest (valor < 0) {\n    emitln(\"Valor negativo\");\n} notest {\n    emitln(\"Valor es cero\");\n}");
        }
        
        if ui.button("🧮 Operaciones Matemáticas").clicked() {
            self.insert_snippet("atom a : atom_num = 10;\natom b : atom_num = 5;\natom suma : atom_num = a + b;\natom resta : atom_num = a - b;\natom multiplicacion : atom_num = a * b;\nemitln(\"Suma: \", suma);\nemitln(\"Resta: \", resta);\nemitln(\"Multiplicación: \", multiplicacion);");
        }
        
        if ui.button("📝 Manejo de Strings").clicked() {
            self.insert_snippet("atom nombre : formula = \"Usuario\";\natom saludo : formula = \"Hola, \" + nombre + \"!\";\nemitln(saludo);");
        }
        
        if ui.button("🔄 Bucle con Entrada").clicked() {
            self.insert_snippet("!! Bucle interactivo\natom limite : atom_num = 0;\n\nemitln(\"¿Hasta qué número contar?\");\ncapture(limite);\n\nemitln(\"Contando...\");\nchain 1 to limite {\n    emitln(\"Número: \", i);\n}");
        }
        
        if ui.button("🏗️ Programa Básico").clicked() {
            self.insert_snippet("!! Programa básico\natom mensaje : formula = \"¡Hola, mundo!\";\n\nreaction mostrar_mensaje() {\n    emitln(mensaje);\n}\n\n!! Ejecutar\nmostrar_mensaje();");
        }
        
        if ui.button("🔄 Programa Con Lógica").clicked() {
            self.insert_snippet("!! Programa con lógica condicional\natom numero : atom_num = 42;\nion LIMITE : atom_num = 50;\n\nreaction verificar_numero() {\n    itest (numero < LIMITE) {\n        emitln(\"El número está dentro del límite\");\n    } notest {\n        emitln(\"El número excede el límite\");\n    }\n}\n\n!! Ejecutar\nverificar_numero();");
        }
    }
    
    fn insert_snippet(&mut self, snippet: &str) {
        // Si hay texto seleccionado, reemplázalo; si no, inserta al final
        if !self.code.is_empty() && !self.code.ends_with('\n') {
            self.code.push('\n');
        }
        self.code.push_str(snippet);
        self.code.push('\n');
    }
}

impl App for IDE {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.show(ctx);
    }
}

pub fn run() {
    let options = NativeOptions::default();
    eframe::run_native(
        "LenguageProject IDE",
        options,
        Box::new(|_cc| Box::new(IDE::new())),
    ).expect("Error al iniciar la GUI");
}