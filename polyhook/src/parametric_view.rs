use egui::{popup, Color32, RichText, Sense, Stroke, Style, Widget};
use hooklib::parametric::{example_flat, Identifier, Operation, OperationRef, ParametricPattern};

pub struct ParametricView {
    cached_identifiers: Vec<Identifier>,
    pattern: ParametricPattern,
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
                if ui.button("Convert to Script").clicked() {
                    println!("{}", self.pattern.to_script());
                }
            })
            .response
    }
}

#[derive(Clone, Default)]
struct ShouldShowAdd(bool);

enum OperationType {
    Define,
    Literal,
    Variable,
    Call,
    Repeat,
}

struct AddStep {
    at: usize,
    kind: OperationType,
}

impl ParametricView {
    fn pattern_ui(&self, ui: &mut egui::Ui) {
        self.operation_ui(ui, self.pattern.root().expect("No root node found."));
    }

    fn add_step_button(&self, ui: &mut egui::Ui) -> egui::Response {
        let w = ui.available_width();
        let mut frame = egui::Frame::none().begin(ui);
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

        add_resp
    }

    fn add_step_ui(&self, ui: &mut egui::Ui, resp: egui::Response, at: usize) -> egui::InnerResponse<Option<AddStep>> {
        let should_show_add = ui.data_mut(|data| data.get_temp_mut_or_default::<ShouldShowAdd>(resp.id).0);
        if should_show_add || resp.contains_pointer() {
            let add_resp = self.add_step_button(ui);

            let popup_id = ui.make_persistent_id(resp.id.with(at).with("new_step_open"));

            if add_resp.clicked() {
                ui.data_mut(|d| {
                    let v = d.get_temp_mut_or_default::<bool>(popup_id);
                    *v = !*v;
                })
            }

            let to_add = if ui.data(|d| d.get_temp::<bool>(popup_id)).unwrap_or(false) {
                let popup_resp = egui::Frame::none().show(ui, |ui| {
                    if ui.button("Define").clicked() { Some(OperationType::Define) }
                    else if ui.button("Literal").clicked() { Some(OperationType::Literal) }
                    else if ui.button("Variable").clicked() { Some(OperationType::Variable) }
                    else if ui.button("Call").clicked() { Some(OperationType::Call) }
                    else if ui.button("Repeat").clicked() { Some(OperationType::Repeat) }
                    else { None }
                });
                if let Some(r) = popup_resp.inner.as_ref() {
                    ui.data_mut(|d| {
                        let v = d.get_temp_mut_or_default::<bool>(popup_id);
                        *v = false;
                    });
                }
                egui::InnerResponse::new(popup_resp.inner.map(|t| AddStep { at, kind: t }), popup_resp.response)
            } else { egui::InnerResponse::new(None, ui.response()) };

            let total_resp = resp
                .union(add_resp.clone())
                .union(to_add.response.clone());

            ui.data_mut(|data| {
                let r = data.get_temp_mut_or_default::<ShouldShowAdd>(resp.id);
                *r = ShouldShowAdd(total_resp.contains_pointer());
            });

            egui::InnerResponse::new(to_add.inner, total_resp)
        } else { egui::InnerResponse::new(None, resp) }
    }

    fn operation_ui(&self, ui: &mut egui::Ui, operation: OperationRef) -> egui::Response {
        match &mut *self.pattern.get_mut(operation).expect("Invalid node index") {
            Operation::Seq(vec) => {
                let resp = vec.iter_mut()
                    .enumerate()
                    .map(|(i, op)| {
                        let resp = self.operation_ui(ui, *op);
                        self.add_step_ui(ui, resp, i)
                    })
                    .reduce(|a, b| 
                        egui::InnerResponse::new(a.inner.or(b.inner), a.response.union(b.response))
                    )
                    .unwrap_or_else(|| egui::InnerResponse::new(None, ui.response()));
                
                if let Some(add) = resp.inner {
                    println!("Add an operation after {}", add.at);
                    let op_to_add = match add.kind {
                        OperationType::Define => self.pattern.define("new_variable", self.pattern.literal(0)),
                        OperationType::Literal => self.pattern.literal(0),
                        OperationType::Variable => self.pattern.variable("select variable..."),
                        OperationType::Call => self.pattern.call("chain"),
                        OperationType::Repeat => self.pattern.repeat(self.pattern.literal(0), self.pattern.seq([])),
                    };
                    vec.insert(add.at + 1, op_to_add);
                }
                resp.response
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
                // let r = ui.horizontal(|ui| ui.label(format!("{identifier}"))).response;
                // r.interact(Sense::hover())
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