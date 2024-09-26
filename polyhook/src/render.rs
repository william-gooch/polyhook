use crate::transform::MVP;

use bytemuck::{Pod, Zeroable};
use eframe::egui_wgpu;
use eframe::egui_wgpu::wgpu;
use eframe::wgpu::util::{DeviceExt, RenderEncoder};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex([f32; 4]);

const VERTICES: [Vertex; 8] = [
    Vertex([-1.,  1., -1., 1.]), // ulb
    Vertex([-1.,  1.,  1., 1.]), // ulf
    Vertex([ 1.,  1., -1., 1.]), // urb
    Vertex([ 1.,  1.,  1., 1.]), // urf
    Vertex([-1., -1., -1., 1.]), // dlb
    Vertex([-1., -1.,  1., 1.]), // dlf
    Vertex([ 1., -1., -1., 1.]), // drb
    Vertex([ 1., -1.,  1., 1.]), // drf
];

const INDICES: [u16; 36] = [
    0, 1, 2, 2, 3, 1, // up
    4, 5, 6, 6, 7, 5, // down
    0, 1, 4, 4, 5, 1, // left
    2, 3, 6, 6, 7, 3, // right
    0, 2, 4, 4, 6, 2, // back
    1, 3, 5, 5, 7, 3, // front
];

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
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                    ],
                }],
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
            label: Some("polyhook_uni"),
            contents: bytemuck::cast_slice(glam::Mat4::IDENTITY.as_ref()),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("polyhook_vtx"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("polyhook_idx"),
            contents: bytemuck::cast_slice(&INDICES),
            usage: wgpu::BufferUsages::INDEX,
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
                vertex_buffer,
                index_buffer,
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
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
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
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw_indexed(0..(INDICES.len() as u32), 0, 0..1);
    }
}
