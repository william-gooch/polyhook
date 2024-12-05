mod render;

mod code_view;
mod parametric_view;

use egui::{Color32, Ui, Vec2};
use hooklib::examples::{self, EXAMPLE_FLAT};
use render::pattern_model::{model_from_pattern, model_from_pattern_2d};
use render::model::ModelData;
use render::transform::Orbit;
use parametric_view::ParametricView;
use rfd::FileDialog;
use std::env::args;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;

use std::{
    error::Error,
    thread::{spawn, JoinHandle},
};

fn load_file(path: &Path) -> Result<String, String> {
    let mut f = File::open(path).map_err(|err| err.to_string())?;
    let mut s = String::new();
    f.read_to_string(&mut s).map_err(|err| err.to_string())?;

    Ok(s)
}

#[derive(Default)]
struct RenderButton {
    err: Option<Box<dyn Error + Send + Sync>>,
    thread: Option<JoinHandle<Result<ModelData, Box<dyn Error + Send + Sync>>>>,
    is_2d_mode: bool,
}

impl RenderButton {
    fn start_render(&mut self, code: String) {
        let is_2d_mode = self.is_2d_mode;
        self.thread = Some(spawn(move || {
            let pattern = hooklib::script::PatternScript::eval_script(code.as_ref());
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

    fn show<F: Fn() -> String>(&mut self, ui: &mut Ui, get_code: F) -> Option<ModelData> {
        if let Some(err) = &self.err {
            let err_str = format!("{err}");

            ui.colored_label(Color32::RED, err_str);
        }

        ui.add_enabled_ui(self.thread.as_ref().is_none_or(|t| t.is_finished()), |ui| {
            ui.checkbox(&mut self.is_2d_mode, "2D Mode");
            let button = ui.add_sized(ui.available_size(), egui::Button::new("Render"));
            if button.clicked() {
                self.err = None;
                self.start_render(get_code());
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
    Parametric,
    Code,
}

struct App {
    code_view: code_view::CodeView,
    parametric_view: ParametricView,
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

        let code = args().nth(1)
            .and_then(|s| load_file(Path::new(&s)).inspect_err(|err| eprintln!("Couldn't open file: {}", err)).ok())
            .unwrap_or(EXAMPLE_FLAT.into());

        Self {
            code_view: code_view::CodeView { code },
            parametric_view: Default::default(),
            renderer: render::Renderer::new(cc.wgpu_render_state.as_ref().unwrap()).unwrap(),
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
                    if ui.button("Open").clicked() {
                        let file = FileDialog::new()
                            .add_filter("polyhook", &["ph"])
                            .set_directory(".")
                            .pick_file();
                        file
                            .and_then(|file| {
                                load_file(file.as_path())
                                    .inspect_err(|err| eprintln!("Couldn't load file: {err}"))
                                    .ok()
                            })
                            .inspect(|code: &String| {
                                self.code_view.load_code(code.as_str());
                            });
                        ui.close_menu();
                    }
                    if ui.button("Save").clicked() {
                        let file = FileDialog::new()
                            .add_filter("polyhook", &["ph"])
                            .set_directory(".")
                            .save_file();
                        file
                            .and_then(|file| {
                                let mut f = OpenOptions::new()
                                    .write(true)
                                    .create(true)
                                    .truncate(true)
                                    .open(file.as_path())
                                    .inspect_err(|err| eprintln!("Couldn't open file: {err}"))
                                    .ok()?;
                                let code = &self.code_view.code;
                                f.write(code.as_bytes())
                                    .inspect_err(|err| eprintln!("Couldn't write file: {err}"))
                                    .ok()?;
                                Some(())
                            });
                        ui.close_menu();
                    }
                });
                ui.menu_button("Examples", |ui| {
                    for &(name, code) in examples::EXAMPLES {
                        if ui.button(name).clicked() {
                            self.code_view.load_code(code);
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
                            .selectable_label(self.tab == AppTab::Parametric, "Parametric View")
                            .clicked()
                        {
                            self.tab = AppTab::Parametric;
                        }
                    });

                    if self.tab == AppTab::Code {
                        self.code_view.code_view_show(ui);
                    } else if self.tab == AppTab::Parametric {
                        ui.add(&mut self.parametric_view);
                    }

                    let new_model = self.render_button.show(ui, || {
                        if self.tab == AppTab::Code {
                            self.code_view.code.clone()
                        } else {
                            self.parametric_view.get_code()
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
