use crate::{run_lexer, run_parser, run_semantics};
use eframe::App;
use eframe::NativeOptions;
use eframe::egui;

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
                    if ui
                        .add(egui::Button::new("📄 Nuevo").fill(egui::Color32::DARK_GRAY))
                        .clicked()
                    {
                        self.code.clear();
                        self.output.clear();
                    }
                    if ui
                        .add(egui::Button::new("📖 Snippets").fill(egui::Color32::DARK_GRAY))
                        .clicked()
                    {
                        self.show_snippets = !self.show_snippets;
                    }
                    if ui
                        .add(egui::Button::new("⚙️ Opciones").fill(egui::Color32::DARK_GRAY))
                        .clicked()
                    {
                        self.show_options = !self.show_options;
                    }
                    if ui
                        .add(egui::Button::new("❌ Salir").fill(egui::Color32::from_rgb(80, 0, 0)))
                        .clicked()
                    {
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

                        ui.label(
                            egui::RichText::new("EDITOR")
                                .color(egui::Color32::WHITE)
                                .strong(),
                        );
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

                        ui.label(
                            egui::RichText::new("OUTPUT")
                                .color(egui::Color32::WHITE)
                                .strong(),
                        );
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
                                                .desired_width(panel_width - 36.0),
                                        );

                                        // Si está esperando entrada, mostrar prompt estilo terminal
                                        if self.waiting_for_input {
                                            ui.add_space(5.0);

                                            // Mostrar tipo esperado en color cyan
                                            let type_str = match &self.input_var_type {
                                                crate::utils::enums::Ty::AtomNum => "int",
                                                crate::utils::enums::Ty::Mass => "double",
                                                crate::utils::enums::Ty::Polarized => "pos/neg",
                                                crate::utils::enums::Ty::Formula => "string",
                                                crate::utils::enums::Ty::Symbol => "char",
                                                _ => "value",
                                            };

                                            // Prompt estilo terminal con cursor parpadeante
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    egui::RichText::new("> ")
                                                        .color(egui::Color32::from_rgb(255, 255, 0)) // Amarillo para el prompt
                                                        .font(egui::FontId::monospace(14.0))
                                                        .strong(),
                                                );

                                                // Campo de entrada estilo terminal
                                                let response = ui.add(
                                                    egui::TextEdit::singleline(
                                                        &mut self.input_buffer,
                                                    )
                                                    .text_color(egui::Color32::WHITE)
                                                    .font(egui::FontId::monospace(14.0))
                                                    .desired_width(panel_width - 60.0)
                                                    .hint_text(""),
                                                );

                                                // Si presiona Enter, procesar la entrada
                                                if response.lost_focus()
                                                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                                {
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
            Ok(_) => {}
            Err(errors) => {
                let mut err_msg = String::from("Errores semánticos:\n");
                for e in errors {
                    err_msg.push_str(&format!("- {} @ {}:{}\n", e.msg, e.line, e.col));
                }
                return err_msg;
            }
        }

        let (instructions, function_table) = match crate::run_codegen(&ast) {
            Ok(result) => result,
            Err(e) => return format!("Error de generación de código: {}", e),
        };

        // 2. Crear ejecutor y comenzar ejecución
        let mut executor = crate::executor::Executor::new();

        // Registrar funciones en el ejecutor
        for (name, addr) in function_table {
            executor.register_function(name, addr);
        }

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
                    self.executor_state = Some(ExecutorState {
                        executor,
                        instructions,
                    });
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
        executor
            .execute_until_capture(instructions)
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
                        Ok(_) => Err(format!("❌ Error de tipo: '{}'", self.input_var_name)),
                        Err(_) => Err(format!("❌ Error de tipo: '{}'", self.input_var_name)),
                    }
                }
                crate::utils::enums::Ty::Mass => {
                    // Debe ser un número (puede ser decimal)
                    match input_text.parse::<f64>() {
                        Ok(num) => Ok(crate::utils::enums::Value::Number(num)),
                        Err(_) => Err(format!("❌ Error de tipo: '{}'", self.input_var_name)),
                    }
                }
                crate::utils::enums::Ty::Polarized => {
                    // Debe ser pos o neg
                    if input_text.eq_ignore_ascii_case("pos") {
                        Ok(crate::utils::enums::Value::Bool(true))
                    } else if input_text.eq_ignore_ascii_case("neg") {
                        Ok(crate::utils::enums::Value::Bool(false))
                    } else {
                        Err(format!("❌ Error de tipo: '{}'", self.input_var_name))
                    }
                }
                crate::utils::enums::Ty::Symbol => {
                    // Debe ser exactamente un carácter, o un carácter entre comillas simples 'a'
                    let trimmed = input_text.trim();

                    // Si está vacío, error
                    if trimmed.is_empty() {
                        Err(format!(
                            "❌ Error de tipo: '{}' (entrada vacía, se esperaba un carácter)",
                            self.input_var_name
                        ))
                    }
                    // Verificar si está entre comillas simples
                    else if trimmed.starts_with('\'') && trimmed.ends_with('\'') {
                        if trimmed.len() < 3 {
                            // Es '' (vacío)
                            Err(format!(
                                "❌ Error de tipo: '{}' (carácter vacío '')",
                                self.input_var_name
                            ))
                        } else {
                            let inner = &trimmed[1..trimmed.len() - 1];

                            // Manejar escapes como \n, \t, etc.
                            if inner.starts_with('\\') && inner.len() == 2 {
                                match inner.chars().nth(1) {
                                    Some('n') => Ok(crate::utils::enums::Value::Char('\n')),
                                    Some('t') => Ok(crate::utils::enums::Value::Char('\t')),
                                    Some('r') => Ok(crate::utils::enums::Value::Char('\r')),
                                    Some('\\') => Ok(crate::utils::enums::Value::Char('\\')),
                                    Some('\'') => Ok(crate::utils::enums::Value::Char('\'')),
                                    Some(c) => Ok(crate::utils::enums::Value::Char(c)),
                                    None => Err(format!(
                                        "❌ Error de tipo: '{}' (escape inválido)",
                                        self.input_var_name
                                    )),
                                }
                            } else {
                                let chars: Vec<char> = inner.chars().collect();
                                if chars.len() == 1 {
                                    Ok(crate::utils::enums::Value::Char(chars[0]))
                                } else {
                                    Err(format!(
                                        "❌ Error de tipo: '{}' (se esperaba un solo carácter entre comillas simples)",
                                        self.input_var_name
                                    ))
                                }
                            }
                        }
                    } else {
                        // Sin comillas, debe ser un solo carácter
                        let chars: Vec<char> = trimmed.chars().collect();
                        if chars.len() == 1 {
                            Ok(crate::utils::enums::Value::Char(chars[0]))
                        } else {
                            Err(format!(
                                "❌ Error de tipo: '{}' (se esperaba un solo carácter o 'c')",
                                self.input_var_name
                            ))
                        }
                    }
                }
                crate::utils::enums::Ty::Formula => {
                    // Cualquier texto es válido
                    Ok(crate::utils::enums::Value::String(
                        self.input_buffer.trim().to_string(),
                    ))
                }
                _ => {
                    // Para tipos desconocidos, intentar parsear como número o usar como string
                    if let Ok(num) = input_text.parse::<f64>() {
                        Ok(crate::utils::enums::Value::Number(num))
                    } else if input_text.eq_ignore_ascii_case("pos") {
                        Ok(crate::utils::enums::Value::Bool(true))
                    } else if input_text.eq_ignore_ascii_case("neg") {
                        Ok(crate::utils::enums::Value::Bool(false))
                    } else {
                        Ok(crate::utils::enums::Value::String(
                            self.input_buffer.trim().to_string(),
                        ))
                    }
                }
            };

            // Limpiar el buffer de entrada
            self.input_buffer.clear();

            match parse_result {
                Ok(value) => {
                    // Almacenar el valor capturado en el ejecutor
                    state
                        .executor
                        .store_captured_value(&self.input_var_name, value);

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
                    // Error de tipo - terminar ejecución como en C++
                    self.output.push_str(&format!("{}\n", error_msg));
                    self.output
                        .push_str("Programa terminado debido a error de tipo.\n");
                    // Limpiar estado - programa finalizado
                    self.executor_state = None;
                    self.waiting_for_input = false;
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
            ui.selectable_value(
                &mut self.selected_tab,
                SnippetTab::Declarations,
                "📝 Declaraciones",
            );
            ui.selectable_value(&mut self.selected_tab, SnippetTab::Control, "🔀 Control");
            ui.selectable_value(&mut self.selected_tab, SnippetTab::Loops, "🔄 Bucles");
            ui.selectable_value(
                &mut self.selected_tab,
                SnippetTab::Functions,
                "⚙️ Funciones",
            );
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
                self.insert_snippet("atom variable_name : polarized = pos;");
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
            self.insert_snippet("itest (condicion) {\n    !! código aquí\n}");
        }

        if ui.button("🔍 If-Else").clicked() {
            self.insert_snippet("itest (condicion) {\n    !! código si verdadero\n} notest {\n    !! código si falso\n}");
        }

        if ui.button("🔍 If-ElseIf-Else").clicked() {
            self.insert_snippet("itest (condicion1) {\n    !! código condición 1\n} inotest (condicion2) {\n    !! código condición 2\n} notest {\n    !! código por defecto\n}");
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
                self.insert_snippet("chain 3 {\n    emit(\"Iteración: \", loop_1, \"\\n\");\n}");
            }
            if ui.button("⬆️ Bucle Ascendente").clicked() {
                self.insert_snippet("chain 1 to 10 {\n    emit(\"Número: \", loop_1, \"\\n\");\n}");
            }
        });

        ui.horizontal(|ui| {
            if ui.button("⬇️ Bucle Descendente").clicked() {
                self.insert_snippet("chain 10 to 1 {\n    emit(\"Countdown: \", loop_1, \"\\n\");\n}");
            }
            if ui.button("🎯 Bucle Personalizado").clicked() {
                self.insert_snippet("chain 5 to 15 {\n    !! Usar variable 'loop_1' aquí\n    emit(\"Valor actual: \", loop_1, \"\\n\");\n}");
            }
        });

        ui.separator();
        ui.label("🧮 Bucles con Matemáticas:");

        ui.horizontal(|ui| {
            if ui.button("✖️ Tabla de Multiplicar").clicked() {
                self.insert_snippet("chain 1 to 10 {\n    atom resultado : atom_num = 5 * loop_1;\n    emit(\"5 x \", loop_1, \" = \", resultado, \"\\n\");\n}");
            }
            if ui.button("🔢 Potencias").clicked() {
                self.insert_snippet("chain 1 to 5 {\n    atom cuadrado : atom_num = loop_1 * loop_1;\n    atom cubo : atom_num = loop_1 * loop_1 * loop_1;\n    emit(\"loop_1=\", loop_1, \" loop_1²=\", cuadrado, \" loop_1³=\", cubo, \"\\n\");\n}");
            }
        });

        ui.horizontal(|ui| {
            if ui.button("➕ Suma Acumulativa").clicked() {
                self.insert_snippet("atom suma : atom_num = 0;\nchain 1 to 100 {\n    suma = suma + loop_1;\n    emit(\"Suma hasta \", loop_1, \": \", suma, \"\\n\");\n}\nemit(\"Suma total: \", suma, \"\\n\");");
            }
            if ui.button("📊 Factorial").clicked() {
                self.insert_snippet("atom factorial : atom_num = 1;\nchain 1 to 5 {\n    factorial = factorial * loop_1;\n    emit(\"Factorial de \", loop_1, \": \", factorial, \"\\n\");\n}");
            }
        });

        ui.separator();
        ui.label("🎯 Bucles con Condicionales:");

        ui.horizontal(|ui| {
            if ui.button("🔢 Par/Impar").clicked() {
                self.insert_snippet("chain 1 to 20 {\n    itest (loop_1 % 2 == 0) {\n        emit(loop_1, \" es par\\n\");\n    } notest {\n        emit(loop_1, \" es impar\\n\");\n    }\n}");
            }
            if ui.button("🔍 Búsqueda").clicked() {
                self.insert_snippet("atom encontrado : polarized = neg;\nchain 1 to 100 {\n    itest (loop_1 == 42) {\n        emit(\"¡Encontrado en posición: \", loop_1, \"!\\n\");\n        encontrado = pos;\n    }\n}");
            }
        });

        ui.horizontal(|ui| {
            if ui.button("📈 Números Primos").clicked() {
                self.insert_snippet("chain 2 to 50 {\n    atom es_primo : polarized = pos;\n    !! Verificar si loop_1 es primo\n    itest (loop_1 > 2) {\n        atom divisor : atom_num = 2;\n        !! Simplificado: verificar solo algunos divisores\n        itest (loop_1 % divisor == 0) {\n            es_primo = neg;\n        }\n    }\n    itest (es_primo) {\n        emit(loop_1, \" es primo\\n\");\n    }\n}");
            }
            if ui.button("🎲 Filtro de Rango").clicked() {
                self.insert_snippet("chain 1 to 100 {\n    itest (loop_1 >= 10) {\n        itest (loop_1 <= 20) {\n            emit(\"En rango: \", loop_1, \"\\n\");\n        } notest {\n            emit(\"Muy grande: \", loop_1, \"\\n\");\n        }\n    } notest {\n        emit(\"Muy pequeño: \", loop_1, \"\\n\");\n    }\n}");
            }
        });

        ui.separator();
        ui.label("🎨 Bucles de Patrones:");

        ui.horizontal(|ui| {
            if ui.button("📐 Triángulo").clicked() {
                self.insert_snippet("!! Patrón de triángulo usando loop_1\nchain 1 to 5 {\n    atom contador : atom_num = 0;\n    chain 1 to 10 {\n        itest (loop_2 <= loop_1) {\n            emit(\"* \");\n            contador = contador + 1;\n        }\n    }\n    emit(\"\\n\");\n}");
            }
            if ui.button("🔶 Triángulo Invertido").clicked() {
                self.insert_snippet("!! Patrón de triángulo invertido\nchain 5 to 1 {\n    atom contador : atom_num = 0;\n    chain 1 to 10 {\n        itest (loop_2 <= loop_1) {\n            emit(\"* \");\n            contador = contador + 1;\n        }\n    }\n    emit(\"\\n\");\n}");
            }
        });

        ui.horizontal(|ui| {
            if ui.button("🔴 Círculos").clicked() {
                self.insert_snippet("!! Patrón de círculos usando loop_1\nchain 1 to 5 {\n    emit(\"Círculo \", loop_1, \": \");\n    atom contador : atom_num = 0;\n    chain 1 to 10 {\n        itest (loop_2 <= loop_1) {\n            emit(\"O \");\n            contador = contador + 1;\n        }\n    }\n    emit(\"\\n\");\n}");
            }
            if ui.button("⬛ Cuadrado").clicked() {
                self.insert_snippet("!! Patrón de cuadrado 5x5\nion TAMAÑO : atom_num = 5;\n\nchain 1 to TAMAÑO {\n    chain 1 to TAMAÑO {\n        emit(\"■ \");\n    }\n    emit(\"\\n\");\n}");
            }
        });

        ui.horizontal(|ui| {
            if ui.button("🎄 Árbol de Navidad").clicked() {
                self.insert_snippet("!! Patrón de árbol con bucles y loop_1\nchain 1 to 4 {\n    !! Imprimir espacios\n    atom espacios_total : atom_num = 4 - loop_1;\n    atom espacio_count : atom_num = 0;\n    chain 1 to 10 {\n        itest (loop_2 <= espacios_total) {\n            emit(\" \");\n            espacio_count = espacio_count + 1;\n        }\n    }\n    \n    !! Imprimir estrellas\n    atom estrellas_total : atom_num = (loop_1 * 2) - 1;\n    atom estrella_count : atom_num = 0;\n    chain 1 to 10 {\n        itest (loop_2 <= estrellas_total) {\n            emit(\"*\");\n            estrella_count = estrella_count + 1;\n        }\n    }\n    emit(\"\\n\");\n}\n\n!! Tronco\nemit(\"   |\\n\");\nemit(\"   |\\n\");");
            }
            if ui.button("📏 Tabla Formateada").clicked() {
                self.insert_snippet("emit(\"Tabla de Valores:\\n\");\nemit(\"loop_1\\tloop_1²\\tloop_1³\\n\");\nemit(\"---\\t---\\t---\\n\");\nchain 1 to 10 {\n    atom cuadrado : atom_num = loop_1 * loop_1;\n    atom cubo : atom_num = loop_1 * loop_1 * loop_1;\n    emit(loop_1, \"\\t\", cuadrado, \"\\t\", cubo, \"\\n\");\n}");
            }
            if ui.button("🎯 Programa Completo").clicked() {
                self.insert_snippet("!! Programa de demostración de bucles\nemit(\"=== DEMOSTRACIÓN DE BUCLES CHAIN ===\\n\\n\");\n\n!! 1. Bucle simple\nemit(\"1. Conteo del 0 al 4:\\n\");\nchain 5 {\n    emit(\"- Iteración \", loop_1, \"\\n\");\n}\n\n!! 2. Bucle con rango\nemit(\"\\n2. Números del 10 al 15:\\n\");\nchain 10 to 15 {\n    emit(\"- Número: \", loop_1, \"\\n\");\n}\n\n!! 3. Bucle descendente\nemit(\"\\n3. Countdown del 5 al 1:\\n\");\nchain 5 to 1 {\n    emit(\"- \", loop_1, \"...\\n\");\n}\n\nemit(\"\\n¡Programa completado!\\n\");");
            }
        });
    }

    fn show_functions_snippets(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚙️ Funciones (Reactions)");

        if ui.button("⚙️ Función Sin Parámetros").clicked() {
            self.insert_snippet("reaction function_name() : void_state {\n    !! código de la función\n    emitln(\"Función ejecutada\");\n}");
        }

        if ui.button("⚙️ Función Con 1 Parámetro").clicked() {
            self.insert_snippet("reaction function_name(param1: atom_num) : void_state {\n    !! código de la función\n    emitln(\"Parámetro: \", param1);\n}");
        }

        if ui.button("⚙️ Función Con 2 Parámetros").clicked() {
            self.insert_snippet("reaction function_name(param1: atom_num, param2: formula) : void_state {\n    !! código de la función\n    emitln(\"Parámetros: \", param1, \", \", param2);\n}");
        }

        if ui.button("⚙️ Función Con 3 Parámetros").clicked() {
            self.insert_snippet("reaction function_name(param1: atom_num, param2: mass, param3: formula) : void_state {\n    !! código de la función\n    emitln(\"Parámetros: \", param1, \", \", param2, \", \", param3);\n}");
        }

        ui.separator();
        ui.heading("🏗️ Plantillas Comunes");

        if ui.button("🧮 Función Calculadora").clicked() {
            self.insert_snippet("reaction calcular(a: atom_num, b: atom_num) : atom_num {\n    atom resultado : atom_num = a + b;\n    release resultado;\n}");
        }

        if ui.button("🔄 Función Procesadora").clicked() {
            self.insert_snippet("reaction procesar(entrada: formula) : void_state {\n    emitln(\"Procesando: \", entrada);\n    !! lógica de procesamiento aquí\n    emitln(\"Procesamiento completado\");\n}");
        }

        if ui.button("🔢 Función Retorna Número").clicked() {
            self.insert_snippet(
                "reaction sumar(x: atom_num, y: atom_num) : atom_num {\n    release x + y;\n}",
            );
        }

        if ui.button("📝 Función Retorna Texto").clicked() {
            self.insert_snippet("reaction saludar(nombre: formula) : formula {\n    release \"Hola, \" + nombre + \"!\";\n}");
        }

        if ui.button("✅ Función Retorna Boolean").clicked() {
            self.insert_snippet("reaction es_positivo(num: atom_num) : polarized {\n    itest (num > 0) {\n        release pos;\n    } notest {\n        release neg;\n    }\n}");
        }

        ui.separator();
        ui.heading("📞 Llamadas a Funciones");

        ui.horizontal(|ui| {
            if ui.button("📞 Llamar Función Simple").clicked() {
                self.insert_snippet("!! Llamar una función\nfunction_name();");
            }
            if ui.button("📞 Llamar con 1 Parámetro").clicked() {
                self.insert_snippet("function_name(arg1);");
            }
        });

        ui.horizontal(|ui| {
            if ui.button("📞 Llamar con 2 Parámetros").clicked() {
                self.insert_snippet("function_name(arg1, arg2);");
            }
            if ui.button("📞 Llamar con 3 Parámetros").clicked() {
                self.insert_snippet("function_name(arg1, arg2, arg3);");
            }
        });

        ui.separator();
        ui.label("📦 Asignar Valor de Retorno:");

        ui.horizontal(|ui| {
            if ui.button("🔢 Guardar Retorno Numérico").clicked() {
                self.insert_snippet("atom resultado : atom_num = sumar(5, 3);");
            }
            if ui.button("📝 Guardar Retorno Texto").clicked() {
                self.insert_snippet("atom mensaje : formula = saludar(\"Mundo\");");
            }
        });

        ui.horizontal(|ui| {
            if ui.button("✅ Guardar Retorno Boolean").clicked() {
                self.insert_snippet("atom es_valido : polarized = verificar(valor);");
            }
            if ui.button("🎯 Usar en Expresión").clicked() {
                self.insert_snippet("atom total : atom_num = calcular(a, b) + 10;");
            }
        });

        ui.separator();
        ui.heading("🎯 Ejemplos Completos");

        if ui.button("🧮 Ejemplo: Función Suma").clicked() {
            self.insert_snippet("!! Definir función\nreaction sumar(x: atom_num, y: atom_num) : atom_num {\n    release x + y;\n}\n\n!! Usar la función\natom resultado : atom_num = sumar(5, 3);\nemitln(\"5 + 3 = \", resultado);");
        }

        if ui.button("📝 Ejemplo: Función Saludo").clicked() {
            self.insert_snippet("!! Definir función\nreaction saludar(nombre: formula) : formula {\n    release \"Hola, \" + nombre + \"!\";\n}\n\n!! Usar la función\natom mensaje : formula = saludar(\"Mundo\");\nemitln(mensaje);");
        }

        if ui.button("✅ Ejemplo: Función Comparación").clicked() {
            self.insert_snippet("!! Definir función\nreaction es_mayor(a: atom_num, b: atom_num) : polarized {\n    itest (a > b) {\n        release pos;\n    } notest {\n        release neg;\n    }\n}\n\n!! Usar la función\natom check : polarized = es_mayor(10, 5);\nitest (check) {\n    emitln(\"10 es mayor que 5\");\n} notest {\n    emitln(\"10 NO es mayor que 5\");\n}");
        }

        ui.separator();
        ui.heading("🎓 Funciones Avanzadas");

        if ui.button("🔢 Factorial").clicked() {
            self.insert_snippet("!! Función para calcular factorial (ahora funciona!)\nreaction factorial_n(n: atom_num) : atom_num {\n    atom resultado : atom_num = 1;\n    atom contador : atom_num = 0;\n    chain 1 to 100 {\n        itest (loop_1 <= n) {\n            resultado = resultado * loop_1;\n            contador = contador + 1;\n        }\n    }\n    release resultado;\n}\n\n!! Uso\natom fact5 : atom_num = factorial_n(5);\nemitln(\"5! = \", fact5);\n\natom fact7 : atom_num = factorial_n(7);\nemitln(\"7! = \", fact7);");
        }

        if ui.button("🔢 Potencia").clicked() {
            self.insert_snippet("!! Función para calcular potencia (ahora funciona!)\nreaction potencia(base: atom_num, exponente: atom_num) : atom_num {\n    atom resultado : atom_num = 1;\n    atom contador : atom_num = 0;\n    chain 1 to 100 {\n        itest (loop_1 <= exponente) {\n            resultado = resultado * base;\n            contador = contador + 1;\n        }\n    }\n    release resultado;\n}\n\n!! Uso\natom pot1 : atom_num = potencia(2, 8);\nemitln(\"2^8 = \", pot1);\n\natom pot2 : atom_num = potencia(3, 4);\nemitln(\"3^4 = \", pot2);");
        }

        if ui.button("✅ Es Par").clicked() {
            self.insert_snippet("!! Función para verificar si es par\nreaction es_par(num: atom_num) : polarized {\n    itest (num % 2 == 0) {\n        release pos;\n    } notest {\n        release neg;\n    }\n}\n\n!! Uso\natom check : polarized = es_par(42);\nitest (check) {\n    emitln(\"42 es par\");\n} notest {\n    emitln(\"42 es impar\");\n}");
        }

        if ui.button("🔢 Valor Absoluto").clicked() {
            self.insert_snippet("!! Función para obtener valor absoluto\nreaction abs(num: atom_num) : atom_num {\n    itest (num < 0) {\n        release num * -1;\n    } notest {\n        release num;\n    }\n}\n\n!! Uso\natom val1 : atom_num = abs(-42);\natom val2 : atom_num = abs(15);\nemitln(\"abs(-42) = \", val1);\nemitln(\"abs(15) = \", val2);");
        }

        if ui.button("🔢 Máximo de 2 Números").clicked() {
            self.insert_snippet("!! Función para obtener el máximo\nreaction maximo(a: atom_num, b: atom_num) : atom_num {\n    itest (a > b) {\n        release a;\n    } notest {\n        release b;\n    }\n}\n\n!! Uso\natom max : atom_num = maximo(10, 20);\nemitln(\"El máximo entre 10 y 20 es: \", max);");
        }

        if ui.button("🔢 Rango Numérico").clicked() {
            self.insert_snippet("!! Función para verificar rango\nreaction en_rango(valor: atom_num, min: atom_num, max: atom_num) : polarized {\n    itest (valor >= min) {\n        itest (valor <= max) {\n            release pos;\n        } notest {\n            release neg;\n        }\n    } notest {\n        release neg;\n    }\n}\n\n!! Uso\natom check : polarized = en_rango(50, 1, 100);\nitest (check) {\n    emitln(\"50 está en el rango [1, 100]\");\n}");
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
            self.insert_snippet("!! Inicializar contador\natom contador : atom_num = 0;\n\n!! Incrementar\ncontador = contador + 1;\nemitln(\"Contador: \", contador);\n\n!! Incrementar de nuevo\ncontador = contador + 1;\nemitln(\"Contador: \", contador);");
        }

        if ui.button("✅ Verificación de Condición").clicked() {
            self.insert_snippet("!! Verificar condiciones\natom valor : atom_num = 42;\n\nitest (valor > 0) {\n    emitln(\"Valor positivo\");\n} inotest (valor < 0) {\n    emitln(\"Valor negativo\");\n} notest {\n    emitln(\"Valor es cero\");\n}");
        }

        if ui.button("🔍 Comparaciones Lógicas").clicked() {
            self.insert_snippet("!! Operadores de comparación\natom a : atom_num = 10;\natom b : atom_num = 5;\n\n!! Comparaciones básicas\nitest (a > b) {\n    emitln(\"a es mayor que b\");\n}\n\nitest (a >= b) {\n    emitln(\"a es mayor o igual que b\");\n}\n\nitest (a == b) {\n    emitln(\"a es igual a b\");\n} notest {\n    emitln(\"a es diferente de b\");\n}\n\n!! Lógica AND con if anidado\nitest (a > 0) {\n    itest (b > 0) {\n        emitln(\"Ambos son positivos\");\n    }\n}\n\n!! Lógica OR con variables auxiliares\natom condicion1 : polarized = neg;\natom condicion2 : polarized = neg;\n\nitest (a > 100) {\n    condicion1 = pos;\n}\nitest (b > 100) {\n    condicion2 = pos;\n}\n\nitest (condicion1) {\n    emitln(\"Al menos uno es mayor que 100\");\n} inotest (condicion2) {\n    emitln(\"Al menos uno es mayor que 100\");\n}");
        }

        if ui.button("🧮 Operaciones Matemáticas").clicked() {
            self.insert_snippet("!! Operaciones matemáticas básicas\natom a : atom_num = 10;\natom b : atom_num = 5;\n\natom suma : atom_num = a + b;\natom resta : atom_num = a - b;\natom mult : atom_num = a * b;\natom modulo : atom_num = a % b;\n\nemitln(\"a = \", a, \", b = \", b);\nemitln(\"Suma: \", suma);\nemitln(\"Resta: \", resta);\nemitln(\"Multiplicación: \", mult);\nemitln(\"Módulo: \", modulo);");
        }

        if ui.button("📝 Manejo de Strings").clicked() {
            self.insert_snippet("!! Manejo de cadenas de texto\natom nombre : formula = \"Usuario\";\natom apellido : formula = \"ElementScript\";\n\n!! Concatenación\natom nombre_completo : formula = nombre + \" \" + apellido;\natom saludo : formula = \"Hola, \" + nombre_completo + \"!\";\n\nemitln(saludo);\nemitln(\"Nombre: \", nombre);\nemitln(\"Apellido: \", apellido);");
        }

        if ui.button("🔄 Bucle con Entrada").clicked() {
            self.insert_snippet("!! Bucle interactivo\natom limite : atom_num = 0;\n\nemitln(\"¿Hasta qué número contar?\");\ncapture(limite);\n\nemitln(\"Contando...\");\nchain 1 to limite {\n    emitln(\"Número: \", loop_1);\n}");
        }

        if ui.button("🏗️ Programa Básico").clicked() {
            self.insert_snippet("!! Programa básico\natom mensaje : formula = \"¡Hola, mundo!\";\n\nreaction mostrar_mensaje() : void_state {\n    emitln(mensaje);\n}\n\n!! Ejecutar\nmostrar_mensaje();");
        }

        if ui.button("🔄 Programa Con Lógica").clicked() {
            self.insert_snippet("!! Programa con lógica condicional\natom numero : atom_num = 42;\nion LIMITE : atom_num = 50;\n\nreaction verificar_numero() : void_state {\n    itest (numero < LIMITE) {\n        emitln(\"El número está dentro del límite\");\n    } notest {\n        emitln(\"El número excede el límite\");\n    }\n}\n\n!! Ejecutar\nverificar_numero();");
        }

        ui.separator();
        ui.heading("🎯 Programas Completos");

        if ui.button("🎮 Juego de Adivinanza").clicked() {
            self.insert_snippet("!! Juego: Adivina el número\nion NUMERO_SECRETO : atom_num = 42;\natom intento : atom_num = 0;\natom intentos_restantes : atom_num = 3;\n\nemitln(\"=== ADIVINA EL NÚMERO ===\");\nemitln(\"Tienes 3 intentos para adivinar un número entre 1 y 100\");\nemitln(\"\");\n\nchain 1 to 3 {\n    emitln(\"Intento \", loop_1, \" de 3:\");\n    capture(intento);\n    \n    itest (intento == NUMERO_SECRETO) {\n        emitln(\"\\n¡FELICIDADES! ¡Adivinaste el número!\");\n    } inotest (intento < NUMERO_SECRETO) {\n        emitln(\"Muy bajo... Intenta de nuevo\\n\");\n    } notest {\n        emitln(\"Muy alto... Intenta de nuevo\\n\");\n    }\n}\n\nemitln(\"Fin del juego. El número era: \", NUMERO_SECRETO);");
        }

        if ui.button("📊 Estadísticas de Lista").clicked() {
            self.insert_snippet("!! Calcular estadísticas de números\natom suma : atom_num = 0;\natom contador : atom_num = 0;\natom mayor : atom_num = 0;\natom menor : atom_num = 999999;\n\nemitln(\"=== ESTADÍSTICAS ===\");\nemitln(\"Analizando números del 1 al 10...\");\n\nchain 1 to 10 {\n    atom valor : atom_num = loop_1;\n    suma = suma + valor;\n    contador = contador + 1;\n    \n    itest (valor > mayor) {\n        mayor = valor;\n    }\n    \n    itest (valor < menor) {\n        menor = valor;\n    }\n}\n\natom promedio : atom_num = suma / contador;\n\nemitln(\"\");\nemitln(\"Resultados:\");\nemitln(\"- Cantidad: \", contador);\nemitln(\"- Suma total: \", suma);\nemitln(\"- Promedio: \", promedio);\nemitln(\"- Mayor: \", mayor);\nemitln(\"- Menor: \", menor);");
        }

        if ui.button("🔢 Tabla de Multiplicar Completa").clicked() {
            self.insert_snippet("!! Tabla de multiplicar completa\natom tabla : atom_num = 0;\n\nemitln(\"=== TABLA DE MULTIPLICAR ===\");\nemitln(\"¿Qué tabla quieres ver?\");\ncapture(tabla);\n\nemitln(\"\");\nemitln(\"Tabla del \", tabla, \":\");\nemitln(\"-------------------\");\n\nchain 1 to 10 {\n    atom resultado : atom_num = tabla * loop_1;\n    emitln(tabla, \" x \", loop_1, \" = \", resultado);\n}");
        }

        if ui.button("🌡️ Conversor de Temperatura").clicked() {
            self.insert_snippet("!! Conversor Celsius a Fahrenheit\natom celsius : mass = 0.0;\n\nemitln(\"=== CONVERSOR DE TEMPERATURA ===\");\nemitln(\"Ingrese temperatura en Celsius:\");\ncapture(celsius);\n\n!! Fórmula: F = C * 9/5 + 32\natom fahrenheit : mass = (celsius * 9.0 / 5.0) + 32.0;\n\nemitln(\"\");\nemitln(celsius, \"°C = \", fahrenheit, \"°F\");");
        }

        if ui.button("💰 Calculadora de Propinas").clicked() {
            self.insert_snippet("!! Calculadora de propinas\natom cuenta : mass = 0.0;\natom porcentaje : atom_num = 0;\n\nemitln(\"=== CALCULADORA DE PROPINAS ===\");\nemitln(\"Ingrese el total de la cuenta:\");\ncapture(cuenta);\n\nemitln(\"Ingrese el porcentaje de propina (10, 15, 20):\");\ncapture(porcentaje);\n\natom propina : mass = cuenta * porcentaje / 100.0;\natom total : mass = cuenta + propina;\n\nemitln(\"\");\nemitln(\"Cuenta: $\", cuenta);\nemitln(\"Propina (\", porcentaje, \"%): $\", propina);\nemitln(\"Total a pagar: $\", total);");
        }

        if ui.button("📅 Validador de Rango").clicked() {
            self.insert_snippet("!! Validar si un número está en rango\natom numero : atom_num = 0;\nion MIN : atom_num = 1;\nion MAX : atom_num = 100;\n\nemitln(\"=== VALIDADOR DE RANGO ===\");\nemitln(\"Ingrese un número entre \", MIN, \" y \", MAX, \":\");\ncapture(numero);\n\nemitln(\"\");\nitest (numero < MIN) {\n    emitln(\"✗ Número demasiado pequeño: \", numero);\n    emitln(\"Debe ser mayor o igual a \", MIN);\n} inotest (numero > MAX) {\n    emitln(\"✗ Número demasiado grande: \", numero);\n    emitln(\"Debe ser menor o igual a \", MAX);\n} notest {\n    emitln(\"✓ Número válido: \", numero);\n    emitln(\"El número está dentro del rango permitido.\");\n}");
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
    )
    .expect("Error al iniciar la GUI");
}
