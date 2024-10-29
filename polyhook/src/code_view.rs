use egui_extras::syntax_highlighting::{highlight, CodeTheme};

use crate::model::{pattern_model::model_from_pattern, ModelData};

pub struct CodeView {
    code: String,
}

impl Default for CodeView {
    fn default() -> Self {
        Self {
            code: r#"new_row();
for _c in 1..=15 {
    chain();
}
for _r in 1..=15 {
    new_row();
    for _s in 1..=15 {
        dc();
    }
}"#.into()
        }
    }
}

impl CodeView {
    pub fn code_view_show(&mut self, ui: &mut egui::Ui) -> Option<ModelData> {
        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = highlight(ui.ctx(), ui.style(), &CodeTheme::from_style(ui.style()), string, "rs");
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        let model = egui::Frame::default()
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
                                .hint_text("Type your code here...")
                        );
                    });

                let button = ui.add_sized(ui.available_size(), egui::Button::new("Render"));
                if button.clicked() {
                    let pattern = hooklib::script::PatternScript::eval_script(&self.code);
                    match pattern {
                        Ok(pattern) => {
                            Some(model_from_pattern(&pattern))
                        }
                        Err(err) => { eprintln!("{:?}", err); None },
                    }
                } else { None }
            });

        model.inner
    }
}
