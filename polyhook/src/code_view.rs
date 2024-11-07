use std::thread::{spawn, JoinHandle};

use egui_extras::syntax_highlighting::{highlight, CodeTheme};

use crate::model::{pattern_model::model_from_pattern, ModelData};

pub struct CodeView {
    code: String,
    thread: Option<JoinHandle<Option<ModelData>>>,
}

impl Default for CodeView {
    fn default() -> Self {
        Self {
            code: r#"
15 # chain;
15 # || {
    turn();
    15 # dc;
}
            "#
            .into(),
            thread: None,
        }
    }
}

impl CodeView {
    pub fn code_view_show(&mut self, ui: &mut egui::Ui) -> Option<ModelData> {
        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = highlight(
                ui.ctx(),
                ui.style(),
                &CodeTheme::from_style(ui.style()),
                string,
                "rs",
            );
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        egui::Frame::default()
            .fill(ui.visuals().extreme_bg_color)
            .stroke(ui.visuals().window_stroke)
            .rounding(ui.visuals().window_rounding)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(ui.available_height() - 50.0)
                    .show(ui, |ui| {
                        ui.add_sized(
                            ui.available_size(),
                            egui::TextEdit::multiline(&mut self.code)
                                .font(egui::TextStyle::Monospace)
                                .frame(false)
                                .lock_focus(true)
                                .layouter(&mut layouter)
                                .hint_text("Type your code here..."),
                        );
                    });

                ui.add_enabled_ui(self.thread.as_ref().is_none_or(|t| t.is_finished()), |ui| {
                    let button = ui.add_sized(ui.available_size(), egui::Button::new("Render"));
                    if button.clicked() {
                        let code = self.code.clone();
                        self.thread = Some(spawn(move || {
                            let pattern = hooklib::script::PatternScript::eval_script(&code);
                            match pattern {
                                Ok(pattern) => Some(model_from_pattern(&pattern)),
                                Err(err) => {
                                    eprintln!("{:?}", err);
                                    None
                                }
                            }
                        }));
                    }
                });
            });

        if self.thread.as_ref().is_some_and(|t| t.is_finished()) {
            let model = self.thread.take().unwrap().join()
                .expect("Failed to join thread.")
                .expect("Error in creating model.");
            Some(model)
        } else {
            None
        }
    }
}
