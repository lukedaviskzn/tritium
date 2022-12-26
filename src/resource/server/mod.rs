use std::{path::Path, io::{Cursor, BufReader}, collections::HashMap, any::{TypeId, Any}};

use wgpu::util::DeviceExt;

use crate::{resource::{Material, Mesh}, engine::Rgba, util::AsAny, renderer::{ModelVertex, Renderer}};

use super::{Texture, Model};

mod handle;
pub use handle::*;

pub trait ManagesResources: AsAny {
    fn drop_invalid(&mut self);
}

impl<R: 'static> AsAny for ResourceManager<R> {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl<R: 'static> ManagesResources for ResourceManager<R> {
    // returns total number of assets
    fn drop_invalid(&mut self) {
        // retain only if marker has strong references
        self.resources.retain(|handle, _| if let Handle::Weak(_, marker) = handle {
            // if marker.strong_count() <= 0 {
            //     log::trace!("Dropping resource {:?}", handle);
            // }
            marker.strong_count() > 0
        } else {
            false
        });
    }
}

pub struct ResourceManager<T> {
    // weak handle, so as to drop resources automatically (on first_update)
    resources: HashMap<Handle<T>, T>,
}

impl<T> ResourceManager<T> {
    pub fn new() -> ResourceManager<T> {
        ResourceManager {
            resources: hashmap!{},
        }
    }

    pub fn store(&mut self, resource: T) -> Handle<T> {
        let handle = Handle::new_strong();

        self.resources.insert(handle.downgrade(), resource);

        handle
    }

    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        match self.resources.get_key_value(&handle) {
            // Resource present
            Some((key, value)) => {
                if key.valid() {
                    Some(value)
                } else {
                    // should be dropped
                    None
                }
            },
            // Resource not present
            None => None,
        }
    }

    pub fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        match self.resources.get_mut(&handle) {
            // Resource present
            Some(resource) => {
                if handle.valid() {
                    Some(resource)
                } else {
                    // should be dropped
                    None
                }
            },
            // Resource not present
            None => None,
        }
    }

    pub fn set(&mut self, handle: &Handle<T>, resource: T) -> Handle<T> {
        self.resources.insert(handle.downgrade(), resource);

        handle.clone()
    }
}

// wrapper that can only be accessed in-engine, used to store resources the game dev shouldn't have access to
// pub(crate) struct EngineStorage<T>(pub(crate) T);

// impl<T> EngineStorage<T> {
//     pub fn new(value: T) -> EngineStorage<T> {
//         EngineStorage(value)
//     }
// }

// impl<T> Deref for EngineStorage<T> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

pub struct Resources {
    managers: HashMap<TypeId, Box<dyn ManagesResources>>,
    global_handles: HashMap<String, Box<dyn Any>>,
    // engine_global_handles: HashMap<String, Box<dyn Any>>,
}

impl Resources {
    pub fn new() -> Resources {
        Resources {
            managers: hashmap!{},
            global_handles: hashmap!{},
            // engine_global_handles: hashmap!{},
        }
    }

    fn register_manager<T: 'static>(&mut self) {
        let typeid = TypeId::of::<T>();
        
        if !self.managers.contains_key(&typeid) {
            log::trace!("Registering resource manager {:?}", std::any::type_name::<T>());
            self.managers.insert(typeid, Box::new(ResourceManager::<T>::new()));
        }
    }

    fn get_manager<T: 'static>(&self) -> Option<&ResourceManager<T>> {
        let manager = self.managers.get(&TypeId::of::<T>());
        
        if manager.is_none() {
            log::error!("Unmanaged resource type {:?}", std::any::type_name::<T>());
        }
        
        let manager = manager?.as_any();
        let manager: &ResourceManager<T> = manager.downcast_ref()?;

        Some(manager)
    }

    fn get_manager_mut<T: 'static>(&mut self) -> Option<&mut ResourceManager<T>> {
        let manager = self.managers.get_mut(&TypeId::of::<T>());
        
        if manager.is_none() {
            log::error!("Unmanaged resource type {:?}", std::any::type_name::<T>());
        }
        
        let manager = manager?.as_any_mut();
        let manager = manager.downcast_mut()?;

        Some(manager)
    }

    pub(crate) fn drop_invalid(&mut self) {
        for manager in self.managers.values_mut() {
            manager.drop_invalid();
        }
    }

    pub fn store<T: 'static>(&mut self, resource: T) -> Handle<T> {
        self.register_manager::<T>();
        let manager = self.get_manager_mut::<T>().unwrap();

        let handle = manager.store(resource);

        // log::trace!("Stored resource {:?}", handle);

        handle
    }

    pub fn get<T: 'static>(&self, handle: &Handle<T>) -> Option<&T> {
        let manager = self.get_manager::<T>()?;
        
        let resource = manager.get(handle);
        
        if resource.is_none() && handle.is_strong() {
            log::error!("Failed to fetch resource with strong handle {handle:?}");
        }
        
        resource
    }

    pub fn get_mut<T: 'static>(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        let manager = self.get_manager_mut::<T>()?;
        
        let resource = manager.get_mut(handle);
        
        if resource.is_none() && handle.is_strong() {
            log::error!("Failed to fetch resource with strong handle {handle:?}");
        }
        
        resource
    }

    pub fn set<T: 'static>(&mut self, handle: &Handle<T>, resource: T) -> Handle<T> {
        self.register_manager::<T>();
        let manager = self.get_manager_mut::<T>().unwrap();
        
        let handle = manager.set(handle, resource);

        handle
    }

    pub fn set_global<T: 'static>(&mut self, key: &str, resource: T) {
        let resource: Box<dyn Any> = Box::new(resource);
        self.global_handles.insert(key.to_owned(), resource);
    }

    pub fn get_global<T: 'static>(&self, key: &str) -> Option<&T> {
        self.global_handles.get(key)?.downcast_ref::<T>()
    }

    pub fn get_global_mut<T: 'static>(&mut self, key: &str) -> Option<&T> {
        self.global_handles.get_mut(key)?.downcast_ref::<T>()
    }

    // pub(crate) fn set_engine_global<T: 'static>(&mut self, key: &str, resource: T) {
    //     let resource: Box<dyn Any> = Box::new(resource);
    //     self.engine_global_handles.insert(key.to_owned(), resource);
    // }

    // pub(crate) fn get_engine_global<T: 'static>(&self, key: &str) -> Option<&T> {
    //     self.engine_global_handles.get(key)?.downcast_ref::<T>()
    // }

    // pub(crate) fn get_engine_global_mut<T: 'static>(&mut self, key: &str) -> Option<&mut T> {
    //     self.engine_global_handles.get_mut(key)?.downcast_mut::<T>()
    // }
}

pub fn load_texture<P: AsRef<Path>>(
    path: P,
    renderer: &Renderer,
    resources: &mut Resources,
    srgb: bool,
) -> Result<Texture, image::ImageError> {
    let path = path.as_ref();

    log::debug!("Loading texture {path:?}");

    let path_str = &path.to_string_lossy().to_string();

    let image= image::open(path)?;

    Ok(Texture::from_image(renderer, resources, &image, Some(path_str), srgb))
}

pub fn load_obj<P: AsRef<Path>>(
    renderer: &crate::renderer::Renderer,
    resources: &mut Resources,
    path: P,
) -> Result<Model, ModelLoadError> {
    let path = path.as_ref();
    let path_str = &path.to_string_lossy().to_string();
    let parent = path.parent().unwrap_or(Path::new("res")).to_owned();

    log::debug!("Loading obj model {path:?}");
    
    let obj_text = std::fs::read_to_string(path.clone()).map_err(|err| ModelLoadError::IoError(err))?;

    log::trace!("Read obj file {path:?}");
    
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf(
        &mut obj_reader,
        &tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ..Default::default()
        },
        |p| {
            let material_path = parent.join(p).to_owned();
            
            log::trace!("Read material file {material_path:?}");
            
            let material_text = std::fs::read_to_string(material_path).unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(material_text)))
        },
    ).map_err(|err| ModelLoadError::TobjError(err))?;

    let mut materials = vec![];

    for material in obj_materials.map_err(|err| ModelLoadError::TobjError(err))? {
        let diffuse_texture = if material.diffuse_texture.len() > 0 {
            load_texture(parent.join(material.diffuse_texture), renderer, resources, true).map_err(|err| ModelLoadError::ImageError(err))?
        } else {
            Texture::from_pixel(renderer, resources, &[255, 255, 255, 255], Some(&format!("{} diffuse pixel texture", material.name)), true)
        };
        let diffuse_texture = &resources.store(diffuse_texture);

        let diffuse_colour = Rgba::new(material.diffuse[0], material.diffuse[1], material.diffuse[2], 1.0);

        let normal_texture = if material.normal_texture.len() > 0 {
            load_texture(parent.join(material.normal_texture), renderer, resources, false).map_err(|err| ModelLoadError::ImageError(err))?
        } else {
            Texture::from_pixel(renderer, resources, &[128, 128, 255, 255], Some(&format!("{} normal pixel texture", material.name)), false)
        };
        let normal_texture = &resources.store(normal_texture);

        let normal_factor = 1.0;

        let material = Material::new(
            &renderer,
            resources,
            &material.name,
            diffuse_texture,
            diffuse_colour,
            normal_texture,
            normal_factor,
        );

        materials.push(resources.store(material));
    }

    let meshes: Vec<Mesh> = models.into_iter()
        .map(|model| {
            let mut vertices: Vec<ModelVertex> = (0..model.mesh.positions.len() / 3)
                .map(|i| ModelVertex {
                    position: [
                        model.mesh.positions[i*3],
                        model.mesh.positions[i*3+1],
                        model.mesh.positions[i*3+2],
                    ],
                    tex_coords: [
                        model.mesh.texcoords[i*2],
                        model.mesh.texcoords[i*2+1],
                    ],
                    normal: [
                        model.mesh.normals[i*3],
                        model.mesh.normals[i*3+1],
                        model.mesh.normals[i*3+2],
                    ],
                    tangent: [0.0; 3],
                    bitangent: [0.0; 3],
                }).collect();
            
            let indices = &model.mesh.indices;
            let mut triangles_included = vec![0; vertices.len()];

            for c in indices.chunks(3) {
                let v0 = vertices[c[0] as usize];
                let v1 = vertices[c[1] as usize];
                let v2 = vertices[c[2] as usize];

                let pos0: glam::Vec3 = v0.position.into();
                let pos1: glam::Vec3 = v1.position.into();
                let pos2: glam::Vec3 = v2.position.into();

                let uv0: glam::Vec2 = v0.tex_coords.into();
                let uv1: glam::Vec2 = v1.tex_coords.into();
                let uv2: glam::Vec2 = v2.tex_coords.into();

                let delta_pos1 = pos1 - pos0;
                let delta_pos2 = pos2 - pos0;

                let delta_uv1 = uv1 - uv0;
                let delta_uv2 = uv2 - uv0;

                // Solve:
                // delta_pos1 = delta_uv1.x * T + delta_uv1.y * B;
                // delta_pos2 = delta_uv2.x * T + delta_uv2.y * B;

                // Solution:
                let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

                vertices[c[0] as usize].tangent = 
                    (tangent + glam::Vec3::from(vertices[c[0] as usize].tangent)).into();
                vertices[c[1] as usize].tangent = 
                    (tangent + glam::Vec3::from(vertices[c[1] as usize].tangent)).into();
                vertices[c[2] as usize].tangent = 
                    (tangent + glam::Vec3::from(vertices[c[2] as usize].tangent)).into();
                vertices[c[0] as usize].tangent = 
                    (bitangent + glam::Vec3::from(vertices[c[0] as usize].bitangent)).into();
                vertices[c[1] as usize].tangent = 
                    (bitangent + glam::Vec3::from(vertices[c[1] as usize].bitangent)).into();
                vertices[c[2] as usize].tangent = 
                    (bitangent + glam::Vec3::from(vertices[c[2] as usize].bitangent)).into();
                
                // Used to average tangents/bitangents for vertices that touch multiple triangles.
                triangles_included[c[0] as usize] += 1;
                triangles_included[c[1] as usize] += 1;
                triangles_included[c[2] as usize] += 1;
            }

            for (i, n) in triangles_included.into_iter().enumerate() {
                let denom = 1.0 / n as f32;
                let mut v = &mut vertices[i];
                v.tangent = (glam::Vec3::from(v.tangent) * denom).into();
                v.bitangent = (glam::Vec3::from(v.bitangent) * denom).into();
            }
            
            let vertex_buffer = resources.store(renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} Vertex Buffer", path_str)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }));
            
            let index_buffer = resources.store(renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} Index Buffer", path_str)),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            }));

            let material = model.mesh.material_id.map(|material_id| materials[material_id].clone());

            let mesh = Mesh {
                name: model.name,
                vertex_buffer,
                index_buffer,
                material,
                num_elements: model.mesh.indices.len() as u32,
                // material: model.mesh.material_id,
            };

            mesh
        }).collect();
    
    Ok(Model {
        meshes,
        // materials,
    })
}

#[derive(Debug)]
pub enum ModelLoadError {
    IoError(std::io::Error),
    ImageError(image::ImageError),
    TobjError(tobj::LoadError),
    // GltfError(gltf::Error),
}

impl std::fmt::Display for ModelLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelLoadError::IoError(err) => err.fmt(f),
            ModelLoadError::ImageError(err) => err.fmt(f),
            ModelLoadError::TobjError(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for ModelLoadError {}
