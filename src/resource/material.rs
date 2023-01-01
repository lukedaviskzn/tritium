use std::num::NonZeroU32;

use crate::{engine::Rgba, renderer::{Renderer, UniformBuffer, BindingHolder}};

use super::{Texture, Handle, Resources};

pub enum AlphaMode {
    Opaque,
    Mask { cutoff: f32 },
    Blend,
}

pub struct Material {
    name: Option<String>,
    double_sided: bool,
    alpha_mode: AlphaMode,
    albedo_texture: Handle<Texture>,
    albedo_buffer: UniformBuffer, // diffuse_colour: Rgba,
    metallic_roughness_texture: Handle<Texture>,
    metallic_factor_buffer: UniformBuffer, // metallic_factor: f32,
    roughness_factor_buffer: UniformBuffer, // roughness_factor: f32,
    normal_texture: Handle<Texture>,
    normal_scale_buffer: UniformBuffer, // normal_scale: f32,
    occlusion_texture: Handle<Texture>,
    occlusion_strength_buffer: UniformBuffer, // normal_strength: f32,
    emissive_texture: Handle<Texture>,
    emissive_factor_buffer: UniformBuffer, // emissive_factor: Rgba,
}

impl Material {
    fn new(
        renderer: &Renderer,
        resources: &mut Resources,
        name: Option<String>,
        double_sided: bool,
        alpha_mode: AlphaMode,
        albedo_texture: Option<Handle<Texture>>,
        albedo: Rgba,
        metallic_roughness_texture: Option<Handle<Texture>>,
        metallic_factor: f32,
        roughness_factor: f32,
        normal_texture: Option<Handle<Texture>>,
        normal_scale: f32,
        occlusion_texture: Option<Handle<Texture>>,
        occlusion_strength: f32,
        emissive_texture: Option<Handle<Texture>>,
        emissive_factor: Rgba,
    ) -> Material {
        let albedo_texture = if let Some(albedo_texture) = albedo_texture {
            albedo_texture
        } else {
            let texture = Texture::from_pixel(renderer, resources, &[255, 255, 255, 255], None, false);
            resources.store(texture)
        };

        let albedo_buffer = UniformBuffer::from_value(renderer, resources, albedo);

        let metallic_roughness_texture = if let Some(metallic_roughness_texture) = metallic_roughness_texture {
            metallic_roughness_texture
        } else {
            // default: metallic: 1.0, roughness: 1.0
            let texture = Texture::from_pixel(renderer, resources, &[255, 255, 0, 0], None, false);
            resources.store(texture)
        };

        let metallic_factor_buffer = UniformBuffer::from_value(renderer, resources, metallic_factor);
        let roughness_factor_buffer = UniformBuffer::from_value(renderer, resources, roughness_factor);

        let normal_texture = if let Some(normal_texture) = normal_texture {
            normal_texture
        } else {
            let texture = Texture::from_pixel(renderer, resources, &[128, 128, 255, 255], None, false);
            resources.store(texture)
        };

        let normal_scale_buffer = UniformBuffer::from_value(renderer, resources, normal_scale);

        let occlusion_texture = if let Some(occlusion_texture) = occlusion_texture {
            occlusion_texture
        } else {
            let texture = Texture::from_pixel(renderer, resources, &[255, 255, 255, 255], None, false);
            resources.store(texture)
        };

        let occlusion_strength_buffer = UniformBuffer::from_value(renderer, resources, occlusion_strength);

        let emissive_texture = if let Some(emissive_texture) = emissive_texture {
            emissive_texture
        } else {
            let texture = Texture::from_pixel(renderer, resources, &[255, 255, 255, 255], None, false);
            resources.store(texture)
        };

        let emissive_factor_buffer = UniformBuffer::from_value(renderer, resources, emissive_factor);

        // let diffuse_texture_res = resources.get(&diffuse_texture).expect("Attempted to create material with invalid diffuse texture handle.");
        // let normal_texture_res = resources.get(&normal_texture).expect("Attempted to create material with invalid normal texture handle.");

        Material {
            name,
            double_sided,
            alpha_mode,
            albedo_texture,
            albedo_buffer,
            metallic_roughness_texture,
            metallic_factor_buffer,
            roughness_factor_buffer,
            normal_texture,
            normal_scale_buffer,
            occlusion_texture,
            occlusion_strength_buffer,
            emissive_texture,
            emissive_factor_buffer,
        }
    }

    pub(crate) fn binding_types() -> Vec<wgpu::BindingType> {
        vec![
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
            // Metallic Roughness Texture
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

    pub(crate) fn binding_resources(&self, resources: &Resources) -> [BindingHolder; 11] {
        let name = match &self.name {
            Some(name) => format!("Material '{}'", name),
            None => "Unnamed material".into(),
        };
        // let diffuse_texture = resources.get(&self.diffuse_texture).expect("Material has invalid diffuse texture handle.");
        // let normal_texture = resources.get(&self.normal_texture).expect("Material has invalid normal texture handle.");
        let albedo_texture = resources.get(&self.albedo_texture).expect(&format!("{} holds invalid base colour texture handle.", name));
        let metallic_roughness_texture = resources.get(&self.metallic_roughness_texture).expect(&format!("{} holds invalid metallic roughness texture handle.", name));
        let normal_texture = resources.get(&self.normal_texture).expect(&format!("{} holds invalid normal texture handle.", name));
        let occlusion_texture = resources.get(&self.occlusion_texture).expect(&format!("{} holds invalid occlusion texture handle.", name));
        let emissive_texture = resources.get(&self.emissive_texture).expect(&format!("{} holds invalid emissive texture handle.", name));
        
        [
            albedo_texture.binding_resource(),
            self.albedo_buffer.binding_resource(),
            metallic_roughness_texture.binding_resource(),
            self.metallic_factor_buffer.binding_resource(),
            self.roughness_factor_buffer.binding_resource(),
            normal_texture.binding_resource(),
            self.normal_scale_buffer.binding_resource(),
            occlusion_texture.binding_resource(),
            self.occlusion_strength_buffer.binding_resource(),
            emissive_texture.binding_resource(),
            self.emissive_factor_buffer.binding_resource(),
        ]
    }

    pub fn builder() -> MaterialBuilder {
        MaterialBuilder::new()
    }
}

pub struct MaterialBuilder {
    name: Option<String>,
    double_sided: bool,
    alpha_mode: AlphaMode,
    albedo_texture: Option<Handle<Texture>>,
    albedo: Rgba,
    metallic_roughness_texture: Option<Handle<Texture>>,
    metallic_factor: f32,
    metallic_set: bool,
    roughness_factor: f32,
    roughness_set: bool,
    normal_texture: Option<Handle<Texture>>,
    normal_scale: f32,
    occlusion_texture: Option<Handle<Texture>>,
    occlusion_strength: f32,
    emissive_texture: Option<Handle<Texture>>,
    emissive_factor: Rgba,
    emissive_set: bool,
}

impl MaterialBuilder {
    fn new() -> MaterialBuilder {
        MaterialBuilder {
            name: None,
            double_sided: false,
            alpha_mode: AlphaMode::Mask { cutoff: 0.5 },
            albedo_texture: None,
            albedo: Rgba::WHITE,
            metallic_roughness_texture: None,
            metallic_factor: 1.0,
            metallic_set: false,
            roughness_factor: 1.0,
            roughness_set: false,
            normal_texture: None,
            normal_scale: 1.0,
            occlusion_texture: None,
            occlusion_strength: 1.0,
            emissive_texture: None,
            emissive_factor: Rgba::WHITE,
            emissive_set: false,
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

    pub fn albedo_texture(mut self, albedo_texture: Handle<Texture>) -> MaterialBuilder {
        self.albedo_texture = Some(albedo_texture);
        self
    }

    pub fn albedo(mut self, albedo: Rgba) -> MaterialBuilder {
        self.albedo = albedo;
        self
    }

    /// Combined metallic and roughness texture. (green: roughness, blue: metallic)
    pub fn metallic_roughness_texture(mut self, metallic_roughness_texture: Handle<Texture>) -> MaterialBuilder {
        self.metallic_roughness_texture = Some(metallic_roughness_texture);
        self.metallic_set = true;
        self.roughness_set = true;
        self
    }

    pub fn metallic_factor(mut self, metallic_factor: f32) -> MaterialBuilder {
        self.metallic_factor = metallic_factor;
        self.metallic_set = true;
        self
    }

    pub fn roughness_factor(mut self, roughness_factor: f32) -> MaterialBuilder {
        self.roughness_factor = roughness_factor;
        self.roughness_set = true;
        self
    }

    pub fn normal_texture(mut self, normal_texture: Handle<Texture>) -> MaterialBuilder {
        self.normal_texture = Some(normal_texture);
        self
    }

    pub fn normal_scale(mut self, normal_scale: f32) -> MaterialBuilder {
        self.normal_scale = normal_scale;
        self
    }

    pub fn occlusion_texture(mut self, occlusion_texture: Handle<Texture>) -> MaterialBuilder {
        self.occlusion_texture = Some(occlusion_texture);
        self
    }

    pub fn occlusion_strength(mut self, occlusion_strength: f32) -> MaterialBuilder {
        self.occlusion_strength = occlusion_strength;
        self
    }

    pub fn emissive_texture(mut self, emissive_texture: Handle<Texture>) -> MaterialBuilder {
        self.emissive_texture = Some(emissive_texture);
        self.emissive_set = true;
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
            self.albedo_texture,
            self.albedo,
            self.metallic_roughness_texture,
            self.metallic_factor,
            self.roughness_factor,
            self.normal_texture,
            self.normal_scale,
            self.occlusion_texture,
            self.occlusion_strength,
            self.emissive_texture,
            self.emissive_factor
        )
    }
}
