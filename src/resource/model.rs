use wgpu::util::DeviceExt;

use crate::{renderer::{Vertex, Renderer, RenderInput, RenderableResource, PositionVertex, ModelVertex}, node::NodeDescriptor};

use super::{Material, Handle, Resources};

pub struct Model {
    pub meshes: Vec<Mesh>,
    // pub materials: Vec<Material>,
}

impl Model {
    pub fn new_plane(renderer: &Renderer, resources: &mut Resources, material: Option<Handle<Material>>) -> Model {
        let indices = vec![
            2, 1, 0,
            2, 0, 3,
        ];

        let mesh = if material.is_some() {
            Mesh::new(renderer, resources, "plane", vec![
                // 0 1
                // 3 2
                ModelVertex {
                    position:[-1.0,0.0, -1.0],
                    tex_coords: [0.0, 0.0],
                    normal: [0.0, 1.0, 0.0],
                    tangent: [1.0, 0.0, 0.0],
                    bitangent: [0.0, 0.0, -1.0],
                },
                ModelVertex {
                    position: [ 1.0, 0.0, -1.0],
                    tex_coords: [1.0, 0.0],
                    normal: [0.0, 1.0, 0.0],
                    tangent: [1.0, 0.0, 0.0],
                    bitangent: [0.0, 0.0, -1.0],
                },
                ModelVertex {
                    position: [ 1.0, 0.0,  1.0],
                    tex_coords: [1.0, 1.0],
                    normal: [0.0, 1.0, 0.0],
                    tangent: [1.0, 0.0, 0.0],
                    bitangent: [0.0, 0.0, -1.0],
                },
                ModelVertex {
                    position: [-1.0, 0.0,  1.0],
                    tex_coords: [0.0, 1.0],
                    normal: [0.0, 1.0, 0.0],
                    tangent: [1.0, 0.0, 0.0],
                    bitangent: [0.0, 0.0, -1.0],
                },
            ], indices, material)
        } else {
            Mesh::new(renderer, resources, "plane", vec![
                // 0 1
                // 3 2
                PositionVertex { position: [-1.0, 0.0, -1.0] },
                PositionVertex { position: [ 1.0, 0.0, -1.0] },
                PositionVertex { position: [ 1.0, 0.0,  1.0] },
                PositionVertex { position: [-1.0, 0.0,  1.0] },
            ], indices, material)
        };
        
        let model = Model {
            meshes: vec![mesh],
        };

        model
    }

    pub fn new_cube(renderer: &Renderer, resources: &mut Resources, material: Option<Handle<Material>>) -> Model {
        let model = Model {
            meshes: vec![Mesh::new(renderer, resources, "cube", vec![
                // 0 1
                // 3 2
                PositionVertex { position: [-1.0,  1.0, -1.0] },
                PositionVertex { position: [ 1.0,  1.0, -1.0] },
                PositionVertex { position: [ 1.0, -1.0, -1.0] },
                PositionVertex { position: [-1.0, -1.0, -1.0] },
                // 4 5
                // 7 6
                PositionVertex { position: [-1.0,  1.0,  1.0] },
                PositionVertex { position: [ 1.0,  1.0,  1.0] },
                PositionVertex { position: [ 1.0, -1.0,  1.0] },
                PositionVertex { position: [-1.0, -1.0,  1.0] },
            ], vec![
                // back face
                1, 2, 0,
                2, 3, 0,
                // front face
                6, 5, 4,
                7, 6, 4,
                // left face
                0, 3, 4,
                3, 7, 4,
                // right face
                5, 6, 1,
                6, 2, 1,
                // top face
                0, 4, 5,
                1, 0, 5,
                // bottom face
                6, 7, 3,
                2, 6, 3,
            ], material)],
        };

        model
    }

    pub fn new_inverted_cube(renderer: &Renderer, resources: &mut Resources, material: Option<Handle<Material>>) -> Model {
        let model = Model {
            meshes: vec![Mesh::new(renderer, resources, "inverted_cube", vec![
                // 0 1
                // 3 2
                PositionVertex { position: [-1.0,  1.0, -1.0] },
                PositionVertex { position: [ 1.0,  1.0, -1.0] },
                PositionVertex { position: [ 1.0, -1.0, -1.0] },
                PositionVertex { position: [-1.0, -1.0, -1.0] },
                // 4 5
                // 7 6
                PositionVertex { position: [-1.0,  1.0,  1.0] },
                PositionVertex { position: [ 1.0,  1.0,  1.0] },
                PositionVertex { position: [ 1.0, -1.0,  1.0] },
                PositionVertex { position: [-1.0, -1.0,  1.0] },
            ], vec![
                // back face
                0, 2, 1,
                0, 3, 2,
                // front face
                4, 5, 6,
                4, 6, 7,
                // left face
                4, 3, 0,
                4, 7, 3,
                // right face
                1, 6, 5,
                1, 2, 6,
                // top face
                5, 4, 0,
                5, 0, 1,
                // bottom face
                3, 7, 6,
                3, 6, 2,
            ], material)],
        };

        model
    }
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
                material: mesh.material.clone(),
                num_elements: mesh.num_elements,
            });
        }

        inputs
    }
}
