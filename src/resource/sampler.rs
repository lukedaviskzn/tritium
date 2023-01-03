use crate::renderer::{Renderer, BindingHolder};

use super::{Handle, Resources, Texture, CubeMap};

pub struct Sampler {
    pub(crate) texture: Handle<Texture>,
    pub(crate) sampler: Handle<wgpu::Sampler>,
}

impl Sampler {
    pub fn new_default(renderer: &Renderer, resources: &mut Resources, texture: Handle<Texture>) -> Sampler {
        Sampler::new(renderer, resources, texture, wgpu::AddressMode::Repeat, wgpu::AddressMode::Repeat, wgpu::FilterMode::Linear, wgpu::FilterMode::Nearest, wgpu::FilterMode::Linear)
    }

    pub fn new(
        renderer: &Renderer,
        resources: &mut Resources,
        texture: Handle<Texture>,
        address_mode_u: wgpu::AddressMode,
        address_mode_v: wgpu::AddressMode,
        mag_filter: wgpu::FilterMode,
        min_filter: wgpu::FilterMode,
        mipmap_filter: wgpu::FilterMode,
    ) -> Sampler {
        let sampler = renderer.device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u,
                address_mode_v,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter,
                min_filter,
                mipmap_filter,
                ..Default::default()
            }
        );

        let sampler = resources.store(sampler);
        
        Sampler {
            texture,
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

    pub(crate) fn binding_resources(&self, resources: &Resources) -> [BindingHolder; 2] {
        [
            BindingHolder::Texture(self.texture.get(resources).view.clone()),
            BindingHolder::Sampler(self.sampler.clone()),
        ]
    }
}

pub struct CubeSampler {
    pub(crate) texture: Handle<CubeMap>,
    pub(crate) sampler: Handle<wgpu::Sampler>,
}

impl CubeSampler {
    pub fn new_default(renderer: &Renderer, resources: &mut Resources, texture: Handle<CubeMap>) -> CubeSampler {
        CubeSampler::new(renderer, resources, texture, wgpu::AddressMode::Repeat, wgpu::AddressMode::Repeat, wgpu::FilterMode::Linear, wgpu::FilterMode::Nearest, wgpu::FilterMode::Linear)
    }

    pub fn new(
        renderer: &Renderer,
        resources: &mut Resources,
        texture: Handle<CubeMap>,
        address_mode_u: wgpu::AddressMode,
        address_mode_v: wgpu::AddressMode,
        mag_filter: wgpu::FilterMode,
        min_filter: wgpu::FilterMode,
        mipmap_filter: wgpu::FilterMode,
    ) -> CubeSampler {
        let sampler = renderer.device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u,
                address_mode_v,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter,
                min_filter,
                mipmap_filter,
                ..Default::default()
            }
        );

        let sampler = resources.store(sampler);
        
        CubeSampler {
            texture,
            sampler,
        }
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

    pub(crate) fn binding_resources(&self, resources: &Resources) -> [BindingHolder; 2] {
        [
            BindingHolder::Texture(self.texture.get(resources).view.clone()),
            BindingHolder::Sampler(self.sampler.clone()),
        ]
    }
}
