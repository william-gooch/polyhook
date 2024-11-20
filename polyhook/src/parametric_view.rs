use egui::{RichText, Widget};
use hooklib::parametric::{example_flat, Identifier, Operation, OperationRef, ParametricPattern};

pub struct ParametricView {
    cached_identifiers: Vec<Identifier>,
    pattern: ParametricPattern
}

impl Default for ParametricView {
    fn default() -> Self {
        Self {
            cached_identifiers: Vec::default(),
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
                self.cached_identifiers = self.pattern.defined_identifiers();
                self.pattern_ui(ui);
            })
            .response
    }
}

impl ParametricView {
    fn pattern_ui(&self, ui: &mut egui::Ui) {
        self.operation_ui(ui, self.pattern.root().expect("No root node found."));
    }

    fn operation_ui(&self, ui: &mut egui::Ui, operation: OperationRef) {
        match &mut *self.pattern.get_mut(operation).expect("Invalid node index") {
            Operation::Seq(vec) => {
                vec.iter_mut()
                    .for_each(|op| {
                        self.operation_ui(ui, *op);
                    });
            },
            Operation::Define(identifier, operation) => {
                ui.horizontal(|ui| {
                    ui.label(format!("define {identifier} as"));
                    self.operation_ui(ui, *operation);
                });
            },
            Operation::Literal(value) => {
                ui.add(egui::DragValue::new(value));
            },
            Operation::Variable(identifier) => {
                // TODO: fix shared id_salt issue
                egui::ComboBox::from_id_salt(operation)
                    .selected_text(identifier.to_string())
                    .show_ui(ui, |ui| {
                        self.cached_identifiers.iter()
                            .for_each(|option| {
                                ui.selectable_value(identifier, option.clone(), option.to_string());
                            })
                    });
            },
            Operation::Call(identifier) => {
                ui.label(format!("{identifier}"));
            },
            Operation::Repeat(n, op) => {
                {
                    ui.horizontal(|ui| {
                        ui.label("do");
                        self.operation_ui(ui, *n);
                        ui.label("times:");
                    });
                }
                ui.indent(0, |ui| {
                    self.operation_ui(ui, *op);
                });
            },
        }
    }
}