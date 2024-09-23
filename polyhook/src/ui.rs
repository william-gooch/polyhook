use egui::Context;
use egui_wgpu::Renderer;
use egui_winit::State;
use winit::{event::WindowEvent, window::Window};

pub struct EguiRenderer {
    state: State,
    renderer: Renderer,
}

impl EguiRenderer {
    pub fn context(&self) -> &Context {
        self.state.egui_ctx()
    }

    pub fn new(device: &wgpu::Device, window: &Window) -> Self {
        let egui_context = Context::default();
        let egui_state = egui_winit::State::new(
            egui_context,
            egui::viewport::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        let egui_renderer = Renderer::new(device, wgpu::TextureFormat::Rgba8Unorm, None, 1, false);

        Self {
            state: egui_state,
            renderer: egui_renderer,
        }
    }

    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) {
        self.state.on_window_event(window, event);
    }

    pub fn ppp(&mut self, v: f32) {
        self.state.egui_ctx().set_pixels_per_point(v);
    }

    pub fn draw<'a>(
        &'a mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &'a mut wgpu::CommandEncoder,
        window: &Window,
        window_surface_view: &wgpu::TextureView,
        screen_descriptor: egui_wgpu::ScreenDescriptor,
        run_ui: impl FnOnce(&Context),
    ) {
        self.state
            .egui_ctx()
            .set_pixels_per_point(screen_descriptor.pixels_per_point);

        let raw_input = self.state.take_egui_input(window);
        let full_output = self.state.egui_ctx().run(raw_input, |ui| {
            run_ui(self.state.egui_ctx());
        });

        self.state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self
            .state
            .egui_ctx()
            .tessellate(full_output.shapes, self.state.egui_ctx().pixels_per_point());
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }

        self.renderer
            .update_buffers(device, queue, encoder, &tris, &screen_descriptor);

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: window_surface_view,
                    resolve_target: None,
                    ops: egui_wgpu::wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                label: Some("egui main render pass"),
                occlusion_query_set: None,
            });
            self.renderer.render(&mut rpass, &tris, &screen_descriptor);
        }

        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }
    }
}
