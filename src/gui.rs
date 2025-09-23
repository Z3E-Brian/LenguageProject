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
}

impl IDE {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn show(&mut self, ctx: &egui::Context) {
        // Aplicar tema claro si está activado
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
        // Reutilizar la función de compilación para validar el código
        let compile_result = self.compilar();
        if compile_result.starts_with("Error") {
            return compile_result;
        }
        // Simular ejecución (puedes personalizar esto según tu lenguaje)
        "Ejecución completada. Salida simulada.".into()
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