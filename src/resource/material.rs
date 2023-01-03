use crate::{engine::Rgba, renderer::{Renderer, UniformBuffer, BindingHolder}};

use super::{Texture, Handle, Resources, Sampler};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlphaMode {
    Opaque,
    Mask { cutoff: f32 },
    Blend,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct AlphaModeUniform {
    blended: u32,
    alpha_cutoff: f32,
}

impl AlphaModeUniform {
    pub fn new(alpha_mode: AlphaMode) -> AlphaModeUniform {
        AlphaModeUniform {
            blended: if alpha_mode == AlphaMode::Blend { 1 } else { 0 },
            alpha_cutoff: match alpha_mode {
                AlphaMode::Opaque => -1.0,
                AlphaMode::Mask { cutoff } => cutoff,
                AlphaMode::Blend => -1.0,
            },
        }
    }
}

pub struct Material {
    name: Option<String>,
    pub(crate) double_sided: bool,
    pub(crate) alpha_mode: AlphaMode,
    alpha_mode_buffer: UniformBuffer,
    // albedo_texture: Handle<Texture>,
    albedo_sampler: Handle<Sampler>,
    albedo_buffer: UniformBuffer, // diffuse_colour: Rgba,
    // metallic_texture: Handle<Texture>,
    metallic_sampler: Handle<Sampler>,
    metallic_factor_buffer: UniformBuffer, // metallic_factor: f32,
    // roughness_texture: Handle<Texture>,
    roughness_sampler: Handle<Sampler>,
    roughness_factor_buffer: UniformBuffer, // roughness_factor: f32,
    // normal_texture: Handle<Texture>,
    normal_sampler: Handle<Sampler>,
    normal_scale_buffer: UniformBuffer, // normal_scale: f32,
    // occlusion_texture: Handle<Texture>,
    occlusion_sampler: Handle<Sampler>,
    occlusion_strength_buffer: UniformBuffer, // normal_strength: f32,
    // emissive_texture: Handle<Texture>,
    emissive_sampler: Handle<Sampler>,
    emissive_factor_buffer: UniformBuffer, // emissive_factor: Rgba,
}

impl Material {
    fn new(
        renderer: &Renderer,
        resources: &mut Resources,
        name: Option<String>,
        double_sided: bool,
        alpha_mode: AlphaMode,
        // albedo_texture: Option<Handle<Texture>>,
        albedo_sampler: Option<Handle<Sampler>>,
        albedo: Rgba,
        // metallic_texture: Option<Handle<Texture>>,
        metallic_sampler: Option<Handle<Sampler>>,
        metallic_factor: f32,
        // roughness_texture: Option<Handle<Texture>>,
        roughness_sampler: Option<Handle<Sampler>>,
        roughness_factor: f32,
        // normal_texture: Option<Handle<Texture>>,
        normal_sampler: Option<Handle<Sampler>>,
        normal_scale: f32,
        // occlusion_texture: Option<Handle<Texture>>,
        occlusion_sampler: Option<Handle<Sampler>>,
        occlusion_strength: f32,
        // emissive_texture: Option<Handle<Texture>>,
        emissive_sampler: Option<Handle<Sampler>>,
        emissive_factor: Rgba,
    ) -> Material {
        let alpha_mode_buffer = UniformBuffer::from_value(renderer, resources, AlphaModeUniform::new(alpha_mode));
        
        let albedo_sampler = if let Some(albedo_sampler) = albedo_sampler {
            albedo_sampler
        } else {
            let texture = Texture::from_pixel(renderer, resources, &[255, 255, 255, 255], None);
            let texture = resources.store(texture);
            let sampler = Sampler::new_default(renderer, resources, texture);
            resources.store(sampler)
        };

        let albedo_buffer = UniformBuffer::from_value(renderer, resources, albedo);

        let metallic_sampler = if let Some(metallic_sampler) = metallic_sampler {
            metallic_sampler
        } else {
            // default: metallic: 1.0, i.e. defer to roughness factor
            let texture = Texture::from_pixel(renderer, resources, &[0, 0, 255, 255], None);
            let texture = resources.store(texture);
            let sampler = Sampler::new_default(renderer, resources, texture);
            resources.store(sampler)
        };

        let metallic_factor_buffer = UniformBuffer::from_value(renderer, resources, metallic_factor);

        let roughness_sampler = if let Some(roughness_sampler) = roughness_sampler {
            roughness_sampler
        } else {
            let texture = Texture::from_pixel(renderer, resources, &[0, 255, 0, 255], None);
            let texture = resources.store(texture);
            let sampler = Sampler::new_default(renderer, resources, texture);
            resources.store(sampler)
        };
        
        let roughness_factor_buffer = UniformBuffer::from_value(renderer, resources, roughness_factor);

        let normal_sampler = if let Some(normal_sampler) = normal_sampler {
            normal_sampler
        } else {
            let texture = Texture::from_pixel(renderer, resources, &[128, 128, 255, 255], None);
            let texture = resources.store(texture);
            let sampler = Sampler::new_default(renderer, resources, texture);
            resources.store(sampler)
        };

        let normal_scale_buffer = UniformBuffer::from_value(renderer, resources, normal_scale);

        let occlusion_sampler = if let Some(occlusion_sampler) = occlusion_sampler {
            occlusion_sampler
        } else {
            let texture = Texture::from_pixel(renderer, resources, &[255, 255, 255, 255], None);
            let texture = resources.store(texture);
            let sampler = Sampler::new_default(renderer, resources, texture);
            resources.store(sampler)
        };

        let occlusion_strength_buffer = UniformBuffer::from_value(renderer, resources, occlusion_strength);

        let emissive_sampler = if let Some(emissive_sampler) = emissive_sampler {
            emissive_sampler
        } else {
            let texture = Texture::from_pixel(renderer, resources, &[255, 255, 255, 255], None);
            let texture = resources.store(texture);
            let sampler = Sampler::new_default(renderer, resources, texture);
            resources.store(sampler)
        };

        let emissive_factor_buffer = UniformBuffer::from_value(renderer, resources, emissive_factor);

        Material {
            name,
            double_sided,
            alpha_mode,
            alpha_mode_buffer,
            // albedo_texture,
            albedo_sampler,
            albedo_buffer,
            // metallic_texture,
            metallic_sampler,
            metallic_factor_buffer,
            // roughness_texture,
            roughness_sampler,
            roughness_factor_buffer,
            // normal_texture,
            normal_sampler,
            normal_scale_buffer,
            // occlusion_texture,
            occlusion_sampler,
            occlusion_strength_buffer,
            // emissive_texture,
            emissive_sampler,
            emissive_factor_buffer,
        }
    }

    pub(crate) fn binding_types() -> Vec<wgpu::BindingType> {
        vec![
            // Alpha Mode
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            // Base Colour Texture
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            // Metallic Texture
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            // Roughness Texture
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            // Normal Texture
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            // Occlusion Texture
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            // Emissive Texture
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        ]
    }

    pub(crate) fn binding_resources(&self, resources: &Resources) -> Vec<BindingHolder> {
        let albedo_sampler = self.albedo_sampler.get(resources);
        let metallic_sampler = self.metallic_sampler.get(resources);
        let roughness_sampler = self.roughness_sampler.get(resources);
        let normal_sampler = self.normal_sampler.get(resources);
        let occlusion_sampler= self.occlusion_sampler.get(resources);
        let emissive_sampler = self.emissive_sampler.get(resources);
        
        let mut binding_resources = vec![];
        
        binding_resources.push(self.alpha_mode_buffer.binding_resource());
        binding_resources.extend(albedo_sampler.binding_resources(resources));
        binding_resources.push(self.albedo_buffer.binding_resource());
        binding_resources.extend(metallic_sampler.binding_resources(resources));
        binding_resources.push(self.metallic_factor_buffer.binding_resource());
        binding_resources.extend(roughness_sampler.binding_resources(resources));
        binding_resources.push(self.roughness_factor_buffer.binding_resource());
        binding_resources.extend(normal_sampler.binding_resources(resources));
        binding_resources.push(self.normal_scale_buffer.binding_resource());
        binding_resources.extend(occlusion_sampler.binding_resources(resources));
        binding_resources.push(self.occlusion_strength_buffer.binding_resource());
        binding_resources.extend(emissive_sampler.binding_resources(resources));
        binding_resources.push(self.emissive_factor_buffer.binding_resource());
        
        binding_resources
    }

    pub fn builder() -> MaterialBuilder {
        MaterialBuilder::new()
    }
}

pub struct MaterialBuilder {
    name: Option<String>,
    double_sided: bool,
    alpha_mode: AlphaMode,
    // albedo_texture: Option<Handle<Texture>>,
    albedo_sampler: Option<Handle<Sampler>>,
    albedo: Rgba,
    // metallic_texture: Option<Handle<Texture>>,
    metallic_sampler: Option<Handle<Sampler>>,
    metallic_factor: f32,
    metallic_set: bool,
    // roughness_texture: Option<Handle<Texture>>,
    roughness_sampler: Option<Handle<Sampler>>,
    roughness_factor: f32,
    roughness_set: bool,
    // normal_texture: Option<Handle<Texture>>,
    normal_sampler: Option<Handle<Sampler>>,
    normal_scale: f32,
    // occlusion_texture: Option<Handle<Texture>>,
    occlusion_sampler: Option<Handle<Sampler>>,
    occlusion_strength: f32,
    // emissive_texture: Option<Handle<Texture>>,
    emissive_sampler: Option<Handle<Sampler>>,
    emissive_factor: Rgba,
    emissive_set: bool,
}

impl MaterialBuilder {
    fn new() -> MaterialBuilder {
        MaterialBuilder {
            name: None,
            double_sided: false,
            alpha_mode: AlphaMode::Mask { cutoff: 0.5 },
            // albedo_texture: None,
            albedo_sampler: None,
            albedo: Rgba::WHITE,
            // metallic_texture: None,
            metallic_sampler: None,
            metallic_factor: 1.0,
            metallic_set: false,
            // roughness_texture: None,
            roughness_sampler: None,
            roughness_factor: 1.0,
            roughness_set: false,
            // normal_texture: None,
            normal_sampler: None,
            normal_scale: 1.0,
            // occlusion_texture: None,
            occlusion_sampler: None,
            occlusion_strength: 1.0,
            // emissive_texture: None,
            emissive_sampler: None,
            emissive_factor: Rgba::WHITE,
            emissive_set: false,
            // height_texture: None,
            // height_scale: 1.0,
        }
    }
    
    pub fn name(mut self, name: &str) -> MaterialBuilder {
        self.name = Some(name.to_owned());
        self
    }

    pub fn double_sided(mut self, double_sided: bool) -> MaterialBuilder {
        self.double_sided = double_sided;
        self
    }

    pub fn alpha_mode(mut self, alpha_mode: AlphaMode) -> MaterialBuilder {
        self.alpha_mode = alpha_mode;
        self
    }

    // pub fn albedo_texture(mut self, albedo_texture: Handle<Texture>) -> MaterialBuilder {
    //     self.albedo_texture = Some(albedo_texture);
    //     self
    // }

    pub fn albedo_sampler(mut self, albedo_sampler: Handle<Sampler>) -> MaterialBuilder {
        self.albedo_sampler = Some(albedo_sampler);
        self
    }

    pub fn albedo(mut self, albedo: Rgba) -> MaterialBuilder {
        self.albedo = albedo;
        self
    }

    /// Metallic value sampled from blue channel
    // pub fn metallic_texture(mut self, metallic_texture: Handle<Texture>) -> MaterialBuilder {
    //     self.metallic_texture = Some(metallic_texture);
    //     self.metallic_set = true;
    //     self.roughness_set = true;
    //     self
    // }

    /// Metallic value sampled from blue channel
    pub fn metallic_sampler(mut self, metallic_sampler: Handle<Sampler>) -> MaterialBuilder {
        self.metallic_sampler = Some(metallic_sampler);
        self
    }

    pub fn metallic_factor(mut self, metallic_factor: f32) -> MaterialBuilder {
        self.metallic_factor = metallic_factor;
        self.metallic_set = true;
        self
    }

    /// Roughness sampled from green channel
    // pub fn roughness_texture(mut self, roughness_texture: Handle<Texture>) -> MaterialBuilder {
    //     self.roughness_texture = Some(roughness_texture);
    //     self.metallic_set = true;
    //     self.roughness_set = true;
    //     self
    // }

    /// Roughness sampled from green channel
    pub fn roughness_sampler(mut self, roughness_sampler: Handle<Sampler>) -> MaterialBuilder {
        self.roughness_sampler = Some(roughness_sampler);
        self
    }

    pub fn roughness_factor(mut self, roughness_factor: f32) -> MaterialBuilder {
        self.roughness_factor = roughness_factor;
        self.roughness_set = true;
        self
    }

    // pub fn normal_texture(mut self, normal_texture: Handle<Texture>) -> MaterialBuilder {
    //     self.normal_texture = Some(normal_texture);
    //     self
    // }

    pub fn normal_sampler(mut self, normal_sampler: Handle<Sampler>) -> MaterialBuilder {
        self.normal_sampler = Some(normal_sampler);
        self
    }

    pub fn normal_scale(mut self, normal_scale: f32) -> MaterialBuilder {
        self.normal_scale = normal_scale;
        self
    }

    /// Occlusion sampled from red channel
    // pub fn occlusion_texture(mut self, occlusion_texture: Handle<Texture>) -> MaterialBuilder {
    //     self.occlusion_texture = Some(occlusion_texture);
    //     self
    // }

    /// Occlusion sampled from red channel
    pub fn occlusion_sampler(mut self, occlusion_sampler: Handle<Sampler>) -> MaterialBuilder {
        self.occlusion_sampler = Some(occlusion_sampler);
        self
    }

    pub fn occlusion_strength(mut self, occlusion_strength: f32) -> MaterialBuilder {
        self.occlusion_strength = occlusion_strength;
        self
    }

    // pub fn emissive_texture(mut self, emissive_texture: Handle<Texture>) -> MaterialBuilder {
    //     self.emissive_texture = Some(emissive_texture);
    //     self.emissive_set = true;
    //     self
    // }

    pub fn emissive_sampler(mut self, emissive_sampler: Handle<Sampler>) -> MaterialBuilder {
        self.emissive_sampler = Some(emissive_sampler);
        self
    }

    pub fn emissive_factor(mut self, emissive_factor: Rgba) -> MaterialBuilder {
        self.emissive_factor = emissive_factor;
        self.emissive_set = true;
        self
    }

    pub fn build(mut self, renderer: &Renderer, resources: &mut Resources) -> Material {
        if !self.metallic_set {
            self.metallic_factor = 0.0;
        }
        if !self.roughness_set {
            self.roughness_factor = 0.5;
        }
        if !self.emissive_set {
            self.emissive_factor = Rgba::TRANSPARENT_BLACK;
        }

        Material::new(
            renderer,
            resources,
            self.name,
            self.double_sided,
            self.alpha_mode,
            // self.albedo_texture,
            self.albedo_sampler,
            self.albedo,
            // self.metallic_texture,
            self.metallic_sampler,
            self.metallic_factor,
            // self.roughness_texture,
            self.roughness_sampler,
            self.roughness_factor,
            // self.normal_texture,
            self.normal_sampler,
            self.normal_scale,
            // self.occlusion_texture,
            self.occlusion_sampler,
            self.occlusion_strength,
            // self.emissive_texture,
            self.emissive_sampler,
            self.emissive_factor,
        )
    }
}
