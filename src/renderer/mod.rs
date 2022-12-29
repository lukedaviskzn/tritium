use std::{collections::HashMap, any::TypeId};

use winit::window::Window;

mod pipeline;
mod window;
mod uniform;
mod vertex;

pub use pipeline::*;
pub use window::*;
pub use uniform::*;
pub use vertex::*;

use crate::{resource::{Material, Handle, Texture, CubeMap, Resources, Mesh}, node::{NodeDescriptor, Node, Component}, util::AsAny};

pub struct Renderer {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) window: WindowAdapter,
    // pub(crate) bind_group_layouts: HashMap<BindGroupLayoutType, wgpu::BindGroupLayout>,
    // pub(crate) vertex_layouts: HashMap<VertexLayoutType, wgpu::VertexBufferLayout<'a>>,
}

impl Renderer {
    pub async fn new(window: Window, vsync: bool) -> Renderer {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }).await.unwrap();

        let (device, queue) = adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("device"),
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
        }, None).await.unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: if vsync { wgpu::PresentMode::AutoVsync } else { wgpu::PresentMode::AutoNoVsync },
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let depth_texture = Texture::create_depth_texture(&device, &config, "Depth Texture");

        let window = WindowAdapter {
            window,
            surface,
            config,
            size,
            depth_texture,
            focused: false,
            vsync,
        };

        // let bind_group_layouts = {
        //     let material = device.create_bind_group_layout(&Material::bind_group_layout_descriptor());
        //     let texture = device.create_bind_group_layout(&Texture::bind_group_layout_descriptor());
        //     let cubemap = device.create_bind_group_layout(&CubeMap::bind_group_layout_descriptor());
        //     let uniform = device.create_bind_group_layout(&Uniform::<()>::bind_group_layout_descriptor());
        //     let storage = device.create_bind_group_layout(&StorageBuffer::<()>::bind_group_layout_descriptor());
        //     let storage_array = device.create_bind_group_layout(&StorageBuffer::<()>::array_bind_group_layout_descriptor());
            
        //     hashmap!{
        //         BindGroupLayoutType::Material => material,
        //         BindGroupLayoutType::Texture => texture,
        //         BindGroupLayoutType::CubeMap => cubemap,
        //         BindGroupLayoutType::Uniform => uniform,
        //         BindGroupLayoutType::Storage => storage,
        //         BindGroupLayoutType::StorageArray => storage_array,
        //     }
        // };

        Renderer {
            device,
            queue,
            window,
            // bind_group_layouts,
            // vertex_layouts,
        }
    }

    pub fn on_resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.window.resize(&self.device, new_size);
    }

    pub fn reconfigure_surface(&mut self) {
        self.window.reconfigure(&self.device);
    }
}

pub struct QueuedRenderObject {
    pub shader: Handle<Shader>,
    pub(crate) vertex_buffer: Handle<wgpu::Buffer>,
    pub(crate) index_buffer: Handle<wgpu::Buffer>,
    // pub(crate) bind_groups: Vec<Handle<wgpu::BindGroup>>,
    pub(crate) bind_group: wgpu::BindGroup,
    pub num_indices: u32,
}

// pub struct RenderInput {
//     pub(crate) name: String,
//     pub(crate) storage: RenderInputStorage,
// }

// impl RenderInput {
//     pub fn new(name: &str, storage: RenderInputStorage) -> RenderInput {
//         RenderInput {
//             name: name.into(),
//             storage,
//         }
//     }
// }

// #[derive(Debug, Clone)]
pub enum RenderInput {
    Shader(Handle<Shader>),
    Mesh {
        vertex_buffer: Handle<wgpu::Buffer>,
        index_buffer: Handle<wgpu::Buffer>,
        material: Option<Handle<Material>>,
        num_elements: u32,
    },
    // BindGroup(String, Handle<wgpu::BindGroup>),
    SceneInput(String, SceneInputItem),

    BindingResources(String, BindingHolder),

    // Buffer(String, UniformBuffer),
    // StorageBuffer(String, StorageBuffer),
    // Texture(String, Handle<wgpu::TextureView>, Handle<wgpu::Sampler>),
}

#[derive(Debug, Clone)]
pub struct SceneInputItem {
    pub(crate) data: Vec<u8>,
    pub(crate) typeid: TypeId,
}

impl SceneInputItem {
    pub fn new<T: bytemuck::Pod + bytemuck::Zeroable>(value: T) -> SceneInputItem {
        SceneInputItem {
            data: bytemuck::cast_slice(&[value]).to_vec(),
            typeid: TypeId::of::<T>(),
        }
    }
}

pub trait Renderable {
    fn render_inputs(&self, node: &NodeDescriptor, renderer: &Renderer, resources: &mut Resources) -> Vec<RenderInput>;
}

pub trait RenderableResource {
    fn render_inputs(&self, node: &NodeDescriptor, renderer: &Renderer, resources: &Resources) -> Vec<RenderInput>;
}

/// Invisible components and their chilcren cannot be accessed at all in the extraction and render stages. For example, one cannot 
/// set 'current_camera' to an invisible camera, or camera with an invisible parent.
pub struct Invisible;

impl AsAny for Invisible {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl Component for Invisible {}
