use wgpu::util::DeviceExt;

use crate::{engine::Rgba, renderer::{Renderer, BindGroupLayoutType}};

use super::{Texture, Handle, Resources};

pub struct Material {
    pub name: String,
    pub diffuse_texture: Handle<Texture>,
    pub diffuse_colour: Rgba,
    // diffuse_colour_buffer: wgpu::Buffer,
    pub normal_texture: Handle<Texture>,
    pub normal_factor: f32,
    // normal_factor_buffer: wgpu::Buffer,
    bind_group: Handle<wgpu::BindGroup>,
}

impl Material {
    pub fn new(
        renderer: &Renderer,
        // device: &wgpu::Device,
        resources: &mut Resources,
        name: &str,
        diffuse_texture: &Handle<Texture>,
        diffuse_colour: Rgba,
        normal_texture: &Handle<Texture>,
        normal_factor: f32,
        // material_layout: &wgpu::BindGroupLayout,
    ) -> Material {
        let diffuse_colour_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[diffuse_colour]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let normal_factor_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[normal_factor]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let diffuse_texture_res = resources.get(diffuse_texture).unwrap();
        let normal_texture_res = resources.get(normal_texture).unwrap();

        let bind_group = resources.store(renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(name),
            layout: &renderer.bind_group_layouts[&BindGroupLayoutType::Material],
            entries: &[
                // Diffuse
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_res.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture_res.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &diffuse_colour_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                // Normals
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&normal_texture_res.view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&normal_texture_res.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &normal_factor_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        }));

        Material {
            name: name.into(),
            diffuse_texture: diffuse_texture.clone(),
            diffuse_colour,
            // diffuse_colour_buffer,
            normal_texture: normal_texture.clone(),
            normal_factor,
            // normal_factor_buffer,
            bind_group,
        }
    }

    pub(crate) fn bind_group(&self) -> Handle<wgpu::BindGroup> {
        self.bind_group.clone()
    }

    pub fn bind_group_layout_descriptor<'a>() -> wgpu::BindGroupLayoutDescriptor<'a> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Material Bind Group Layout"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Normal Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        }
    }
}
