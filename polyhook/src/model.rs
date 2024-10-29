pub mod pattern_model;

use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use eframe::egui_wgpu::wgpu;

use crate::shader::Shader;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 4],
}

impl Vertex {
    pub const fn new(position: [f32; 4]) -> Self {
        Self { position }
    }

    pub const fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
            ],
        }
    }
}

#[derive(Clone)]
pub struct ModelData {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl ModelData {
    pub fn num_indices(&self) -> usize {
        self.indices.len()
    }
}

pub struct ModelBuffers {
    vertex: wgpu::Buffer,
    index: wgpu::Buffer,
    uniform: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

#[derive(Clone)]
pub struct Model {
    data: ModelData,
    buffers: Arc<ModelBuffers>,
}

impl Model {
    pub fn new(data: ModelData, device: &wgpu::Device, shader: &Shader) -> Self {
        use wgpu::util::DeviceExt;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("polyhook_vtx"),
            contents: bytemuck::cast_slice(&data.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("polyhook_idx"),
            contents: bytemuck::cast_slice(&data.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("polyhook_uni"),
            contents: bytemuck::cast_slice(glam::Mat4::IDENTITY.as_ref()),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });
        
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("polyhook_bdg"),
            layout: shader.bind_group_layout(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let buffers = ModelBuffers {
            vertex: vertex_buffer,
            index: index_buffer,
            uniform: uniform_buffer,
            bind_group
        };
        let buffers = Arc::new(buffers);

        Self {
            data,
            buffers,
        }
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_bind_group(0, &self.buffers.bind_group, &[]);
        render_pass.set_index_buffer(self.buffers.index.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_vertex_buffer(0, self.buffers.vertex.slice(..));
        render_pass.draw_indexed(0..(self.data.num_indices() as u32), 0, 0..1);
    }

    pub fn write_uniform(&self, queue: &wgpu::Queue, value: &glam::Mat4) {
        queue.write_buffer(
            &self.buffers.uniform,
            0,
            bytemuck::cast_slice(value.as_ref()),
        );
    }
}

const VERTICES: [Vertex; 8] = [
    Vertex::new([-1.,  1., -1., 1.]), // ulb
    Vertex::new([-1.,  1.,  1., 1.]), // ulf
    Vertex::new([ 1.,  1., -1., 1.]), // urb
    Vertex::new([ 1.,  1.,  1., 1.]), // urf
    Vertex::new([-1., -1., -1., 1.]), // dlb
    Vertex::new([-1., -1.,  1., 1.]), // dlf
    Vertex::new([ 1., -1., -1., 1.]), // drb
    Vertex::new([ 1., -1.,  1., 1.]), // drf
];

const INDICES: [u16; 36] = [
    0, 1, 2, 2, 3, 1, // up
    4, 5, 6, 6, 7, 5, // down
    0, 1, 4, 4, 5, 1, // left
    2, 3, 6, 6, 7, 3, // right
    0, 2, 4, 4, 6, 2, // back
    1, 3, 5, 5, 7, 3, // front
];

pub fn cube() -> ModelData {
    ModelData {
        vertices: VERTICES.to_vec(),
        indices: INDICES.to_vec(),
    }
}
