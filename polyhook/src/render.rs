use eframe::egui_wgpu;
use eframe::egui_wgpu::wgpu;
use eframe::wgpu::util::DeviceExt;

pub struct Orbit {
    pub phi: f32,
    pub theta: f32,
    pub d: f32,
}

impl Orbit {
    pub fn matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_translation(glam::vec3(0.0, 0.0, self.d)) * glam::Mat4::from_rotation_x(self.phi) * glam::Mat4::from_rotation_y(self.theta) 
    }
}

pub struct MVP {
    pub model: glam::Mat4,
    pub view: glam::Mat4,
    pub projection: glam::Mat4,
}

impl MVP {
    const Z_NEAR: f32 = 0.01;
    const Z_FAR: f32 = 100.0;

    pub fn new() -> Self {
        Self {
            model: glam::Mat4::IDENTITY,
            view: glam::Mat4::from_translation(glam::vec3(0.0, 0.0, 3.0)),
            projection: glam::Mat4::perspective_lh(
                std::f32::consts::PI / 4.0,
                1.0,
                MVP::Z_NEAR,
                MVP::Z_FAR,
            ),
        }
    }

    pub fn matrix(&self) -> glam::Mat4 {
        self.projection * self.view * self.model
    }

    pub fn update_projection(&mut self, aspect_ratio: f32) {
        self.projection = glam::Mat4::perspective_lh(
            std::f32::consts::PI / 4.0,
            aspect_ratio,
            MVP::Z_NEAR,
            MVP::Z_FAR,
        );
    }
}

pub struct Renderer {
    pub mvp: MVP,
}

impl Renderer {
    pub fn new(wgpu_render_state: &egui_wgpu::RenderState) -> Option<Self> {
        let device = &wgpu_render_state.device;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("polyhook"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shader.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("polyhook"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(64),
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("polyhook"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("polyhook"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu_render_state.target_format.into())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                polygon_mode: wgpu::PolygonMode::Line,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("polyhook"),
            contents: bytemuck::cast_slice(glam::Mat4::IDENTITY.as_ref()),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("polyhook"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(RendererResources {
                pipeline,
                bind_group,
                uniform_buffer,
            });

        Some(Self { mvp: MVP::new() })
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

    fn paint<'a>(
        &'a self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut eframe::wgpu::RenderPass<'a>,
        callback_resources: &'a egui_wgpu::CallbackResources,
    ) {
        let resources: &RendererResources = callback_resources.get().unwrap();
        resources.paint(render_pass);
    }
}

struct RendererResources {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
}

impl RendererResources {
    fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue, params: &RendererCallback) {
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(params.0.as_ref()),
        );
    }

    fn paint<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
