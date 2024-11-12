use std::{error::Error, thread::{spawn, JoinHandle}};

use egui::{Color32, TextStyle};
use egui_extras::syntax_highlighting::{highlight, CodeTheme};
use hooklib::examples;

use crate::model::{pattern_model::model_from_pattern, ModelData};

pub struct CodeView {
    code: String,
    err: Option<Box<dyn Error + Send + Sync>>,
    thread: Option<JoinHandle<Result<ModelData, Box<dyn Error + Send + Sync>>>>,
}

impl Default for CodeView {
    fn default() -> Self {
        Self {
            code: examples::EXAMPLE_SPIRAL_ROUNDS.into(),
            err: None,
            thread: None,
        }
    }
}

impl CodeView {
    pub fn load_code(&mut self, code: &str) {
        self.code = code.into();
    }

    fn start_render(&mut self) {
        let code = self.code.clone();
        self.thread = Some(spawn(move || {
            let pattern = hooklib::script::PatternScript::eval_script(&code);
            match pattern {
                Ok(pattern) => Ok(model_from_pattern(&pattern)),
                Err(err) => Err(err),
            }
        }));
    }

    fn check_render(&mut self) -> Option<Result<ModelData, Box<dyn Error + Send + Sync>>> {
        if self.thread.as_ref().is_some_and(|t| t.is_finished()) {
            Some(self
                .thread
                .take()
                .unwrap()
                .join()
                .expect("Failed to join thread."))
        } else {
            None
        }
    }

    pub fn code_view_show(&mut self, ui: &mut egui::Ui) -> Option<ModelData> {
        egui::Frame::default()
            .fill(ui.visuals().extreme_bg_color)
            .stroke(ui.visuals().window_stroke)
            .rounding(ui.visuals().window_rounding)
            .show(ui, |ui| {
                if let Some(err) = &self.err {
                    let err_str = format!("{err}");

                    ui
                        .colored_label(Color32::RED, err_str);
                }

                egui::ScrollArea::vertical()
                    .max_height(ui.available_height() - 50.0)
                    .show(ui, |ui| {
                        let code_rows = if self.code.ends_with('\n') || self.code.is_empty() {
                            self.code.lines().count() + 1
                        } else {
                            self.code.lines().count()
                        };
                        let max_length = (code_rows.ilog10() + 1) as usize;

                        let mut linenums = (1..=code_rows)
                            .map(|i| {
                                let num = i.to_string();
                                let n_spaces = max_length - num.len();
                                let spaces = " ".repeat(n_spaces);
                                format!("{spaces}{num}")
                            })
                            .collect::<Vec<_>>()
                            .join("\n");

                        ui.horizontal_top(|ui| {
                            let mut linenums_layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                                let mut layout_job = egui::text::LayoutJob::single_section(
                                    string.to_string(),
                                    egui::TextFormat::simple(
                                        egui::FontId::monospace(TextStyle::Monospace.resolve(ui.style()).size),
                                        Color32::LIGHT_GRAY
                                    ),
                                );
                                layout_job.wrap.max_width = f32::INFINITY;
                                ui.fonts(|f| f.layout_job(layout_job))
                            };

                            ui.add(
                                egui::TextEdit::multiline(&mut linenums)
                                    .font(TextStyle::Monospace)
                                    .interactive(false)
                                    .frame(false)
                                    .desired_rows(code_rows)
                                    .desired_width((max_length as f32) * TextStyle::Monospace.resolve(ui.style()).size * 0.65)
                                    .layouter(&mut linenums_layouter)
                            );

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

                            ui.add_sized(
                                ui.available_size(),
                                egui::TextEdit::multiline(&mut self.code)
                                    .font(TextStyle::Monospace)
                                    .frame(false)
                                    .lock_focus(true)
                                    .layouter(&mut layouter)
                                    .hint_text("Type your code here..."),
                            );
                        })
                    });

                ui.add_enabled_ui(self.thread.as_ref().is_none_or(|t| t.is_finished()), |ui| {
                    let button = ui.add_sized(ui.available_size(), egui::Button::new("Render"));
                    if button.clicked() {
                        self.err = None;
                        self.start_render();
                    }
                });
            });

        match self.check_render() {
            Some(Ok(model)) => Some(model),
            Some(Err(err)) => {
                self.err = Some(err);
                None
            },
            _ => None,
        }
    }
}
