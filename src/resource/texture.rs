use std::num::NonZeroU32;

use image::GenericImageView;

use crate::renderer::{Renderer, BindingHolder, Shader, PipelineProperties};

use super::{Handle, Resources};

pub struct Texture {
    pub(crate) texture: wgpu::Texture,
    pub(crate) size: glam::UVec2,
    pub(crate) view: Handle<wgpu::TextureView>,
    // pub(crate) sampler: Handle<wgpu::Sampler>,
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
        dimensions: (u32, u32),
        format: wgpu::TextureFormat,
        label: Option<&str>,
        // srgb: bool,
        flip_y: bool,
    ) -> Texture {
        let format_desc = format.describe();
        if !format_desc.required_features.is_empty() {
            panic!("Given image format has feature requirements not met by renderer.");
        }
        if format_desc.block_dimensions != (1,1) {
            panic!("Compressed textures not supported.");
        }

        // srgb will be calculated in the shader
        let format = match format {
            wgpu::TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8Unorm,
            _ => format,
        };

        let bytes = bytes.to_vec();
        
        let (bytes, format): (Vec<_>, _) = match &format {
            wgpu::TextureFormat::R32Float => {
                let raw = bytes.chunks(4).flat_map(|p| {
                    let value = bytemuck::cast_slice::<_, f32>(p)[0];
                    half::f16::from_f32(value).to_ne_bytes()
                }).collect();
                (raw, wgpu::TextureFormat::R16Float)
            },
            wgpu::TextureFormat::Rg32Float => {
                let raw = bytes.chunks(4).flat_map(|p| {
                    let value = bytemuck::cast_slice::<_, f32>(p)[0];
                    half::f16::from_f32(value).to_ne_bytes()
                }).collect();
                (raw, wgpu::TextureFormat::R16Float)
            },
            wgpu::TextureFormat::Rgba32Float => {
                let raw = bytes.chunks(4).flat_map(|p| {
                    let value = bytemuck::cast_slice::<_, f32>(p)[0];
                    half::f16::from_f32(value).to_ne_bytes()
                }).collect();
                (raw, wgpu::TextureFormat::R16Float)
            },
            _ => (bytes, format),
        };

        let format_desc = format.describe();

        match format_desc.sample_type {
            wgpu::TextureSampleType::Float { filterable: true } => {},
            _ => panic!("Non-filterable textures not supported."),
        };

        let bytes_per_pixel = format_desc.block_size as u32;

        // log::trace!("Creating texture: {:?}", &image_raw[0..4]);

        let bytes = if flip_y {
            bytes.chunks((bytes_per_pixel * dimensions.0) as usize).rev().flatten().map(|b| *b).collect::<Vec<_>>()
        } else {
            bytes
        };

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        
        let source_texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
        });
            
        renderer.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &source_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(bytes_per_pixel * size.width),
                rows_per_image: std::num::NonZeroU32::new(size.height),
            },
            size,
        );

        let dest_texture = Texture::generate_mipmaps(renderer, resources, source_texture, format, size);
        let dest_view = dest_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let dest_view = resources.store(dest_view);
        
        Texture {
            texture: dest_texture,
            size: dimensions.into(),
            view: dest_view,
            // sampler,
        }
    }

    pub fn from_image(
        renderer: &Renderer,
        resources: &mut Resources,
        image: &image::DynamicImage,
        label: Option<&str>,
        // srgb: bool,
        flip_y: bool,
    ) -> Texture {
        let (image_raw, dimensions, format) = {
            use image::DynamicImage::*;
            match image {
                ImageRgb8(_) => {
                    let dimensions = image.dimensions();
                    let raw = image.to_rgba8().to_vec();
                    // (raw, dimensions, if srgb { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm }, 4)
                    (raw, dimensions, wgpu::TextureFormat::Rgba8Unorm)
                },
                ImageRgba8(_) => {
                    let dimensions = image.dimensions();
                    let raw = image.to_rgba8().to_vec();
                    // (raw, dimensions, if srgb { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm }, 4)
                    (raw, dimensions, wgpu::TextureFormat::Rgba8Unorm)
                },
                ImageRgb16(_) => {
                    let dimensions = image.dimensions();
                    let raw = bytemuck::cast_slice(&image.to_rgba16().to_vec()).to_vec();
                    (raw, dimensions, wgpu::TextureFormat::Rgba16Unorm)
                },
                ImageRgba16(_) => {
                    let dimensions = image.dimensions();
                    let raw = bytemuck::cast_slice(&image.to_rgba16().to_vec()).to_vec();
                    (raw, dimensions, wgpu::TextureFormat::Rgba16Unorm)
                },
                // 32F converted to 16F due to filtering issues
                ImageRgb32F(_) => {
                    let dimensions = image.dimensions();
                    let raw = image.to_rgba32f().into_iter().flat_map(|f| half::f16::from_f32(*f).to_ne_bytes()).collect();
                    (raw, dimensions, wgpu::TextureFormat::Rgba16Float)
                },
                ImageRgba32F(_) => {
                    let dimensions = image.dimensions();
                    let raw = image.to_rgba32f().into_iter().flat_map(|f| half::f16::from_f32(*f).to_ne_bytes()).collect();
                    (raw, dimensions, wgpu::TextureFormat::Rgba16Float)
                },
                _ => unimplemented!("invalid texture format"),
            }
        };

        Texture::from_bytes(renderer, resources, &image_raw, dimensions, format, label, flip_y)
    }

    pub fn from_pixel(
        renderer: &Renderer,
        resources: &mut Resources,
        colour: &[u8; 4],
        label: Option<&str>,
        // srgb: bool,
    ) -> Texture {
        let size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };

        // let format = if srgb {
        //     wgpu::TextureFormat::Rgba8UnormSrgb
        // } else {
        //     wgpu::TextureFormat::Rgba8Unorm
        // };

        let format = wgpu::TextureFormat::Rgba8Unorm;

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

        let view = resources.store(view);
        
        Texture {
            texture,
            size: glam::UVec2::ONE,
            view,
        }
    }

    pub(crate) fn binding_types() -> Vec<wgpu::BindingType> {
        vec![
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            // wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        ]
    }

    pub(crate) fn binding_resource(&self) -> BindingHolder {
        BindingHolder::Texture(self.view.clone())
    }

    pub(crate) fn generate_mipmaps(renderer: &Renderer, resources: &mut Resources, source_texture: wgpu::Texture, format: wgpu::TextureFormat, size: wgpu::Extent3d) -> wgpu::Texture {
        let source_view = source_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let source_sampler = renderer.device.create_sampler(
            &wgpu::SamplerDescriptor {
                mag_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }
        );
        
        let shader = if let Some(shader) = resources.get_engine_global_mut::<Shader>("texture::mipmap_pipeline") {
            shader.prepare_pipeline(renderer, PipelineProperties {
                transparent: false, double_sided: false,
                colour_format: format, depth_format: None,
            });
            &*shader
        } else {
            let mut shader = Shader::from_resource(renderer, "pipelines/builtin/texture_mipmaps.ron").expect("Mipmap shader not present.");
            shader.prepare_pipeline(renderer, PipelineProperties {
                transparent: false, double_sided: false,
                colour_format: format, depth_format: None,
            });
            resources.set_engine_global("texture::mipmap_pipeline", shader);
            resources.get_engine_global::<Shader>("texture::mipmap_pipeline").expect("unreachable")
        };

        let mip_level_count = ((size.width as f32).log2().max((size.height as f32).log2()).ceil() as u32).max(1);

        let dest_texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        });

        // let dest_view = dest_texture.create_view(&wgpu::TextureViewDescriptor {
        //     label: None,
        //     mip_level_count: NonZeroU32::new(mip_level_count),
        //     ..Default::default()
        // });

        let bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &shader.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&source_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&source_sampler),
                },
            ],
        });

        let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        
        for i in 0..mip_level_count {
            let mip_view = dest_texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                base_mip_level: i,
                mip_level_count: NonZeroU32::new(1),
                ..Default::default()
            });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &mip_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });
                render_pass.set_pipeline(shader.get_pipeline(PipelineProperties {
                    transparent: false, double_sided: false,
                    colour_format: format, depth_format: None,
                }).unwrap());
                render_pass.set_bind_group(0, &bind_group, &[]);
                render_pass.draw(0..3, 0..1);
            }
        }

        renderer.queue.submit(std::iter::once(encoder.finish()));
        
        dest_texture
    }
}
