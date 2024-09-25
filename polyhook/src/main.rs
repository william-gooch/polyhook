mod render;
use std::sync::Arc;

use egui::Vec2;

struct App {
    renderer: render::Renderer,
    orbit: render::Orbit,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            renderer: render::Renderer::new(cc.wgpu_render_state.as_ref().unwrap()).unwrap(),
            orbit: render::Orbit {
                phi: 0.0,
                theta: 0.0,
                d: 3.0,
            },
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::SidePanel::left("left_panel")
                .resizable(true)
                .default_width(150.0)
                .show_inside(ui, |ui| {
                    ui.heading(format!(
                        "Hello, world! The window size is {} by {}",
                        ctx.screen_rect().width(),
                        ctx.screen_rect().height()
                    ));
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

                let mut rot = glam::Mat4::IDENTITY;
                if response.dragged_by(egui::PointerButton::Secondary) {
                    let del = response.drag_motion();
                    self.orbit.theta -= del.x * 0.01;
                    self.orbit.phi -= del.y * 0.01;
                    // transform *=
                    //     glam::Mat4::from_translation(glam::vec3(del.x * 0.01, -del.y * 0.01, 0.0));
                    // rot *= glam::Mat4::from_rotation_y(del.x * 0.01);
                    // rot *= glam::Mat4::from_rotation_z(del.y * 0.01);
                }
                let zdel = ctx.input(|input| input.smooth_scroll_delta.y);
                self.orbit.d -= zdel * 0.01;
                let zoom = glam::Mat4::from_translation(glam::vec3(0.0, 0.0, -zdel * 0.01));
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
            device_descriptor: Arc::new(|adapter| wgpu::DeviceDescriptor {
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
