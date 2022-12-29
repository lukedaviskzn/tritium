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

    pub fn from_image(
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

    // pub(crate) fn bind_group(&self) -> Handle<wgpu::BindGroup> {
    //     self.bind_group.clone()
    // }

    // pub fn bind_group_layout_descriptor<'a>() -> wgpu::BindGroupLayoutDescriptor<'a> {
    //     wgpu::BindGroupLayoutDescriptor {
    //         entries: &[
    //             // Diffuse Texture
    //             wgpu::BindGroupLayoutEntry {
    //                 binding: 0,
    //                 visibility: wgpu::ShaderStages::FRAGMENT,
    //                 ty: wgpu::BindingType::Texture {
    //                     sample_type: wgpu::TextureSampleType::Float { filterable: true },
    //                     view_dimension: wgpu::TextureViewDimension::Cube,
    //                     multisampled: false,
    //                 },
    //                 count: None,
    //             },
    //             wgpu::BindGroupLayoutEntry {
    //                 binding: 1,
    //                 visibility: wgpu::ShaderStages::FRAGMENT,
    //                 ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
    //                 count: None,
    //             },
    //         ],
    //         label: None,
    //     }
    // }

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
        // vec![RenderInput::new("cubemap", RenderInputStorage::BindGroup(self.bind_group()))]
        // vec![RenderInput::BindGroup("cubemap".into(), self.bind_group())]
        // vec![RenderInput::Texture("cubemap".into(), self.view.clone(), self.sampler.clone())]

        // let generator = TextureBindingGenerator {
        //     view: self.view,
        //     sampler: self.sampler,
        // };

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
