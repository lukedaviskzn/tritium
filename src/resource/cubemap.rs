use image::GenericImageView;

use crate::{renderer::{Renderer, RenderInput, RenderableResource, BindingHolder, Shader, PipelineProperties, UniformBuffer}, node::NodeDescriptor, resource::{Texture, Model}};

use super::{Resources, Handle, Sampler};

pub struct CubeMap {
    pub texture: wgpu::Texture,
    pub view: Handle<wgpu::TextureView>,
    // pub sampler: Handle<wgpu::Sampler>,
    // bind_group: Handle<wgpu::BindGroup>,
}

impl CubeMap {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    /// From 2x1 equirectangular image, 2048x1024 texture will result in 512x512x6 cubemap. (cubemap width = source width / 4, cubemap height = source height / 2)
    pub fn from_image_equirectangular(
        renderer: &Renderer,
        resources: &mut Resources,
        source: &image::DynamicImage,
        label: Option<&str>,
        // srgb: bool,
    ) -> Result<CubeMap, CubeError> {
        let (image_raw, dimensions, format, bytes_per_pixel) = {
            let srgb = false;
            use image::DynamicImage::*;
            match source {
                ImageRgb8(_) => {
                    let dimensions = source.dimensions();
                    let raw = source.to_rgba8().to_vec();
                    (raw, dimensions, if srgb { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm }, 4)
                },
                ImageRgba8(_) => {
                    let dimensions = source.dimensions();
                    let raw = source.to_rgba8().to_vec();
                    (raw, dimensions, if srgb { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm }, 4)
                },
                ImageRgb16(_) => {
                    let dimensions = source.dimensions();
                    let raw = bytemuck::cast_slice(&source.to_rgba16().to_vec()).to_vec();
                    (raw, dimensions, wgpu::TextureFormat::Rgba16Unorm, 8)
                },
                ImageRgba16(_) => {
                    let dimensions = source.dimensions();
                    let raw = bytemuck::cast_slice(&source.to_rgba16().to_vec()).to_vec();
                    (raw, dimensions, wgpu::TextureFormat::Rgba16Unorm, 8)
                },
                // 32F converted to 16F due to filtering issues
                ImageRgb32F(_) => {
                    let dimensions = source.dimensions();
                    let raw = source.to_rgba32f().into_iter().flat_map(|f| half::f16::from_f32(*f).to_ne_bytes()).collect();
                    (raw, dimensions, wgpu::TextureFormat::Rgba16Float, 8)
                },
                ImageRgba32F(_) => {
                    let dimensions = source.dimensions();
                    let raw = source.to_rgba32f().into_iter().flat_map(|f| half::f16::from_f32(*f).to_ne_bytes()).collect();
                    (raw, dimensions, wgpu::TextureFormat::Rgba16Float, 8)
                },
                _ => unimplemented!("invalid texture format"),
            }
        };

        let source = Texture::from_bytes(renderer, resources, &image_raw, dimensions, format, None, true);
        let source = resources.store(source);
        let source = Sampler::new_default(renderer, resources, source);

        let size = wgpu::Extent3d {
            width: dimensions.0 / 4,
            height: dimensions.1 / 2,
            depth_or_array_layers: 6,
        };
        
        // let mip_level_count = ((size.width as f32).log2().max((size.height as f32).log2()).ceil() as u32).max(1);
        let mip_level_count = 1;
        
        let cubemap_texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        // (dir, tangent, bitangent)
        let cubemap_proj = glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_2, 1.0, 0.1, 10.0);
        let cubemap_viewproj = [
            cubemap_proj * glam::Mat4::look_at_rh(glam::Vec3::ZERO, glam::Vec3::X, glam::Vec3::Y),
            cubemap_proj * glam::Mat4::look_at_rh(glam::Vec3::ZERO, glam::Vec3::NEG_X, glam::Vec3::Y),
            cubemap_proj * glam::Mat4::look_at_rh(glam::Vec3::ZERO, glam::Vec3::Y, glam::Vec3::Z),
            cubemap_proj * glam::Mat4::look_at_rh(glam::Vec3::ZERO, glam::Vec3::NEG_Y, glam::Vec3::NEG_Z),
            cubemap_proj * glam::Mat4::look_at_rh(glam::Vec3::ZERO, glam::Vec3::NEG_Z, glam::Vec3::Y),
            cubemap_proj * glam::Mat4::look_at_rh(glam::Vec3::ZERO, glam::Vec3::Z, glam::Vec3::Y),
        ];

        let index = PipelineProperties {
            transparent: false,
            double_sided: false,
            colour_format: format,
            depth_format: None,
        };

        let mut pipeline = Shader::from_resource(renderer, "pipelines/builtin/cubemap_equirectangular.ron").expect("Cubemap equirectangular shader not found.");
        pipeline.prepare_pipeline(renderer, index);

        let cube_model = Model::new_inverted_cube(renderer, resources, None);

        // let image_chunks = image_raw.chunks(bytes_per_pixel as usize).collect::<Vec<_>>();

        for (i, matrix) in cubemap_viewproj.into_iter().enumerate() {
            let face_texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            });

            let face_view = face_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let uniform = UniformBuffer::from_value(renderer, resources, matrix);

            let bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &pipeline.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&source.texture.get(resources).view.get(resources)),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&source.sampler.get(resources)),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: uniform.buffer.get(resources).as_entire_binding(),
                    },
                ],
            });

            let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &face_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(pipeline.get_pipeline(index).unwrap());
                render_pass.set_vertex_buffer(0, cube_model.meshes[0].vertex_buffer.get(resources).slice(..));
                render_pass.set_index_buffer(cube_model.meshes[0].index_buffer.get(resources).slice(..), wgpu::IndexFormat::Uint32);
                render_pass.set_bind_group(0, &bind_group, &[]);

                render_pass.draw_indexed(0..cube_model.meshes[0].num_elements, 0, 0..1);
            }
            
            renderer.queue.submit(std::iter::once(encoder.finish()));

            let face_texture = Texture::generate_mipmaps(renderer, resources, face_texture, format, wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            });

            let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            
            for mip in 0..mip_level_count {
                encoder.copy_texture_to_texture(
                    wgpu::ImageCopyTexture {
                        texture: &face_texture,
                        mip_level: mip,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::ImageCopyTexture {
                        texture: &cubemap_texture,
                        mip_level: mip,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: i as u32,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width: size.width / 2_u32.pow(mip),
                        height: size.height / 2_u32.pow(mip),
                        depth_or_array_layers: 1,
                    }
                );
            }
            
            renderer.queue.submit(std::iter::once(encoder.finish()));
        }

        let view = cubemap_texture.create_view(&wgpu::TextureViewDescriptor {
            label,
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        let view = resources.store(view);
        
        Ok(CubeMap {
            texture: cubemap_texture,
            view,
            // sampler,
            // bind_group,
        })
    }

    pub fn from_images(
        renderer: &Renderer,
        resources: &mut Resources,
        source: Cube<&image::DynamicImage>,
        label: Option<&str>,
        srgb: bool,
    ) -> Result<CubeMap, CubeError> {
        let (faces, dimensions, format) = {
            use image::DynamicImage::*;

            let faces: [&image::DynamicImage; 6] = source.into();
            let faces = faces.map(|face| {
                let data = match face {
                    ImageRgb8(_) => {
                        let dimensions = face.dimensions();
                        let raw = face.to_rgba8().to_vec();
                        ImageRawInfo {
                            raw,
                            dimensions,
                            format: if srgb { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm },
                        }
                    },
                    ImageRgba8(_) => {
                        let dimensions = face.dimensions();
                        let raw = face.to_rgba8().to_vec();
                        ImageRawInfo {
                            raw,
                            dimensions,
                            format: if srgb { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm },
                        }
                    },
                    ImageRgb16(_) => {
                        let dimensions = face.dimensions();
                        let raw = bytemuck::cast_vec(face.to_rgba16().to_vec());
                        ImageRawInfo {
                            raw,
                            dimensions,
                            format: wgpu::TextureFormat::Rgba16Unorm,
                        }
                    },
                    ImageRgba16(_) => {
                        let dimensions = face.dimensions();
                        let raw = bytemuck::cast_vec(face.to_rgba16().to_vec());
                        ImageRawInfo {
                            raw,
                            dimensions,
                            format: wgpu::TextureFormat::Rgba16Unorm,
                        }
                    },
                    ImageRgb32F(_) => {
                        let dimensions = face.dimensions();
                        let raw = bytemuck::cast_vec(face.to_rgba32f().to_vec());
                        ImageRawInfo {
                            raw,
                            dimensions,
                            format: wgpu::TextureFormat::Rgba32Float,
                        }
                    },
                    ImageRgba32F(_) => {
                        let dimensions = face.dimensions();
                        let raw = bytemuck::cast_vec(face.to_rgba32f().to_vec());
                        ImageRawInfo {
                            raw,
                            dimensions,
                            format: wgpu::TextureFormat::Rgba32Float,
                        }
                    },
                    _ => unimplemented!("invalid texture format"),
                };
                data
            });

            // Check matching formats and dimensions
            let (dimensions, format) = {
                let mut face_iter = faces.iter();

                let (first_dimension, first_format) = {
                    let face = face_iter.next().unwrap();
                    (face.dimensions, face.format)
                };
                for face in face_iter {
                    if face.dimensions != first_dimension {
                        return Err(CubeError::DimensionMismatch);
                    }
                    if face.format != first_format {
                        return Err(CubeError::TextureFormatMismatch);
                    }
                }

                (first_dimension, first_format)
            };

            let mut images = vec![];

            for face in faces {
                images.push(face.raw);
            }

            (images, dimensions, format)
        };

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 6,
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

        for (i, image_raw) in faces.iter().enumerate() {
            renderer.queue.write_texture(
                wgpu::ImageCopyTexture {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                },
                &image_raw,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
                    rows_per_image: std::num::NonZeroU32::new(dimensions.1),
                },
                wgpu::Extent3d {
                    width: dimensions.0,
                    height: dimensions.1,
                    depth_or_array_layers: 1,
                },
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label,
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        // let sampler = renderer.device.create_sampler(
        //     &wgpu::SamplerDescriptor {
        //         address_mode_u: wgpu::AddressMode::ClampToEdge,
        //         address_mode_v: wgpu::AddressMode::ClampToEdge,
        //         address_mode_w: wgpu::AddressMode::ClampToEdge,
        //         mag_filter: wgpu::FilterMode::Linear,
        //         min_filter: wgpu::FilterMode::Nearest,
        //         mipmap_filter: wgpu::FilterMode::Nearest,
        //         ..Default::default()
        //     }
        // );

        let view = resources.store(view);
        // let sampler = resources.store(sampler);

        Ok(CubeMap {
            texture,
            view,
            // sampler,
            // bind_group,
        })
    }

    pub(crate) fn binding_types() -> Vec<wgpu::BindingType> {
        vec![
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::Cube,
                multisampled: false,
            },
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        ]
    }

    pub(crate) fn binding_resource(&self) -> BindingHolder {
        BindingHolder::Texture(self.view.clone())
        // BindingHolder::Sampler(self.sampler.clone()),
    }
}

impl RenderableResource for CubeMap {
    fn render_inputs(&self, _node: &NodeDescriptor, _renderer: &Renderer, _resources: &Resources) -> Vec<RenderInput> {
        vec![RenderInput::BindingResources("cubemap".into(), vec![self.binding_resource()])]
    }
}

pub struct Cube<T> {
    pub pos_x: T,
    pub neg_x: T,
    pub pos_y: T,
    pub neg_y: T,
    pub pos_z: T,
    pub neg_z: T,
}

// impl<T> Cube<T> {
//     pub fn faces(&self) -> [&T; 6] {
//     }
// }

impl<T> From<[T; 6]> for Cube<T> {
    fn from(faces: [T; 6]) -> Self {
        let [pos_x, neg_x, pos_y, neg_y, pos_z, neg_z] = faces;
        Cube {
            pos_x,
            neg_x,
            pos_y,
            neg_y,
            pos_z,
            neg_z,
        }
    }
}

impl<T> Into<[T; 6]> for Cube<T> {
    fn into(self) -> [T; 6] {
        [
            self.pos_x,
            self.neg_x,
            self.pos_y,
            self.neg_y,
            self.pos_z,
            self.neg_z,
        ]
    }
}

struct ImageRawInfo {
    pub raw: Vec<u8>,
    pub dimensions: (u32, u32),
    pub format: wgpu::TextureFormat,
}

#[derive(Debug)]
pub enum CubeError {
    DimensionMismatch,
    TextureFormatMismatch,
}
