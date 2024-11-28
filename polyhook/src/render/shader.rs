use eframe::egui_wgpu::wgpu;

use super::texture::Texture;

pub struct Shader {
    shader: wgpu::ShaderModule,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group_layout: wgpu::BindGroupLayout,
}

pub struct BindGroups {
    pub uniform: wgpu::BindGroup,
    pub texture: wgpu::BindGroup,
}

impl Shader {
    fn shader_module_descriptor() -> wgpu::ShaderModuleDescriptor<'static> {
        wgpu::ShaderModuleDescriptor {
            label: Some("polyhook"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shader.wgsl").into()),
        }
    }

    pub fn new_shader(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(Self::shader_module_descriptor());

        let uniform_bind_group_layout: wgpu::BindGroupLayout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("uniform_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(192),
                    },
                    count: None,
                }],
            });

        let texture_bind_group_layout: wgpu::BindGroupLayout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true }
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        Self {
            shader,
            uniform_bind_group_layout,
            texture_bind_group_layout,
        }
    }

    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.shader
    }

    pub fn bind_group_layouts(&self) -> Vec<&wgpu::BindGroupLayout> {
        vec![&self.uniform_bind_group_layout, &self.texture_bind_group_layout]
    }

    pub fn init_bind_groups(&self, device: &wgpu::Device, uniform_buffer: &wgpu::Buffer, texture: &Texture) -> BindGroups{
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform_bind_group"),
            layout: &self.uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture_bind_group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        });

        BindGroups { uniform: uniform_bind_group, texture: texture_bind_group }
    }
}
