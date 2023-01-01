use image::GenericImageView;

use crate::{renderer::{Renderer, RenderInput, RenderableResource, BindingHolder}, node::NodeDescriptor};

use super::{Resources, Handle};

pub struct CubeMap {
    pub texture: wgpu::Texture,
    pub view: Handle<wgpu::TextureView>,
    pub sampler: Handle<wgpu::Sampler>,
    // bind_group: Handle<wgpu::BindGroup>,
}

impl CubeMap {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    /// From 2x1 equirectangular image
    pub fn from_equirectangular(
        renderer: &Renderer,
        resources: &mut Resources,
        source: &image::DynamicImage,
        label: Option<&str>,
        srgb: bool,
    ) -> Result<CubeMap, CubeError> {
        let (image_raw, dimensions, format, bytes_per_pixel) = {
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

        // let image_raw = image_raw.chunks((bytes_per_pixel * dimensions.0) as usize).rev().flatten().map(|b| *b).collect::<Vec<_>>();
        
        let size = wgpu::Extent3d {
            width: dimensions.0 / 2,
            height: dimensions.1 / 1,
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

        // (dir, tangent, bitangent)
        let cubemap_dirs = [
            (glam::Vec3::X, glam::Vec3::NEG_Z, glam::Vec3::Y),
            (glam::Vec3::NEG_X, glam::Vec3::Z, glam::Vec3::Y),
            (glam::Vec3::NEG_Y, glam::Vec3::X, glam::Vec3::Z),
            (glam::Vec3::Y, glam::Vec3::X, glam::Vec3::NEG_Z),
            (glam::Vec3::Z, glam::Vec3::X, glam::Vec3::Y),
            (glam::Vec3::NEG_Z, glam::Vec3::NEG_X, glam::Vec3::Y),
        ];
        
        fn spherical_to_equirectangular(v: glam::Vec3) -> glam::Vec2 {
            let mut uv = glam::vec2(f32::atan2(v.z, v.x), f32::asin(v.y));
            let inv_atan = glam::vec2(0.1591, 0.3183);
            uv *= inv_atan;
            uv += 0.5;
            return uv;
        }

        let image_chunks = image_raw.chunks(bytes_per_pixel as usize).collect::<Vec<_>>();

        for (i, (dir, tangent, bitangent)) in cubemap_dirs.into_iter().enumerate() {
            let mut face_bytes: Vec<u8> = vec![];

            for cy in 0..size.height {
                let cy = cy as f32 * 2.0 / size.height as f32 - 1.0;
                for cx in 0..size.width {
                    let cx = cx as f32 * 2.0 / size.width as f32 - 1.0;

                    let dir = (dir + cx * tangent + cy * bitangent).normalize();

                    let uv = spherical_to_equirectangular(dir) * glam::vec2(dimensions.0 as f32, dimensions.1 as f32);
                    let uv = glam::uvec2((uv.x.round() as u32).clamp(0, dimensions.0 - 1), (uv.y.round() as u32).clamp(0, dimensions.1 - 1));
                    let index = (uv.x + uv.y * dimensions.0) as usize;
                    
                    let chunk = image_chunks[index];
                    face_bytes.extend(chunk);
                }
            }

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
                &face_bytes,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(bytes_per_pixel * size.width),
                    rows_per_image: std::num::NonZeroU32::new(size.height),
                },
                wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label,
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

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

        let view = resources.store(view);
        let sampler = resources.store(sampler);

        // let bind_group = resources.store(renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label,
        //     layout: &renderer.bind_group_layouts[&BindGroupLayoutType::CubeMap],
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: wgpu::BindingResource::TextureView(&view),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::Sampler(&sampler),
        //         },
        //     ],
        // }));
        
        Ok(CubeMap {
            texture,
            view,
            sampler,
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

        let view = resources.store(view);
        let sampler = resources.store(sampler);

        Ok(CubeMap {
            texture,
            view,
            sampler,
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
        BindingHolder::Texture(self.view.clone(), self.sampler.clone())
    }
}

impl RenderableResource for CubeMap {
    fn render_inputs(&self, _node: &NodeDescriptor, _renderer: &Renderer, _resources: &Resources) -> Vec<RenderInput> {
        vec![RenderInput::BindingResources("cubemap".into(), self.binding_resource())]
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
