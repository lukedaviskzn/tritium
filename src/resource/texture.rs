use image::GenericImageView;

use crate::renderer::{BindGroupLayoutType, Renderer};

use super::{Handle, Resources};

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    bind_group: Handle<wgpu::BindGroup>,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub(crate) fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str) -> Texture {
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
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Texture {
            texture,
            view,
            sampler,
            bind_group: Handle::new_invalid(), // handle never used
        }
    }

    pub fn from_bytes(
        renderer: &Renderer,
        resources: &mut Resources,
        bytes: &[u8],
        label: Option<&str>,
        srgb: bool,
    ) -> Result<Texture, image::ImageError> {
        let img = image::load_from_memory(bytes)?;
        Ok(Texture::from_image(renderer, resources, &img, label, srgb))
    }

    pub fn from_image(
        renderer: &Renderer,
        resources: &mut Resources,
        image: &image::DynamicImage,
        label: Option<&str>,
        srgb: bool,
    ) -> Texture {
        let (image_raw, dimensions, format) = {
            use image::DynamicImage::*;
            match image {
                ImageRgb8(_) => {
                    let dimensions = image.dimensions();
                    let raw = image.to_rgba8().to_vec();
                    (raw, dimensions, if srgb { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm })
                },
                ImageRgba8(_) => {
                    let dimensions = image.dimensions();
                    let raw = image.to_rgba8().to_vec();
                    (raw, dimensions, if srgb { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm })
                },
                ImageRgb16(_) => {
                    let dimensions = image.dimensions();
                    let raw = bytemuck::cast_vec(image.to_rgba16().to_vec());
                    (raw, dimensions, wgpu::TextureFormat::Rgba16Unorm)
                },
                ImageRgba16(_) => {
                    let dimensions = image.dimensions();
                    let raw = bytemuck::cast_vec(image.to_rgba16().to_vec());
                    (raw, dimensions, wgpu::TextureFormat::Rgba16Unorm)
                },
                ImageRgb32F(_) => {
                    let dimensions = image.dimensions();
                    let raw = bytemuck::cast_vec(image.to_rgba32f().to_vec());
                    (raw, dimensions, wgpu::TextureFormat::Rgba32Float)
                },
                ImageRgba32F(_) => {
                    let dimensions = image.dimensions();
                    let raw = bytemuck::cast_vec(image.to_rgba32f().to_vec());
                    (raw, dimensions, wgpu::TextureFormat::Rgba32Float)
                },
                _ => unimplemented!("invalid texture format"),
            }
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
                bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = renderer.device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        );

        let bind_group = resources.store(renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout: &renderer.bind_group_layouts[&BindGroupLayoutType::Texture],
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        }));
        
        Texture {
            texture,
            view,
            sampler,
            bind_group,
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

        let bind_group = resources.store(renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout: &renderer.bind_group_layouts[&BindGroupLayoutType::Texture],
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        }));
        
        Texture {
            texture,
            view,
            sampler,
            bind_group,
        }
    }

    pub(crate) fn bind_group(&self) -> Handle<wgpu::BindGroup> {
        self.bind_group.clone()
    }

    pub fn bind_group_layout_descriptor<'a>() -> wgpu::BindGroupLayoutDescriptor<'a> {
        wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Diffuse Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
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
            label: None,
        }
    }
}
