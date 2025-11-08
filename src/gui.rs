use crate::{run_lexer, run_parser, run_semantics};
use eframe::egui;
use eframe::NativeOptions;
use eframe::App;

#[derive(Default)]
pub struct IDE {
    pub code: String,
    pub output: String,
    pub light_mode: bool,
    pub show_options: bool,
    pub show_snippets: bool,
    pub selected_tab: SnippetTab,
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
            egui::Window::new("🧩 Snippets de Código")
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
                // Distribución en dos columnas
                ui.columns(2, |cols| {
                    // -------- IZQUIERDA: EDITOR --------
                    cols[0].vertical(|ui| {
                        ui.label(egui::RichText::new("EDITOR").color(egui::Color32::WHITE).strong());
                        // Caja del editor con un gris oscuro para diferenciar del fondo
                        egui::Frame::group(ui.style())
                            .fill(egui::Color32::from_rgb(30, 30, 30))
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(80)))
                            .show(ui, |ui| {
                                let h = ui.available_height() - 12.0; // reservar un poco de margen
                                ui.add_sized(
                                    [ui.available_width(), h],
                                    egui::TextEdit::multiline(&mut self.code)
                                        .code_editor()
                                        .text_color(egui::Color32::WHITE)
                                        .hint_text("Escribe tu código aquí..."),
                                );
                            });
                    });

                    // -------- DERECHA: OUTPUT + BOTONES DEBAJO --------
                    cols[1].vertical(|ui| {
                        ui.label(egui::RichText::new("OUTPUT").color(egui::Color32::WHITE).strong());

                        egui::Frame::group(ui.style())
                            .fill(egui::Color32::from_rgb(30, 30, 30))
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(80)))
                            .show(ui, |ui| {
                                // Área de salida (solo lectura)
                                let h = ui.available_height() - 58.0; // dejar espacio para botones
                                ui.add_sized(
                                    [ui.available_width(), h],
                                    egui::TextEdit::multiline(&mut self.output)
                                        .interactive(false)
                                        .text_color(egui::Color32::WHITE)
                                        .desired_rows(12),
                                );
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

    fn ejecutar(&self) -> String {
        if self.code.trim().is_empty() {
            return "Error: archivo vacío".into();
        }

        // Usar el nuevo pipeline completo de compilación y ejecución
        match crate::compile_and_execute(&self.code) {
            Ok(output) => {
                if output.is_empty() {
                    "Programa ejecutado sin salida.".into()
                } else {
                    output
                }
            }
            Err(error) => error,
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