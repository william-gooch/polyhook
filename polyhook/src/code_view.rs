use egui::{Color32, TextStyle};
use egui_extras::syntax_highlighting::{highlight, CodeTheme};
use hooklib::examples;

pub struct CodeView {
    pub code: String,
}

impl Default for CodeView {
    fn default() -> Self {
        Self {
            code: examples::EXAMPLE_FLAT.into(),
        }
    }
}

impl CodeView {
    pub fn load_code(&mut self, code: &str) {
        self.code = code.into();
    }

    pub fn code_view_show(&mut self, ui: &mut egui::Ui) {
        egui::Frame::default()
            .fill(ui.visuals().extreme_bg_color)
            .stroke(ui.visuals().window_stroke)
            .rounding(ui.visuals().window_rounding)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(ui.available_height() - 100.0)
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
                            let mut linenums_layouter =
                                |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                                    let mut layout_job = egui::text::LayoutJob::single_section(
                                        string.to_string(),
                                        egui::TextFormat::simple(
                                            egui::FontId::monospace(
                                                TextStyle::Monospace.resolve(ui.style()).size,
                                            ),
                                            Color32::LIGHT_GRAY,
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
                                    .desired_width(
                                        (max_length as f32)
                                            * TextStyle::Monospace.resolve(ui.style()).size
                                            * 0.65,
                                    )
                                    .layouter(&mut linenums_layouter),
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
            });
    }
}
