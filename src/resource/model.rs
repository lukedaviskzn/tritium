use wgpu::util::DeviceExt;

use crate::{renderer::{Vertex, Renderer, RenderInput, RenderableResource, PositionVertex, ModelVertex}, node::NodeDescriptor, resource::compute_tangents};

use super::{Material, Handle, Resources};

pub enum SphereUV {
    Equirectangular,
    /// Repeat texture on back
    Equirectangular2X,
    /// Use same tex coords for each face
    Cube,
}

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
            Mesh::new(renderer, resources, Some("plane"), vec![
                // 0 1
                // 3 2
                ModelVertex {
                    position: glam::vec3(-1.0,0.0, -1.0),
                    tex_coords: glam::vec2(0.0, 0.0),
                    normal: glam::vec3(0.0, 1.0, 0.0),
                    tangent: glam::vec3(1.0, 0.0, 0.0),
                    bitangent: glam::vec3(0.0, 0.0, -1.0),
                },
                ModelVertex {
                    position: glam::vec3( 1.0, 0.0, -1.0),
                    tex_coords: glam::vec2(1.0, 0.0),
                    normal: glam::vec3(0.0, 1.0, 0.0),
                    tangent: glam::vec3(1.0, 0.0, 0.0),
                    bitangent: glam::vec3(0.0, 0.0, -1.0),
                },
                ModelVertex {
                    position: glam::vec3( 1.0, 0.0,  1.0),
                    tex_coords: glam::vec2(1.0, 1.0),
                    normal: glam::vec3(0.0, 1.0, 0.0),
                    tangent: glam::vec3(1.0, 0.0, 0.0),
                    bitangent: glam::vec3(0.0, 0.0, -1.0),
                },
                ModelVertex {
                    position: glam::vec3(-1.0, 0.0,  1.0),
                    tex_coords: glam::vec2(0.0, 1.0),
                    normal: glam::vec3(0.0, 1.0, 0.0),
                    tangent: glam::vec3(1.0, 0.0, 0.0),
                    bitangent: glam::vec3(0.0, 0.0, -1.0),
                },
            ], indices, material)
        } else {
            Mesh::new(renderer, resources, Some("plane"), vec![
                // 0 1
                // 3 2
                PositionVertex { position: glam::vec3(-1.0, 0.0, -1.0) },
                PositionVertex { position: glam::vec3( 1.0, 0.0, -1.0) },
                PositionVertex { position: glam::vec3( 1.0, 0.0,  1.0) },
                PositionVertex { position: glam::vec3(-1.0, 0.0,  1.0) },
            ], indices, material)
        };
        
        let model = Model {
            meshes: vec![mesh],
        };

        model
    }

    pub fn new_cube(renderer: &Renderer, resources: &mut Resources, material: Option<Handle<Material>>) -> Model {
        let model = Model {
            meshes: vec![Mesh::new(renderer, resources, Some("cube"), vec![
                // 0 1
                // 3 2
                PositionVertex { position: glam::vec3(-1.0,  1.0, -1.0) },
                PositionVertex { position: glam::vec3( 1.0,  1.0, -1.0) },
                PositionVertex { position: glam::vec3( 1.0, -1.0, -1.0) },
                PositionVertex { position: glam::vec3(-1.0, -1.0, -1.0) },
                // 4 5
                // 7 6
                PositionVertex { position: glam::vec3(-1.0,  1.0,  1.0) },
                PositionVertex { position: glam::vec3( 1.0,  1.0,  1.0) },
                PositionVertex { position: glam::vec3( 1.0, -1.0,  1.0) },
                PositionVertex { position: glam::vec3(-1.0, -1.0,  1.0) },
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
            meshes: vec![Mesh::new(renderer, resources, Some("inverted_cube"), vec![
                // 0 1
                // 3 2
                PositionVertex { position: glam::vec3(-1.0,  1.0, -1.0) },
                PositionVertex { position: glam::vec3( 1.0,  1.0, -1.0) },
                PositionVertex { position: glam::vec3( 1.0, -1.0, -1.0) },
                PositionVertex { position: glam::vec3(-1.0, -1.0, -1.0) },
                // 4 5
                // 7 6
                PositionVertex { position: glam::vec3(-1.0,  1.0,  1.0) },
                PositionVertex { position: glam::vec3( 1.0,  1.0,  1.0) },
                PositionVertex { position: glam::vec3( 1.0, -1.0,  1.0) },
                PositionVertex { position: glam::vec3(-1.0, -1.0,  1.0) },
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

    pub fn new_sphere(renderer: &Renderer, resources: &mut Resources, material: Option<Handle<Material>>, subdivisions: u32, sphere_uvs: SphereUV) -> Model {
        // todo: fix seam on left side of sphere
        
        fn spherical_to_equirectangular(v: glam::Vec3) -> glam::Vec2 {
            let mut uv = glam::vec2(f32::atan2(v.z, v.x), f32::asin(v.y));
            let inv_atan = glam::vec2(0.1591, 0.3183);
            uv *= inv_atan;
            uv += 0.5;
            return uv;
        }
        
        // (dir, tangent, bitangent)
        let dirs = [
            (glam::Vec3::X, glam::Vec3::NEG_Z, glam::Vec3::Y),
            (glam::Vec3::NEG_X, glam::Vec3::Z, glam::Vec3::Y),
            (glam::Vec3::Y, glam::Vec3::X, glam::Vec3::NEG_Z),
            (glam::Vec3::NEG_Y, glam::Vec3::NEG_X, glam::Vec3::NEG_Z),
            (glam::Vec3::Z, glam::Vec3::X, glam::Vec3::Y),
            (glam::Vec3::NEG_Z, glam::Vec3::NEG_X, glam::Vec3::Y),
        ];

        let use_model_vertices = material.is_some();

        let mut model_vertices = vec![];
        let mut position_vertices = vec![];

        for (dir, tangent, bitangent) in dirs {
            for y in 0..(subdivisions + 1) {
                let cy = y as f32 * 2.0 / subdivisions as f32 - 1.0;
                for x in 0..(subdivisions + 1) {
                    let cx = x as f32 * 2.0 / subdivisions as f32 - 1.0;
                    let pos = (dir + cx * tangent + cy * bitangent).normalize();

                    if use_model_vertices {
                        model_vertices.push(ModelVertex {
                            position: pos,
                            // tex_coords: glam::vec2(x as f32 / subdivisions as f32, y as f32 / subdivisions as f32),
                            tex_coords: match sphere_uvs {
                                SphereUV::Equirectangular => spherical_to_equirectangular(pos),
                                SphereUV::Equirectangular2X => spherical_to_equirectangular(pos) * glam::vec2(2.0, 1.0),
                                SphereUV::Cube => glam::vec2(x as f32 / subdivisions as f32, y as f32 / subdivisions as f32),
                            },
                            normal: pos,
                            tangent: glam::Vec3::ZERO,
                            bitangent: glam::Vec3::ZERO,
                        });
                    } else {
                        position_vertices.push(PositionVertex {
                            position: pos,
                        });
                    }
                }
            }
        }
        
        let mut indices = vec![];

        for f in 0..6 {
            let face_offset = (f * (subdivisions + 1) * (subdivisions + 1)) as u32;
            for y in 0..subdivisions {
                let row_offset = (y * (subdivisions + 1)) as u32;
                for x in 0..subdivisions {
                    let index = (face_offset + row_offset + x as u32) as u32;
                    
                    indices.push(index);
                    indices.push(index + 1);
                    indices.push(index + (subdivisions + 1) + 1);
                    indices.push(index);
                    indices.push(index + (subdivisions + 1) + 1);
                    indices.push(index + (subdivisions + 1));
                }
            }
        }

        if use_model_vertices {
            compute_tangents(&mut model_vertices, &indices);
        }
        
        let mesh = if use_model_vertices {
            Mesh::new(renderer, resources, Some("sphere"), model_vertices, indices, material)
        } else {
            Mesh::new(renderer, resources, Some("sphere"), position_vertices, indices, material)
        };

        Model {
            meshes: vec![mesh],
        }
    }
}

pub struct Mesh {
    pub name: Option<String>,
    pub(crate) vertex_buffer: Handle<wgpu::Buffer>,
    pub(crate) index_buffer: Handle<wgpu::Buffer>,
    pub num_elements: u32,
    pub material: Option<Handle<Material>>,
}

impl Mesh {
    pub fn new<T: Vertex>(renderer: &Renderer, resources: &mut Resources, name: Option<&str>, vertices: Vec<T>, indices: Vec<u32>, material: Option<Handle<Material>>) -> Mesh {
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
            name: name.map(|n| n.into()),
            vertex_buffer,
            index_buffer,
            num_elements,
            material,
        }
    }
}

impl RenderableResource for Model {
    fn render_inputs(&self, _node: &NodeDescriptor, _renderer: &Renderer, _resources: &Resources) -> Vec<RenderInput> {
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
