use std::collections::HashMap;

use egui::{Layout, Widget};
use egui_extras::{Column, TableBuilder};
use hooklib::script::{PatternScript, Script};
use rhai::{Dynamic, ImmutableString};

#[derive(Default)]
pub struct ParameterView {
    pub parameters: HashMap<ImmutableString, Dynamic>,
}

impl Widget for &mut ParameterView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 100.0)
            .show(ui, |ui| {
                egui::Frame::group(ui.style())
                    .show(ui, |ui| {
                        TableBuilder::new(ui)
                            .column(Column::auto())
                            .column(Column::remainder())
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.with_layout(
                                        Layout::centered_and_justified(egui::Direction::TopDown),
                                        |ui| ui.heading("Name"),
                                    );
                                });
                                header.col(|ui| {
                                    ui.with_layout(
                                        Layout::centered_and_justified(egui::Direction::TopDown),
                                        |ui| ui.heading("Value"),
                                    );
                                });
                            })
                            .body(|mut body| {
                                self.parameters.iter_mut().for_each(|(name, value)| {
                                    body.row(30.0, |mut row| {
                                        row.col(|ui| {
                                            ui.with_layout(
                                                Layout::top_down(egui::Align::Max),
                                                |ui| ui.label(format!("{name}: ")),
                                            );
                                        });
                                        row.col(|ui| {
                                            ui.with_layout(
                                                Layout::top_down(egui::Align::Center),
                                                |ui| match value.type_name() {
                                                    "i64" => {
                                                        let mut v = value.as_int().unwrap();
                                                        ui.add(egui::DragValue::new(&mut v));
                                                        *value = v.into();
                                                    }
                                                    "f64" => {
                                                        let mut v = value.as_float().unwrap();
                                                        ui.add(egui::DragValue::new(&mut v));
                                                        *value = v.into();
                                                    }
                                                    _ => (),
                                                },
                                            );
                                        });
                                    });
                                });
                            });
                        ui.allocate_space(ui.available_size());
                    })
                    .response
            })
            .inner
    }
}

impl ParameterView {
    pub fn refresh_parameters(&mut self, script: &Script) {
        let exports =
            PatternScript::get_script_exports(script).expect("Couldn't get script exports");

        exports.iter().for_each(|(exp, default)| {
            if !self.parameters.contains_key(exp) {
                self.parameters.insert(exp.clone(), default.clone());
            }
        });
        self.parameters
            .retain(|k, _| exports.iter().any(|(ek, _)| k == ek));
    }
}
