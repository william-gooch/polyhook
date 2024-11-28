pub mod model;
pub mod pattern_model;
pub mod shader;
pub mod transform;
pub mod texture;

use pattern_model::model_from_pattern;
use model::{Model, ModelData, Vertex};
use shader::Shader;
use texture::Texture;
use transform::Mvp;

use eframe::egui_wgpu;
use eframe::egui_wgpu::wgpu;

pub struct Renderer {
    pub mvp: Mvp,
    render_state: egui_wgpu::RenderState,
    shader: Shader,
}

impl Renderer {
    pub fn new(wgpu_render_state: &egui_wgpu::RenderState) -> Option<Self> {
        let device = &wgpu_render_state.device;

        let shader = Shader::new_shader(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("polyhook"),
            bind_group_layouts: &shader.bind_group_layouts()[..],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("polyhook"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader.module(),
                entry_point: "vs_main",
                buffers: &[Vertex::buffer_layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader.module(),
                entry_point: "fs_main",
                targets: &[Some(wgpu_render_state.target_format.into())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                // polygon_mode: wgpu::PolygonMode::Line,
                // topology: wgpu::PrimitiveTopology::LineList,
                polygon_mode: wgpu::PolygonMode::Fill,
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: Default::default(),
        });

        let pattern = hooklib::pattern::test_pattern_sphere();

        let texture_bytes = include_bytes!("../assets/dc.png");
        let texture = Texture::from_bytes(device, texture_bytes, "dc_texture");

        let model = Model::new(model_from_pattern(&pattern), device, &shader, &texture);

        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(RendererResources { pipeline, model, texture });

        Some(Self {
            mvp: Mvp::new(),
            render_state: wgpu_render_state.clone(),
            shader,
        })
    }

    pub fn set_model(&mut self, model: ModelData) {
        let texture_bytes = include_bytes!("../assets/dc.png");
        let texture = Texture::from_bytes(&self.render_state.device, texture_bytes, "dc_texture");

        let model = Model::new(model, &self.render_state.device, &self.shader, &texture);

        let mut state = self.render_state.renderer.write();
        let resources = state
            .callback_resources
            .get_mut::<RendererResources>()
            .expect("Couldn't get renderer resources");

        resources.model = model;
    }
}

pub struct RendererCallback(pub Mvp);

impl egui_wgpu::CallbackTrait for RendererCallback {
    fn prepare(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut eframe::wgpu::CommandEncoder,
        callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<eframe::wgpu::CommandBuffer> {
        let resources: &RendererResources = callback_resources.get().unwrap();
        resources.prepare(device, queue, self);
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut eframe::wgpu::RenderPass<'static>,
        callback_resources: &egui_wgpu::CallbackResources,
    ) {
        let resources: &RendererResources = callback_resources.get().unwrap();
        resources.paint(render_pass);
    }
}

struct RendererResources {
    pipeline: wgpu::RenderPipeline,
    model: Model,
    texture: Texture,
}

impl RendererResources {
    fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue, params: &RendererCallback) {
        self.model.write_mvp(queue, &params.0);
        self.texture.write_image(queue);
    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        self.model.draw(render_pass);
    }
}
