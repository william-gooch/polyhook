use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use eframe::egui_wgpu::wgpu;

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

pub struct ModelData {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl ModelData {
    pub fn num_indices(&self) -> usize {
        self.indices.len()
    }
}

#[derive(Clone)]
pub struct Model {
    data: ModelData,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl Model {
    pub fn new(data: ModelData, device: &wgpu::Device) -> Self {
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

        Self {
            data,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn transform(&self) -> glam::Mat4 {
        self.transform
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw_indexed(0..(self.model.num_indices() as u32), 0, 0..1);
    }
}

pub struct ModelInstance {
    model: Arc<Model>,

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
