use eframe::egui_wgpu::wgpu;

pub struct Shader {
    shader: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
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

        let bind_group_layout: wgpu::BindGroupLayout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("polyhook"),
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

        Self {
            shader,
            bind_group_layout,
        }
    }

    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.shader
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
}
