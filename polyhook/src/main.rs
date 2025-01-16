mod render;

mod code_view;
mod parameter_view;
mod visual_view;

use egui::{Color32, Ui, Vec2};
use hooklib::examples;
use hooklib::script::{PatternScript, Script};
use parameter_view::ParameterView;
use render::model::ModelData;
use render::pattern_model::{model_from_pattern, model_from_pattern_2d};
use render::transform::Orbit;
use rfd::FileDialog;
use rhai::{Dynamic, ImmutableString};
use std::collections::HashMap;
use std::env::args;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use visual_view::VisualView;

use std::{
    error::Error,
    thread::{spawn, JoinHandle},
};

#[derive(Default)]
struct RenderButton {
    err: Option<Box<dyn Error + Send + Sync>>,
    thread: Option<JoinHandle<Result<ModelData, Box<dyn Error + Send + Sync>>>>,
    is_2d_mode: bool,
}

impl RenderButton {
    fn start_render(&mut self, code: Script, parameters: HashMap<ImmutableString, Dynamic>) {
        let is_2d_mode = self.is_2d_mode;
        self.thread = Some(spawn(move || {
            let pattern =
                hooklib::script::PatternScript::eval_script_with_exports(&code, &parameters);
            match pattern {
                Ok(pattern) => {
                    if is_2d_mode {
                        Ok(model_from_pattern_2d(&pattern))
                    } else {
                        Ok(model_from_pattern(&pattern))
                    }
                }
                Err(err) => Err(err),
            }
        }));
    }

    fn check_render(&mut self) -> Option<Result<ModelData, Box<dyn Error + Send + Sync>>> {
        if self.thread.as_ref().is_some_and(|t| t.is_finished()) {
            Some(
                self.thread
                    .take()
                    .unwrap()
                    .join()
                    .expect("Failed to join thread."),
            )
        } else {
            None
        }
    }

    fn show<F: Fn() -> (Script, HashMap<ImmutableString, Dynamic>)>(
        &mut self,
        ui: &mut Ui,
        get_code: F,
    ) -> Option<ModelData> {
        if let Some(err) = &self.err {
            let err_str = format!("{err}");
            ui.horizontal(|ui| {
                if ui.button("X").clicked() {
                    self.err = None;
                }
                ui.add_sized(
                    [300.0, ui.available_height()],
                    egui::Label::new(egui::RichText::new(err_str).color(Color32::RED)).wrap(),
                );
            });
        }

        ui.add_enabled_ui(self.thread.as_ref().is_none_or(|t| t.is_finished()), |ui| {
            ui.checkbox(&mut self.is_2d_mode, "2D Mode");
            let button = ui.add_sized(ui.available_size(), egui::Button::new("Render"));
            if button.clicked() {
                self.err = None;
                let (code, parameters) = get_code();
                self.start_render(code, parameters);
            }
        });

        match self.check_render() {
            Some(Ok(model)) => Some(model),
            Some(Err(err)) => {
                self.err = Some(err);
                None
            }
            _ => None,
        }
    }
}

#[derive(PartialEq)]
enum AppTab {
    Visual,
    Parameters,
    Code,
}

struct App {
    code_view: code_view::CodeView,
    visual_view: VisualView,
    parameter_view: ParameterView,
    renderer: render::Renderer,
    render_button: RenderButton,
    orbit: Orbit,
    tab: AppTab,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = &cc.egui_ctx;
        let mut style = (*ctx.style()).clone();

        use egui::{FontFamily::Proportional, FontId, TextStyle::*};
        style.text_styles = [
            (Heading, FontId::new(30.0, Proportional)),
            (Body, FontId::new(18.0, Proportional)),
            (Monospace, FontId::new(18.0, egui::FontFamily::Monospace)),
            (Button, FontId::new(14.0, Proportional)),
            (Small, FontId::new(10.0, Proportional)),
        ]
        .into();
        ctx.set_style(style);

        let (code, starting_pattern) = args()
            .nth(1)
            .and_then(|s| {
                let script = Script::load_file(Path::new(&s))
                    .inspect_err(|err| eprintln!("Couldn't open file: {}", err))
                    .ok()?;

                let pattern = PatternScript::eval_script(&script)
                    .inspect_err(|err| eprintln!("{err}"))
                    .ok()?;
                Some((script, Some(pattern)))
            })
            .unwrap_or((Script::new(""), None));

        Self {
            code_view: code_view::CodeView { code },
            visual_view: Default::default(),
            parameter_view: Default::default(),
            renderer: render::Renderer::new(
                cc.wgpu_render_state.as_ref().unwrap(),
                starting_pattern,
            )
            .unwrap(),
            render_button: Default::default(),
            orbit: Orbit {
                phi: 0.0,
                theta: 0.0,
                d: 3.0,
            },
            tab: AppTab::Code,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        let script = Script::new("");
                        self.code_view.load_code(script);
                        ui.close_menu();
                    }
                    if ui.button("Open").clicked() {
                        let file = FileDialog::new()
                            .add_filter("polyhook", &["ph"])
                            .set_directory(".")
                            .pick_file();
                        let script = file.and_then(|file| {
                            Script::load_file(file.as_path())
                                .inspect_err(|err| eprintln!("Couldn't load file: {err}"))
                                .ok()
                        });
                        if let Some(script) = script {
                            self.code_view.load_code(script)
                        }
                        ui.close_menu();
                    }
                    if ui.button("Save").clicked() {
                        let file = self
                            .code_view
                            .code
                            .path()
                            .map(|path| path.to_path_buf())
                            .or_else(|| {
                                FileDialog::new()
                                    .add_filter("polyhook", &["ph"])
                                    .set_directory(".")
                                    .save_file()
                            });
                        file.and_then(|file| {
                            self.code_view.code.set_path(&file);
                            self.code_view.code.save_file().ok()?;
                            Some(())
                        });
                        ui.close_menu();
                    }
                    if ui.button("Save As").clicked() {
                        let file = FileDialog::new()
                            .add_filter("polyhook", &["ph"])
                            .set_directory(".")
                            .save_file();
                        file.and_then(|file| {
                            let mut f = OpenOptions::new()
                                .write(true)
                                .create(true)
                                .truncate(true)
                                .open(&file)
                                .inspect_err(|err| eprintln!("Couldn't open file: {err}"))
                                .ok()?;
                            let code = &self.code_view.code;
                            f.write(code.source().as_bytes())
                                .inspect_err(|err| eprintln!("Couldn't write file: {err}"))
                                .ok()?;
                            self.code_view.code.set_path(&file);
                            Some(())
                        });
                        ui.close_menu();
                    }
                });
                ui.menu_button("Examples", |ui| {
                    for &(name, file) in examples::EXAMPLES {
                        if ui.button(name).clicked() {
                            let script = Script::load_file(Path::new(file))
                                .inspect_err(|err| eprintln!("Couldn't load file: {err}"))
                                .ok();
                            if let Some(script) = script {
                                self.code_view.load_code(script)
                            }
                            ui.close_menu();
                        }
                    }
                });
            });

            egui::SidePanel::left("left_panel")
                .resizable(true)
                .default_width(ui.available_width() * 0.5)
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(self.tab == AppTab::Code, "Code View")
                            .clicked()
                        {
                            self.tab = AppTab::Code;
                        }
                        if ui
                            .selectable_label(self.tab == AppTab::Parameters, "Parameters")
                            .clicked()
                        {
                            self.parameter_view.refresh_parameters(&self.code_view.code);
                            self.tab = AppTab::Parameters;
                        }
                        if ui
                            .selectable_label(self.tab == AppTab::Visual, "Visual View")
                            .clicked()
                        {
                            self.tab = AppTab::Visual;
                        }
                    });

                    if self.tab == AppTab::Code {
                        self.code_view.code_view_show(ui);
                    } else if self.tab == AppTab::Visual {
                        ui.add(&mut self.visual_view);
                    } else if self.tab == AppTab::Parameters {
                        ui.add(&mut self.parameter_view);
                    }

                    let new_model = self.render_button.show(ui, || {
                        if self.tab == AppTab::Code || self.tab == AppTab::Parameters {
                            (
                                self.code_view.code.clone(),
                                self.parameter_view.parameters.clone(),
                            )
                        } else {
                            (self.visual_view.get_code().into(), Default::default())
                        }
                    });
                    if let Some(new_model) = new_model {
                        // TODO: switch the model to the new one
                        self.renderer.set_model(new_model);
                    }
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                let rect = ui.max_rect();
                let aspect_ratio = rect.width() / rect.height();
                self.renderer.mvp.update_projection(aspect_ratio);
                let response = ui.allocate_rect(
                    rect,
                    egui::Sense {
                        click: false,
                        drag: true,
                        focusable: false,
                    },
                );
                ui.painter()
                    .add(eframe::egui_wgpu::Callback::new_paint_callback(
                        rect,
                        render::RendererCallback(self.renderer.mvp),
                    ));

                let drag = response.drag_motion();
                let z = ctx.input(|input| input.smooth_scroll_delta.y);
                self.orbit.update(glam::vec3(drag.x, drag.y, z));
                self.renderer.mvp.view = self.orbit.matrix();
            });
        });

        // self.renderer.mvp.update_model(glam::Mat4::from_rotation_y(0.02) * self.renderer.mvp.model);
        ctx.request_repaint();
    }
}

fn main() -> eframe::Result {
    use eframe::egui_wgpu::{self, wgpu};

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size(Vec2::new(1024.0, 768.0))
            .with_resizable(true),
        wgpu_options: egui_wgpu::WgpuConfiguration {
            device_descriptor: Arc::new(|_adapter| wgpu::DeviceDescriptor {
                required_features: wgpu::Features::POLYGON_MODE_LINE,
                ..Default::default()
            }),
            ..Default::default()
        },
        depth_buffer: 32,
        ..Default::default()
    };
    eframe::run_native(
        "Polyhook",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
    .unwrap();

    Ok(())
}
