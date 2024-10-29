use crate::model::pattern_model::model_from_pattern;
use crate::model::{cube, Model, ModelData, Vertex};
use crate::shader::Shader;
use crate::transform::MVP;

use eframe::egui_wgpu;
use eframe::egui_wgpu::wgpu;

pub struct Renderer {
    pub mvp: MVP,
    render_state: egui_wgpu::RenderState,
    shader: Shader,
}

impl Renderer {
    pub fn new(wgpu_render_state: &egui_wgpu::RenderState) -> Option<Self> {
        let device = &wgpu_render_state.device;

        let shader = Shader::new_shader(&device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("polyhook"),
            bind_group_layouts: &[shader.bind_group_layout()],
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
                polygon_mode: wgpu::PolygonMode::Line,
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: Default::default(),
        });

        let pattern = hooklib::pattern::test_pattern_sphere();
        let model = Model::new(model_from_pattern(&pattern), &device, &shader);

        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(RendererResources {
                pipeline,
                model,
            });

        Some(Self { mvp: MVP::new(), render_state: wgpu_render_state.clone(), shader })
    }

    pub fn set_model(&mut self, model: ModelData) {
        let model = Model::new(model, &self.render_state.device, &self.shader);

        let mut state = self.render_state.renderer.write();
        let resources = state
            .callback_resources
            .get_mut::<RendererResources>()
            .expect("Couldn't get renderer resources");

        resources.model = model;
    }
}

pub struct RendererCallback(pub glam::Mat4);

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
        info: egui::PaintCallbackInfo,
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
}

impl RendererResources {
    fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue, params: &RendererCallback) {
        self.model.write_uniform(queue, &params.0);
    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        self.model.draw(render_pass);
    }
}
