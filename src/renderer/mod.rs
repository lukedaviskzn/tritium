use std::collections::HashMap;

use winit::window::Window;

mod pipeline;
mod window;
mod uniform;
mod vertex;

pub use pipeline::*;
pub use window::*;
pub use uniform::*;
pub use vertex::*;

use crate::{resource::{Material, Handle, Texture, CubeMap, Resources}, world::NodeDescriptor};

pub struct Renderer {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) window: WindowAdapter,
    pub(crate) bind_group_layouts: HashMap<BindGroupLayoutType, wgpu::BindGroupLayout>,
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

        let bind_group_layouts = {
            let material_bind_group_layout = device.create_bind_group_layout(&Material::bind_group_layout_descriptor());
            let uniform_bind_group_layout = device.create_bind_group_layout(&Uniform::<()>::bind_group_layout_descriptor());
            let texture_bind_group_layout = device.create_bind_group_layout(&Texture::bind_group_layout_descriptor());
            let cubemap_bind_group_layout = device.create_bind_group_layout(&CubeMap::bind_group_layout_descriptor());
            
            hashmap!{
                BindGroupLayoutType::Material => material_bind_group_layout,
                BindGroupLayoutType::Uniform => uniform_bind_group_layout,
                BindGroupLayoutType::Texture => texture_bind_group_layout,
                BindGroupLayoutType::CubeMap => cubemap_bind_group_layout,
            }
        };

        Renderer {
            device,
            queue,
            window,
            bind_group_layouts,
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
    pub(crate) bind_groups: Vec<Handle<wgpu::BindGroup>>,
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

#[derive(Debug, Clone)]
pub enum RenderInput {
    Shader(Handle<Shader>),
    Mesh {
        vertex_buffer: Handle<wgpu::Buffer>,
        index_buffer: Handle<wgpu::Buffer>,
        material: Option<Handle<wgpu::BindGroup>>,
        num_elements: u32,
    },
    BindGroup(String, Handle<wgpu::BindGroup>),
}

pub trait Renderable {
    fn render_inputs(&self, node: &NodeDescriptor, renderer: &Renderer, resources: &mut Resources) -> Vec<RenderInput>;
}

pub trait RenderableResource {
    fn render_inputs(&self, node: &NodeDescriptor, renderer: &Renderer, resources: &Resources) -> Vec<RenderInput>;
}
