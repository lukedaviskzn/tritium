use image::GenericImageView;

use crate::renderer::{Renderer, BindingHolder};

use super::{Handle, Resources};

pub struct Texture {
    pub(crate) texture: wgpu::Texture,
    pub(crate) size: glam::UVec2,
    pub(crate) view: Handle<wgpu::TextureView>,
    pub(crate) sampler: Handle<wgpu::Sampler>,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub(crate) fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str) -> wgpu::TextureView {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Texture::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        view
    }

    pub fn from_bytes(
        renderer: &Renderer,
        resources: &mut Resources,
        bytes: &[u8],
        label: Option<&str>,
        // srgb: bool,
        flip_y: bool,
    ) -> Result<Texture, image::ImageError> {
        let img = image::load_from_memory(bytes)?;
        Ok(Texture::from_image(renderer, resources, &img, label, flip_y))
    }

    pub fn from_image(
        renderer: &Renderer,
        resources: &mut Resources,
        image: &image::DynamicImage,
        label: Option<&str>,
        // srgb: bool,
        flip_y: bool,
    ) -> Texture {
        let (image_raw, dimensions, format, bytes_per_pixel) = {
            use image::DynamicImage::*;
            match image {
                ImageRgb8(_) => {
                    let dimensions = image.dimensions();
                    let raw = image.to_rgba8().to_vec();
                    // (raw, dimensions, if srgb { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm }, 4)
                    (raw, dimensions, wgpu::TextureFormat::Rgba8Unorm, 4)
                },
                ImageRgba8(_) => {
                    let dimensions = image.dimensions();
                    let raw = image.to_rgba8().to_vec();
                    // (raw, dimensions, if srgb { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm }, 4)
                    (raw, dimensions, wgpu::TextureFormat::Rgba8Unorm, 4)
                },
                ImageRgb16(_) => {
                    let dimensions = image.dimensions();
                    let raw = bytemuck::cast_slice(&image.to_rgba16().to_vec()).to_vec();
                    (raw, dimensions, wgpu::TextureFormat::Rgba16Unorm, 8)
                },
                ImageRgba16(_) => {
                    let dimensions = image.dimensions();
                    let raw = bytemuck::cast_slice(&image.to_rgba16().to_vec()).to_vec();
                    (raw, dimensions, wgpu::TextureFormat::Rgba16Unorm, 8)
                },
                // 32F converted to 16F due to filtering issues
                ImageRgb32F(_) => {
                    let dimensions = image.dimensions();
                    let raw = image.to_rgba32f().into_iter().flat_map(|f| half::f16::from_f32(*f).to_ne_bytes()).collect();
                    (raw, dimensions, wgpu::TextureFormat::Rgba16Float, 8)
                },
                ImageRgba32F(_) => {
                    let dimensions = image.dimensions();
                    let raw = image.to_rgba32f().into_iter().flat_map(|f| half::f16::from_f32(*f).to_ne_bytes()).collect();
                    (raw, dimensions, wgpu::TextureFormat::Rgba16Float, 8)
                },
                _ => unimplemented!("invalid texture format"),
            }
        };

        let image_raw = if flip_y {
            image_raw.chunks((bytes_per_pixel * dimensions.0) as usize).rev().flatten().map(|b| *b).collect::<Vec<_>>()
        } else {
            image_raw
        };

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        };    
        
        let texture = renderer.device.create_texture(&desc);

        renderer.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &image_raw,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(bytes_per_pixel * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = renderer.device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        );

        let view = resources.store(view);
        let sampler = resources.store(sampler);
        
        Texture {
            texture,
            size: dimensions.into(),
            view,
            sampler,
        }
    }

    pub fn from_pixel(
        renderer: &Renderer,
        resources: &mut Resources,
        colour: &[u8; 4],
        label: Option<&str>,
        srgb: bool,
    ) -> Texture {
        let size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };

        let format = if srgb {
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        };

        let desc = wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        };    
        
        let texture = renderer.device.create_texture(&desc);

        renderer.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            bytemuck::cast_slice(colour),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * size.width),
                rows_per_image: std::num::NonZeroU32::new(1 * size.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(format),
            ..Default::default()
        });
        let sampler = renderer.device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        );

        let view = resources.store(view);
        let sampler = resources.store(sampler);
        
        Texture {
            texture,
            size: glam::UVec2::ONE,
            view,
            sampler,
        }
    }

    pub(crate) fn binding_types() -> Vec<wgpu::BindingType> {
        vec![
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        ]
    }

    pub(crate) fn binding_resource(&self) -> BindingHolder {
        BindingHolder::Texture(self.view.clone(), self.sampler.clone())
    }
}
