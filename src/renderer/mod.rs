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
            power_preference: wgpu::PowerPreference::HighPerformance,
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

pub trait RenderTarget {
    fn view(&self) -> Handle<wgpu::TextureView>;
}

pub(crate) struct QueuedRenderObject {
    pub shader: Handle<Shader>,
    // pub target: Handle<Box<dyn RenderTarget>>,
    pub vertex_buffer: Handle<wgpu::Buffer>,
    pub index_buffer: Handle<wgpu::Buffer>,
    pub bind_group: wgpu::BindGroup,
    pub num_indices: u32,
}

// #[derive(Debug, Clone)]
pub enum RenderInput {
    Shader(Handle<Shader>),
    Mesh {
        vertex_buffer: Handle<wgpu::Buffer>,
        index_buffer: Handle<wgpu::Buffer>,
        material: Option<Handle<Material>>,
        num_elements: u32,
    },
    BindingResources(String, BindingHolder),
    SceneInput(String, SceneInputItem),
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
