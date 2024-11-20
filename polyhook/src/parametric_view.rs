use egui::{Color32, RichText, Stroke, Style, Widget};
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

#[derive(Clone, Default)]
struct ShouldShowAdd(bool);

impl ParametricView {
    fn pattern_ui(&self, ui: &mut egui::Ui) {
        self.operation_ui(ui, self.pattern.root().expect("No root node found."));
    }

    fn operation_ui(&self, ui: &mut egui::Ui, operation: OperationRef) -> egui::Response {
        match &mut *self.pattern.get_mut(operation).expect("Invalid node index") {
            Operation::Seq(vec) => {
                let (resp, insert_idx) = vec.iter_mut()
                    .enumerate()
                    .map(|(i, op)| {
                        let resp = self.operation_ui(ui, *op);
                        let should_show_add = ui.data_mut(|data| data.get_temp_mut_or_default::<ShouldShowAdd>(resp.id).0);
                        if should_show_add || resp.hovered() {
                            let mut frame = egui::Frame::none().begin(ui);
                            let w = ui.available_width();
                            let add_resp = frame.content_ui.add_sized(
                                egui::Vec2::new(w, 10.0), 
                                egui::Button::new("(+)")
                                    .frame(false)
                            );
                            let p = frame.content_ui.painter();
                            let line_color = ui.style().interact(&add_resp).text_color();
                            p.line_segment([
                                frame.content_ui.min_rect().left_center() + egui::vec2(10.0, 0.0),
                                frame.content_ui.min_rect().center()      + egui::vec2(-10.0, 0.0),
                            ], Stroke::new(1.0, line_color));
                            p.line_segment([
                                frame.content_ui.min_rect().right_center() + egui::vec2(-10.0, 0.0),
                                frame.content_ui.min_rect().center()       + egui::vec2(10.0, 0.0),
                            ], Stroke::new(1.0, line_color));
                            frame.end(ui);
                            let total_resp = resp.union(add_resp.clone());

                            ui.data_mut(|data| {
                                let r = data.get_temp_mut_or_default::<ShouldShowAdd>(resp.id);
                                *r = ShouldShowAdd(total_resp.hovered());
                            });

                            if add_resp.clicked() {
                                (total_resp, Some(i))
                            } else {
                                (total_resp, None)
                            }
                        } else { (resp, None) }
                    })
                    .reduce(|(a_resp, a_idx), (b_resp, b_idx)| 
                        (a_resp.union(b_resp), a_idx.or(b_idx))
                    )
                    .unwrap_or_else(|| (ui.response(), None));
                
                if let Some(idx) = insert_idx {
                    println!("Add an operation after {idx}");
                    vec.insert(idx + 1, self.pattern.define("asdf", self.pattern.literal(5)));
                }
                resp
            },
            Operation::Define(identifier, operation) => {
                let resp = ui.horizontal(|ui| {
                    ui.label(format!("define {identifier} as"))
                        .union(self.operation_ui(ui, *operation))
                });
                resp.inner.union(resp.response)
            },
            Operation::Literal(value) => {
                ui.add(egui::DragValue::new(value))
            },
            Operation::Variable(identifier) => {
                egui::ComboBox::from_id_salt(operation)
                    .selected_text(identifier.to_string())
                    .show_ui(ui, |ui| {
                        self.cached_identifiers.iter()
                            .for_each(|option| {
                                ui.selectable_value(identifier, option.clone(), option.to_string());
                            });
                    }).response
            },
            Operation::Call(identifier) => {
                ui.label(format!("{identifier}"))
            },
            Operation::Repeat(n, op) => {
                let resp_1 = ui.horizontal(|ui| {
                    ui.label("do")
                        .union(self.operation_ui(ui, *n))
                        .union(ui.label("times:"))
                });

                let resp_2 = ui.indent(0, |ui| {
                    self.operation_ui(ui, *op)
                });

                resp_1.inner
                    .union(resp_1.response)
                    .union(resp_2.inner)
                    .union(resp_2.response)
            },
        }
    }
}