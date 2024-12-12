pub mod model;
pub mod pattern_model;
pub mod shader;
pub mod texture;
pub mod transform;

use model::{Model, ModelData, Vertex};
use pattern_model::model_from_pattern;
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            primitive: wgpu::PrimitiveState {
                // polygon_mode: wgpu::PolygonMode::Line,
                // topology: wgpu::PrimitiveTopology::LineList,
                polygon_mode: wgpu::PolygonMode::Fill,
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Front),
                ..Default::default()
            },
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: Default::default(),
        });

        let pattern = hooklib::pattern::test_pattern_sphere();

        let diffuse_bytes = include_bytes!("../assets/dc.png");
        let tex_diffuse = Texture::from_bytes(device, diffuse_bytes, "dc_diffuse");

        let normal_bytes = include_bytes!("../assets/dc_normal.png");
        let tex_normal = Texture::from_bytes(device, normal_bytes, "dc_normal");

        let model = Model::new(
            model_from_pattern(&pattern),
            device,
            &shader,
            &tex_diffuse,
            &tex_normal,
        );

        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(RendererResources {
                pipeline,
                model,
                tex_diffuse,
                tex_normal,
            });

        Some(Self {
            mvp: Mvp::new(),
            render_state: wgpu_render_state.clone(),
            shader,
        })
    }

    pub fn set_model(&mut self, model: ModelData) {
        let device = &*self.render_state.device;

        let diffuse_bytes = include_bytes!("../assets/dc.png");
        let tex_diffuse = Texture::from_bytes(device, diffuse_bytes, "dc_diffuse");

        let normal_bytes = include_bytes!("../assets/dc_normal.png");
        let tex_normal = Texture::from_bytes(device, normal_bytes, "dc_normal");

        let model = Model::new(model, device, &self.shader, &tex_diffuse, &tex_normal);

        let mut state = self.render_state.renderer.write();
        let resources = state
            .callback_resources
            .get_mut::<RendererResources>()
            .expect("Couldn't get renderer resources");

        resources.model = model;
        resources.tex_diffuse = tex_diffuse;
        resources.tex_normal = tex_normal;
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
    tex_diffuse: Texture,
    tex_normal: Texture,
}

impl RendererResources {
    fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue, params: &RendererCallback) {
        self.model.write_mvp(queue, &params.0);
        self.tex_diffuse.write_image(queue);
        self.tex_normal.write_image(queue);
    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        self.model.draw(render_pass);
    }
}
