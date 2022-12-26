use wgpu::util::DeviceExt;

use crate::{renderer::{Vertex, Renderer, RenderInput, RenderableResource}, world::NodeDescriptor};

use super::{Material, Handle, Resources};

pub struct Model {
    pub meshes: Vec<Mesh>,
    // pub materials: Vec<Material>,
}

pub struct Mesh {
    pub name: String,
    pub(crate) vertex_buffer: Handle<wgpu::Buffer>,
    pub(crate) index_buffer: Handle<wgpu::Buffer>,
    pub num_elements: u32,
    pub material: Option<Handle<Material>>,
}

impl Mesh {
    pub fn new<T: Vertex>(renderer: &Renderer, resources: &mut Resources, name: &str, vertices: Vec<T>, indices: Vec<u32>, material: Option<Handle<Material>>) -> Mesh {
        let num_elements = indices.len() as u32;

        let vertex_buffer = resources.store(renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Mesh Vertex Buffer")),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }));
        
        let index_buffer = resources.store(renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Mesh Index Buffer")),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        }));

        Mesh {
            name: name.into(),
            vertex_buffer,
            index_buffer,
            num_elements,
            material,
        }
    }
}

impl RenderableResource for Model {
    fn render_inputs(&self, _node: &NodeDescriptor, _renderer: &Renderer, resources: &Resources) -> Vec<RenderInput> {
        let mut inputs = vec![];

        for mesh in &self.meshes {
            // inputs.push(RenderInput::new(&mesh.name, RenderInputStorage::Mesh {
            inputs.push(RenderInput::Mesh {
                vertex_buffer: mesh.vertex_buffer.clone(),
                index_buffer: mesh.index_buffer.clone(),
                material: if let Some(material) = mesh.material.clone() {
                    resources.get(&material).map(|material| material.bind_group())
                } else {
                    None
                },
                num_elements: mesh.num_elements,
            });
        }

        inputs
    }
}
