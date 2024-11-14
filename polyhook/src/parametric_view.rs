use std::{error::Error, thread::{spawn, JoinHandle}};

use egui::{Color32, RichText, TextStyle, Widget};
use egui_extras::syntax_highlighting::{highlight, CodeTheme};
use hooklib::{examples, parametric::{example_flat, Operation, ParametricPattern}};

use crate::model::{pattern_model::model_from_pattern, ModelData};

pub struct ParametricView {
    pattern: ParametricPattern<'static>
}

impl Default for ParametricView {
    fn default() -> Self {
        Self {
            pattern: example_flat(),
        }
    }
}

impl Widget for &mut ParametricView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.set_min_size(ui.available_size());
        egui::Frame::default()
            .show(ui, |ui| {
                ui.label(RichText::new("Parametric View").strong());
                ParametricView::operation_ui(ui, &mut self.pattern);
            })
            .response
    }
}

impl ParametricView {
    fn operation_ui(ui: &mut egui::Ui, pattern: &mut Operation) {
        match pattern {
            Operation::Seq(vec) => {
                vec.iter_mut()
                    .for_each(|op| {
                        ParametricView::operation_ui(ui, op);
                    });
            },
            Operation::Define(identifier, operation) => {
                ui.horizontal(|ui| {
                    ui.label(format!("define {identifier} as"));
                    ParametricView::operation_ui(ui, operation);
                });
            },
            Operation::Literal(value) => {
                ui.add(egui::DragValue::new(value));
            },
            Operation::Variable(identifier) => {
                // TODO: fix shared id_salt issue
                // egui::ComboBox::from_id_salt(identifier.to_string())
                //     .selected_text(identifier.to_string())
                //     .show_ui(ui, |ui| {
                //         ui.selectable_value(identifier, identifier.clone(), identifier.to_string());
                //     });
                ui.label(identifier.to_string());
            },
            Operation::Call(identifier) => {
                ui.label(format!("{identifier}"));
            },
            Operation::Repeat(n, op) => {
                ui.horizontal(|ui| {
                    ui.label("do");
                    ParametricView::operation_ui(ui, n);
                    ui.label("times:");
                });
                ui.indent(0, |ui| {
                    ParametricView::operation_ui(ui, op);
                });
            },
        }
    }
}