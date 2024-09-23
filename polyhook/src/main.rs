mod ui;

use std::{borrow::Cow, sync::Arc};

use anyhow::{anyhow, Result};
use egui_wgpu::WgpuConfiguration;
use ui::EguiRenderer;
use wgpu::TextureFormat;
use winit::{
    application::ApplicationHandler,
    event::{Event, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    instance: Option<wgpu::Instance>,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    ui: Option<EguiRenderer>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        ));
        pollster::block_on(self.setup_graphics()).expect("Failed to setup graphics");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Exiting...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let frame = self
                    .surface
                    .as_ref()
                    .unwrap()
                    .get_current_texture()
                    .expect("Failed to acquire next swapchain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = self
                    .device
                    .as_ref()
                    .unwrap()
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    rpass.set_pipeline(self.render_pipeline.as_ref().unwrap());
                    rpass.draw(0..3, 0..1);
                }

                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [frame.texture.width(), frame.texture.height()],
                    pixels_per_point: self.window.as_ref().unwrap().scale_factor() as f32,
                };

                self.ui.as_mut().unwrap().draw(
                    self.device.as_ref().unwrap(),
                    self.queue.as_ref().unwrap(),
                    &mut encoder,
                    self.window.as_ref().unwrap(),
                    &view,
                    screen_descriptor,
                    |ctx| {
                        egui::Window::new("winit + egui + wgpu says hello!")
                            .resizable(true)
                            .vscroll(true)
                            .default_open(false)
                            .show(ctx, |ui| {
                                ui.label("Label!");

                                if ui.button("Button!").clicked() {
                                    println!("boom!")
                                }

                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.label(format!(
                                        "Pixels per point: {}",
                                        ctx.pixels_per_point()
                                    ));
                                    if ui.button("-").clicked() {
                                        println!("zoom out");
                                    }
                                    if ui.button("+").clicked() {
                                        println!("zoom in");
                                    }
                                });
                            });
                    },
                );

                self.queue.as_ref().unwrap().submit(Some(encoder.finish()));
                frame.present();
            }
            _ => (),
        }
    }
}

impl App {
    async fn setup_graphics(&mut self) -> Result<()> {
        let window = self
            .window
            .as_ref()
            .ok_or(anyhow!("Window not initialized"))?;
        let mut size = window.inner_size();

        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .ok_or(anyhow!("Couldn't find compatible adapter"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
            )
            .await?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        self.ui = Some(EguiRenderer::new(&device, &window));

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let mut config = surface
            .get_default_config(&adapter, size.width, size.height)
            .ok_or(anyhow!("Failed to create surface config"))?;
        surface.configure(&device, &config);

        self.instance = Some(instance);
        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.render_pipeline = Some(render_pipeline);

        Ok(())
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    env_logger::init();
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
