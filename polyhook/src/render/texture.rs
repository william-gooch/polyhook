use eframe::egui_wgpu::wgpu;
use image::{GenericImageView, ImageBuffer, Rgba};

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,

    pub rgba: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
}

impl Texture {
    pub fn from_bytes(device: &wgpu::Device, bytes: &[u8], label: &str) -> Self {
        let img = image::load_from_memory(bytes).unwrap();
        Self::from_image(device, &img, Some(label))
    }

    pub fn from_image(
        device: &wgpu::Device,
        image: &image::DynamicImage,
        label: Option<&str>,
    ) -> Self {
        let rgba = image.to_rgba8();
        let (width, height) = image.dimensions();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            rgba: Some(rgba),
        }
    }

    pub fn write_image(&self, queue: &wgpu::Queue) {
        let rgba = self
            .rgba
            .as_ref()
            .expect("Attempted to write image of a non-image texture (depth?)");
        let (width, height) = rgba.dimensions();

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}
