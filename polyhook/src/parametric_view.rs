use egui::{Color32, Id, InnerResponse, Pos2, Rect, Sense, Stroke, Vec2, Widget};
use hooklib::parametric::{example_flat, Identifier, Operation, OperationRef, ParametricPattern};
use std::{iter::{once, Once}, time::{Duration, Instant}};

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
        let r = egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 50.0)
            .show(ui, |ui| {
                egui::Frame::group(ui.style())
                    .show(ui, |ui| {
                        self.cached_identifiers = self.pattern.defined_identifiers();
                        let r = self.pattern_ui(ui);
                        ui.allocate_space(ui.available_size());
                        r
                    })
                    .response
            });
        r.inner
    }
}

#[derive(Clone, Default)]
struct ShouldShowAdd(Option<Instant>);
impl ShouldShowAdd {
    const SHOW_DELAY: Duration = Duration::from_secs(1);

    fn now(&mut self) {
        self.0 = Some(Instant::now());
    }

    fn reset(&mut self) {
        self.0 = None;
    }

    fn should_show(&self) -> bool {
        self.0.is_some_and(|t| t.elapsed() < Self::SHOW_DELAY)
    }
}

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

enum Instruction {
    AddStep(AddStep),
    RemoveStep(usize),
}

impl ParametricView {
    pub fn get_code(&self) -> String {
        self.pattern.to_script()
    }

    fn pattern_ui(&self, ui: &mut egui::Ui) -> egui::Response {
        self.operation_ui(ui, self.pattern.root().expect("No root node found."))
    }

    fn add_step_button(&self, ui: &mut egui::Ui) -> egui::Response {
        let w = ui.available_width();
        let mut frame = egui::Frame::none().begin(ui);
        let add_resp = frame.content_ui.add_sized(
            egui::Vec2::new(w, 10.0),
            egui::Button::new("(+)").frame(false),
        );
        let p = frame.content_ui.painter();
        let line_color = ui.style().interact(&add_resp).text_color();
        p.line_segment(
            [
                frame.content_ui.min_rect().left_center() + egui::vec2(10.0, 0.0),
                frame.content_ui.min_rect().center() + egui::vec2(-10.0, 0.0),
            ],
            Stroke::new(1.0, line_color),
        );
        p.line_segment(
            [
                frame.content_ui.min_rect().right_center() + egui::vec2(-10.0, 0.0),
                frame.content_ui.min_rect().center() + egui::vec2(10.0, 0.0),
            ],
            Stroke::new(1.0, line_color),
        );
        frame.end(ui);

        add_resp
    }

    fn add_step_ui(
        &self,
        ui: &mut egui::Ui,
        resp: egui::Response,
        at: usize,
    ) -> InnerResponse<Option<AddStep>> {
        let popup_id = ui.make_persistent_id(resp.id.with(at).with("new_step_open"));
        let popup_open = ui.data(|d| d.get_temp::<bool>(popup_id)).unwrap_or(false);

        let should_show_add = ui.data_mut(|data| {
            let d = data.get_temp_mut_or_default::<ShouldShowAdd>(resp.id);
            if d.should_show() || popup_open {
                true
            } else {
                d.reset();
                false
            }
        });
        if should_show_add || ui.rect_contains_pointer(resp.rect) {
            let add_resp = self.add_step_button(ui);
            if add_resp.clicked() {
                ui.data_mut(|d| {
                    let v = d.get_temp_mut_or_default::<bool>(popup_id);
                    *v = !*v;
                })
            }

            let to_add = if popup_open {
                let popup_resp = ui.vertical_centered_justified(|ui| {
                    if ui.button("Define").clicked() {
                        Some(OperationType::Define)
                    } else if ui.button("Literal").clicked() {
                        Some(OperationType::Literal)
                    } else if ui.button("Variable").clicked() {
                        Some(OperationType::Variable)
                    } else if ui.button("Call").clicked() {
                        Some(OperationType::Call)
                    } else if ui.button("Repeat").clicked() {
                        Some(OperationType::Repeat)
                    } else {
                        None
                    }
                });
                if popup_resp.inner.is_some() {
                    ui.data_mut(|d| {
                        let v = d.get_temp_mut_or_default::<bool>(popup_id);
                        *v = false;
                    });
                }
                Some(InnerResponse::new(
                    popup_resp.inner.map(|t| AddStep { at, kind: t }),
                    popup_resp.response,
                ))
            } else {
                None
            };

            let total_resp = resp.clone() | add_resp.clone();
            let total_resp = if let Some(r) = to_add.as_ref() {
                total_resp | r.response.clone()
            } else {
                total_resp
            };

            let should_show = ui.rect_contains_pointer(total_resp.rect);
            ui.data_mut(|data| {
                let r = data.get_temp_mut_or_default::<ShouldShowAdd>(resp.id);
                if should_show {
                    r.now();
                }
            });

            InnerResponse::new(to_add.and_then(|r| r.inner), total_resp)
        } else {
            InnerResponse::new(None, resp)
        }
    }

    fn remove_step_ui(
        &self,
        ui: &mut egui::Ui,
        in_rect: Rect,
        at: usize,
    ) -> InnerResponse<Option<usize>> {
        let rect = Rect::from_center_size(in_rect.right_top() + Vec2::new(-10.0, 10.0), Vec2::new(10.0, 10.0));
        let resp = ui.put(rect, egui::Button::new("X"));
        ui.advance_cursor_after_rect(in_rect);

        if resp.clicked() {
            InnerResponse::new(Some(at), resp)
        } else {
            InnerResponse::new(None, resp)
        }
    }

    fn operation_ui(&self, ui: &mut egui::Ui, operation: OperationRef) -> egui::Response {
        match &mut *self.pattern.get_mut(operation).expect("Invalid node index") {
            Operation::Seq(vec) => {
                let resp = egui::Frame::default()
                    .fill(Color32::from_white_alpha(1))
                    .show(ui, |ui| {
                        let before_operations = once({
                            let InnerResponse { inner: add, response: add_resp } = self.add_step_ui(ui, ui.interact(ui.max_rect(), ui.id(), Sense::hover()), 0);
                            InnerResponse::new(add.map(Instruction::AddStep), add_resp)
                        });
                        let after_operations = vec.iter_mut()
                            .enumerate()
                            .map(|(i, op)| {
                                let resp = self.operation_ui(ui, *op);
                                let InnerResponse { inner: add, response: add_resp } = self.add_step_ui(ui, resp, i + 1);
                                if let Some(add) = add {
                                    InnerResponse::new(Some(Instruction::AddStep(add)), add_resp)
                                } else {
                                    let InnerResponse { inner: remove, response: remove_resp } = self.remove_step_ui(ui, add_resp.rect, i);
                                    InnerResponse::new(remove.map(Instruction::RemoveStep), remove_resp | add_resp)
                                }
                            });
                        before_operations.chain(after_operations)
                            .reduce(|a, b| {
                                InnerResponse::new(
                                    a.inner.or(b.inner),
                                    a.response | b.response,
                                )
                            })
                            .unwrap_or_else(|| InnerResponse::new(None, ui.response()))
                    })
                    .inner;

                if let Some(Instruction::AddStep(add)) = resp.inner {
                    let op_to_add = match add.kind {
                        OperationType::Define => {
                            self.pattern.define("new_variable", self.pattern.literal(0))
                        }
                        OperationType::Literal => self.pattern.literal(0),
                        OperationType::Variable => self.pattern.variable("select variable..."),
                        OperationType::Call => self.pattern.call("chain"),
                        OperationType::Repeat => self
                            .pattern
                            .repeat(self.pattern.literal(0), self.pattern.seq([])),
                    };
                    vec.insert(add.at, op_to_add);
                } else if let Some(Instruction::RemoveStep(i)) = resp.inner {
                    vec.remove(i);
                }
                resp.response
            }
            Operation::Define(identifier, operation) => {
                ui.horizontal(|ui| {
                    ui.label(format!("define {identifier} as"));
                    self.operation_ui(ui, *operation);
                    ui.allocate_space([ui.available_width(), 0.0].into());
                })
                .response
            }
            Operation::Literal(value) => ui.add(egui::DragValue::new(value)),
            Operation::Variable(identifier) => {
                egui::ComboBox::from_id_salt(operation)
                    .selected_text(identifier.to_string())
                    .show_ui(ui, |ui| {
                        self.cached_identifiers.iter().for_each(|option| {
                            ui.selectable_value(identifier, option.clone(), option.to_string());
                        });
                    })
                    .response
            }
            Operation::Call(identifier) => {
                ui.horizontal(|ui| {
                    ui.label(format!("{identifier}"));
                    ui.allocate_space([ui.available_width(), 0.0].into());
                })
                .response
            }
            Operation::Repeat(n, op) => {
                let resp_1 = ui.horizontal(|ui| {
                    ui.label("do")
                        .union(self.operation_ui(ui, *n))
                        .union(ui.label("times:"))
                });

                let resp_2 = ui.indent(0, |ui| self.operation_ui(ui, *op));

                resp_1.inner | resp_1.response | resp_2.inner | resp_2.response
            }
        }
    }
}
