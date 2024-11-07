mod model;
mod render;
mod shader;
mod transform;

mod code_view;

use egui::Vec2;
use hooklib::examples;
use std::sync::Arc;

struct App {
    code_view: code_view::CodeView,
    renderer: render::Renderer,
    orbit: transform::Orbit,
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

        Self {
            code_view: Default::default(),
            renderer: render::Renderer::new(cc.wgpu_render_state.as_ref().unwrap()).unwrap(),
            orbit: transform::Orbit {
                phi: 0.0,
                theta: 0.0,
                d: 3.0,
            },
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Examples", |ui| {
                    for &(name, code) in examples::EXAMPLES {
                        if ui.button(name).clicked() {
                            self.code_view.load_code(code);
                            ui.close_menu();
                        }
                    }
                })
            });

            egui::SidePanel::left("left_panel")
                .resizable(true)
                .default_width(ui.available_width() * 0.5)
                .show_inside(ui, |mut ui| {
                    let new_model = self.code_view.code_view_show(&mut ui);
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
                        render::RendererCallback(self.renderer.mvp.matrix()),
                    ));

                let drag = response.drag_motion();
                let z = ctx.input(|input| input.smooth_scroll_delta.y);
                self.orbit.update(glam::vec3(drag.x, drag.y, z));
                self.renderer.mvp.view = self.orbit.matrix();
            });
        });

        //self.renderer.mvp.model = glam::Mat4::from_rotation_y(0.02) * self.renderer.mvp.model;
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
