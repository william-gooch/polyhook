use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use eframe::egui_wgpu::wgpu;
use glam::{Vec2, Vec3};
use std::mem::offset_of;

use crate::render::{shader::Shader, texture::Texture, transform::Mvp};

use super::shader::BindGroups;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 4],
    uv: [f32; 2],
}

impl Vertex {
    pub const fn new(position: [f32; 4], uv: [f32; 2]) -> Self {
        Self { position, uv }
    }

    pub const fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: offset_of!(Vertex, position) as u64,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: offset_of!(Vertex, uv) as u64,
                    shader_location: 1
                },
            ],
        }
    }
}

impl From<(Vec3, Vec2)> for Vertex {
    fn from((pos, uv): (Vec3, Vec2)) -> Self {
        Self::new([pos.x, pos.y, pos.z, 1.0], [uv.x, uv.y])
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

    pub fn new(vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
        Self {
            vertices,
            indices,
        }
    }
}

pub struct ModelBuffers {
    vertex: wgpu::Buffer,
    index: wgpu::Buffer,
    uniform: wgpu::Buffer,
    bind_groups: BindGroups,
}

#[derive(Clone)]
pub struct Model {
    data: ModelData,
    buffers: Arc<ModelBuffers>,
}

impl Model {
    pub fn new(data: ModelData, device: &wgpu::Device, shader: &Shader, texture: &Texture) -> Self {
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
            contents: bytemuck::cast_slice(&[Mvp::new()]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let bind_groups = shader.init_bind_groups(device, &uniform_buffer, texture);

        let buffers = ModelBuffers {
            vertex: vertex_buffer,
            index: index_buffer,
            uniform: uniform_buffer,
            bind_groups,
        };
        let buffers = Arc::new(buffers);

        Self { data, buffers }
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_bind_group(0, &self.buffers.bind_groups.uniform, &[]);
        render_pass.set_bind_group(1, &self.buffers.bind_groups.texture, &[]);
        render_pass.set_index_buffer(self.buffers.index.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_vertex_buffer(0, self.buffers.vertex.slice(..));
        render_pass.draw_indexed(0..(self.data.num_indices() as u32), 0, 0..1);
    }

    pub fn write_mvp(&self, queue: &wgpu::Queue, value: &Mvp) {
        queue.write_buffer(&self.buffers.uniform, 0, bytemuck::cast_slice(&[*value]));
    }
}
