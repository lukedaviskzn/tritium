use std::{path::Path, io::{Cursor, BufReader}, collections::HashMap, any::{TypeId, Any}, ffi::OsStr};

use glam::Vec4Swizzles;
use gltf::Gltf;
use wgpu::util::DeviceExt;

use crate::{resource::{Material, Mesh, AlphaMode}, engine::Rgba, util::AsAny, renderer::{ModelVertex, Renderer}, node::Node, components::Transform, camera::Camera};

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
    // srgb: bool,
    flip_y: bool,
) -> Result<Texture, image::ImageError> {
    let path = path.as_ref();

    log::debug!("Loading texture {path:?}");

    let path_str = &path.to_string_lossy().to_string();

    let image= image::open(path)?;

    Ok(Texture::from_image(renderer, resources, &image, Some(path_str), flip_y))
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
        let name = if material.name.len() > 0 {
            Some(material.name.as_str())
        } else {
            None
        };

        let diffuse_texture = if material.diffuse_texture.len() > 0 {
            load_texture(parent.join(material.diffuse_texture), renderer, resources, true).map_err(|err| ModelLoadError::ImageError(err))?
        } else {
            Texture::from_pixel(renderer, resources, &[255, 255, 255, 255], Some(&format!("{} diffuse pixel texture", material.name)), true)
        };
        let diffuse_texture = resources.store(diffuse_texture);

        let diffuse_colour = Rgba::new(material.diffuse[0], material.diffuse[1], material.diffuse[2], 1.0);

        let normal_texture = if material.normal_texture.len() > 0 {
            load_texture(parent.join(material.normal_texture), renderer, resources, true).map_err(|err| ModelLoadError::ImageError(err))?
        } else if let Some(disp_map) = material.unknown_param.get("map_Disp") {
            if disp_map.len() > 0 {
                load_texture(parent.join(&material.unknown_param["map_Disp"]), renderer, resources, true).map_err(|err| ModelLoadError::ImageError(err))?
            } else {
                Texture::from_pixel(renderer, resources, &[128, 128, 255, 255], Some(&format!("{} normal pixel texture", material.name)), false)
            }
        } else {
            Texture::from_pixel(renderer, resources, &[128, 128, 255, 255], Some(&format!("{} normal pixel texture", material.name)), false)
        };
        let normal_texture = resources.store(normal_texture);

        let normal_scale = 1.0;

        // let material = Material::new(
        //     &renderer,
        //     resources,
        //     name,
        //     false,
        //     AlphaMode::Mask { cutoff: 0.1 },
        //     Some(diffuse_texture),
        //     diffuse_colour,
        //     None,
        //     0.0,
        //     0.5,
        //     Some(normal_texture),
        //     normal_scale,
        //     None,
        //     1.0,
        //     None,
        //     Rgba::BLACK,
        // );
        let material = Material::builder()
            .albedo_texture(diffuse_texture)
            .albedo(diffuse_colour)
            .metallic_factor(0.0)
            .roughness_factor(0.5)
            .normal_texture(normal_texture)
            .build(renderer, resources);

        materials.push(resources.store(material));
    }

    let meshes: Vec<Mesh> = models.into_iter()
        .map(|model| {
            let mut vertices: Vec<ModelVertex> = (0..model.mesh.positions.len() / 3)
                .map(|i| ModelVertex {
                    position: glam::vec3(
                        model.mesh.positions[i*3],
                        model.mesh.positions[i*3+1],
                        model.mesh.positions[i*3+2],
                    ),
                    tex_coords: glam::vec2(
                        model.mesh.texcoords[i*2],
                        model.mesh.texcoords[i*2+1],
                    ),
                    normal: glam::vec3(
                        model.mesh.normals[i*3],
                        model.mesh.normals[i*3+1],
                        model.mesh.normals[i*3+2],
                    ),
                    tangent: glam::Vec3::ZERO,
                    bitangent: glam::Vec3::ZERO,
                }).collect();
            
            let indices = &model.mesh.indices;
                
            compute_tangents(&mut vertices, indices);
            
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

            let name = if model.name.len() > 0 {
                Some(model.name)
            } else {
                None
            };

            let mesh = Mesh {
                name,
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

// If scene is None, use default scene
pub fn load_gltf<P: AsRef<Path>>(
    renderer: &crate::renderer::Renderer,
    resources: &mut Resources,
    path: P,
    scene: Option<usize>,
) -> Result<Node, SceneLoadError> {
    // let gltf = Gltf::open(path).map_err(|err| SceneLoadError::GltfError(err))?;

    struct ImportData {
        buffers: Vec<gltf::buffer::Data>,
        textures: Vec<Handle<Texture>>,
    }

    log::debug!("Loading gltf {:?}", path.as_ref());

    // todo: don't load all scenes, if only one is needed
    let (document, import_data) = {
        let (document, buffers, images) = gltf::import(&path).map_err(|err| SceneLoadError::GltfError(err))?;

        let textures = images.into_iter().map(|data| {
            let gltf::image::Data {
                pixels,
                format,
                width,
                height,
            } = data;

            // convert `gltf` Image to `image` DynamicImage
            let image = match format {
                gltf::image::Format::R8 => image::DynamicImage::ImageLuma8(image::GrayImage::from_raw(width, height, pixels).unwrap()),
                gltf::image::Format::R8G8 => {
                    let pixels = pixels.chunks(2).flat_map(|chunk| [chunk[0], chunk[1], 0]).collect();
                    image::DynamicImage::ImageRgb8(image::RgbImage::from_raw(width, height, pixels).unwrap())
                },
                gltf::image::Format::R8G8B8 => image::DynamicImage::ImageRgb8(image::RgbImage::from_raw(width, height, pixels).unwrap()),
                gltf::image::Format::R8G8B8A8 => image::DynamicImage::ImageRgba8(image::RgbaImage::from_raw(width, height, pixels).unwrap()),
                gltf::image::Format::B8G8R8 => {
                    let pixels = pixels.chunks(3).flat_map(|chunk| {
                        [chunk[2], chunk[1], chunk[0]]
                    }).collect();
                    image::DynamicImage::ImageRgb8(image::RgbImage::from_raw(width, height, pixels).unwrap())
                },
                gltf::image::Format::B8G8R8A8 => {
                    let pixels = pixels.chunks(4).flat_map(|chunk| {
                        [chunk[2], chunk[1], chunk[0], chunk[3]]
                    }).collect();
                    image::DynamicImage::ImageRgb8(image::RgbImage::from_raw(width, height, pixels).unwrap())
                },
                gltf::image::Format::R16 => {
                    let pixels = pixels.chunks(2).map(|chunk| bytemuck::cast_slice(chunk)[0]).collect();
                    image::DynamicImage::ImageLuma16(image::ImageBuffer::<image::Luma<u16>, Vec<u16>>::from_raw(width, height, pixels).unwrap())
                },
                gltf::image::Format::R16G16 => {
                    let pixels = pixels.chunks(4).flat_map(|chunk| {
                        let slice = bytemuck::cast_slice(chunk);
                        [slice[0], slice[1], 0]
                    }).collect();
                    image::DynamicImage::ImageRgb16(image::ImageBuffer::<image::Rgb<u16>, Vec<u16>>::from_raw(width, height, pixels).unwrap())
                },
                gltf::image::Format::R16G16B16 => {
                    let pixels = pixels.chunks(2).map(|chunk| bytemuck::cast_slice(chunk)[0]).collect();
                    image::DynamicImage::ImageRgb16(image::ImageBuffer::<image::Rgb<u16>, Vec<u16>>::from_raw(width, height, pixels).unwrap())
                },
                gltf::image::Format::R16G16B16A16 => {
                    let pixels = pixels.chunks(2).map(|chunk| bytemuck::cast_slice(chunk)[0]).collect();
                    image::DynamicImage::ImageRgba16(image::ImageBuffer::<image::Rgba<u16>, Vec<u16>>::from_raw(width, height, pixels).unwrap())
                },
            };

            let texture = Texture::from_image(renderer, resources, &image, None, false);

            resources.store(texture)
        }).collect();
        
        (document, ImportData {
            buffers,
            textures,
        })
    };

    log::trace!("Gltf buffers loaded {:?}", path.as_ref());

    let scene = if let Some(scene) = scene {
        document.scenes().nth(scene).ok_or(SceneLoadError::SceneNotFound)?
    } else {
        document.default_scene().ok_or(SceneLoadError::SceneNotFound)?
    };

    fn visit(renderer: &Renderer, resources: &mut Resources, node: gltf::Node, import_data: &ImportData) -> Node {
        let mut builder = Node::builder(node.name().unwrap_or("#"));
        
        let (scale, rotation, translation) = match node.transform() {
            gltf::scene::Transform::Matrix { matrix } => glam::Mat4::from_cols_array_2d(&matrix).to_scale_rotation_translation(),
            gltf::scene::Transform::Decomposed { translation, rotation, scale } => (scale.into(), glam::Quat::from_array(rotation), translation.into()),
        };

        let transform = Transform::new(translation, rotation, scale);

        builder = builder.add_component(transform);

        if let Some(camera) = node.camera() {
            log::trace!("Loading camera, name: {:?}", camera.name());
            
            let camera = match camera.projection() {
                gltf::camera::Projection::Perspective(proj) => Camera::Perspective { fovy: proj.yfov(), znear: proj.znear(), zfar: proj.zfar() },
                gltf::camera::Projection::Orthographic(proj) => Camera::Orthographic { ymag: proj.ymag(), znear: proj.znear(), zfar: proj.zfar() },
            };

            builder = builder.add_component(camera);
        }
        if let Some(mesh) = node.mesh() {
            log::trace!("Loading mesh, name: {:?}", mesh.name());
            
            let mut meshes = vec![];
            
            for primitive in mesh.primitives() {
                // todo: handle topology
                // let topology = match primitive.mode() {
                //     gltf::mesh::Mode::Points => wgpu::PrimitiveTopology::PointList,
                //     gltf::mesh::Mode::Lines => wgpu::PrimitiveTopology::LineList,
                //     gltf::mesh::Mode::LineLoop => unimplemented!("GLTF 'LineLoop' rendering mode not supported."),
                //     gltf::mesh::Mode::LineStrip => wgpu::PrimitiveTopology::LineStrip,
                //     gltf::mesh::Mode::Triangles => wgpu::PrimitiveTopology::TriangleList,
                //     gltf::mesh::Mode::TriangleStrip => wgpu::PrimitiveTopology::TriangleStrip,
                //     gltf::mesh::Mode::TriangleFan => unimplemented!("GLTF 'TriangleFan' rendering mode not supported."),
                // };

                // log::trace!("Loading mesh");
                let mut positions = vec![];
                let mut normals = vec![];
                // let mut tangents = vec![];
                let mut tex_coords = vec![];

                for (semantic, accessor) in primitive.attributes() {
                    // log::trace!("Reading semantic {:?}", semantic);
                    match &semantic {
                        // todo: sparse accessors
                        gltf::Semantic::Positions | gltf::Semantic::Normals | gltf::Semantic::Tangents | gltf::Semantic::TexCoords(0) => {
                            let view = accessor.view().expect("Sparse accessors not supported.");
                            let buffer = &import_data.buffers[view.buffer().index()].0;

                            let dimensions = accessor.dimensions();
                            let data_type = accessor.data_type();
                            
                            let start_index = view.offset() + accessor.offset();
                            let item_size = data_type.size() * dimensions.multiplicity();
                            let stride = view.stride().unwrap_or(item_size);

                            // log::trace!("{semantic:?} {dimensions:?} {data_type:?} {item_size:?} {stride:?}");

                            for i in 0..accessor.count() {
                                let index = start_index + i * stride;

                                let data = buffer[index..(index + item_size)].chunks(data_type.size()).map(|chunk| {
                                    match data_type {
                                        gltf::accessor::DataType::U8 => chunk[0] as f32 / 255.0,
                                        gltf::accessor::DataType::U16 => bytemuck::cast_slice::<_, i8>(chunk)[0] as f32 / std::u16::MAX as f32,
                                        gltf::accessor::DataType::F32 => bytemuck::cast_slice::<_, f32>(chunk)[0],
                                        _ => todo!(),
                                    }
                                }).collect::<Vec<_>>();

                                match semantic {
                                    gltf::Semantic::Positions => {
                                        // position should be vec3
                                        positions.push(glam::vec3(data[0], data[1], data[2]));
                                    },
                                    gltf::Semantic::Normals => {
                                        // normal should be vec3
                                        normals.push(glam::vec3(data[0], data[1], data[2]));
                                    },
                                    // Tangents will be calculated from UV's
                                    // gltf::Semantic::Tangents => {
                                    //     // tangent are vec4, w is handedness
                                    //     tangents.push(glam::vec4(data[0], data[1], data[2], data[3]));
                                    // },
                                    gltf::Semantic::TexCoords(0) => {
                                        // tex coord should be vec2
                                        tex_coords.push(glam::vec2(data[0], data[1]));
                                    },
                                    _ => {},
                                }
                            }
                        },
                        _ => {},
                    }
                }

                let indices = if let Some(accessor) = primitive.indices() {
                    let view = accessor.view().expect("Sparse accessors not supported.");
                    let buffer = &import_data.buffers[view.buffer().index()].0;

                    let dimensions = accessor.dimensions();
                    let data_type = accessor.data_type();
                    
                    let start_index = view.offset() + accessor.offset();
                    let item_size = data_type.size() * dimensions.multiplicity();
                    let stride = view.stride().unwrap_or(item_size);

                    let mut indices = vec![];
                    
                    for i in 0..accessor.count() {
                        let index = start_index + i * stride;

                        let vertex_index = buffer[index..(index + item_size)].chunks(data_type.size()).map(|chunk| {
                            match data_type {
                                gltf::accessor::DataType::U8 => chunk[0] as u32,
                                gltf::accessor::DataType::U16 => bytemuck::cast_slice::<_, u16>(chunk)[0] as u32,
                                gltf::accessor::DataType::U32 => chunk[0] as u32,
                                _ => todo!(),
                                
                            }
                        }).next().unwrap();
                        
                        indices.push(vertex_index);
                    }

                    indices
                } else {
                    (0..positions.len()).map(|i| i as u32).collect()
                };

                let mut vertices = vec![];
                // let has_tangents = tangents.len() > 0;

                for i in 0..positions.len() {
                    vertices.push(ModelVertex {
                        position: positions[i],
                        tex_coords: tex_coords[i],
                        normal: normals[i],
                        // tangent: if has_tangents { tangents[i].xyz() } else { glam::Vec3::ZERO },
                        tangent: glam::Vec3::ZERO,
                        bitangent: glam::Vec3::ZERO,
                    });
                }
                
                compute_tangents(&mut vertices, &indices);

                // primitive.bounding_box(), todo: use this

                fn gltf_wrap_to_wgpu(wrap: gltf::texture::WrappingMode) -> wgpu::AddressMode {
                    match wrap {
                        gltf::texture::WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
                        gltf::texture::WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
                        gltf::texture::WrappingMode::Repeat => wgpu::AddressMode::Repeat,
                    }
                }

                fn gltf_mag_filter_to_wgpu(filter: gltf::texture::MagFilter) -> wgpu::FilterMode {
                    match filter {
                        gltf::texture::MagFilter::Nearest => wgpu::FilterMode::Nearest,
                        gltf::texture::MagFilter::Linear => wgpu::FilterMode::Linear,
                    }
                }

                fn gltf_min_filter_to_wgpu(filter: gltf::texture::MinFilter) -> (wgpu::FilterMode, wgpu::FilterMode) {
                    match filter {
                        gltf::texture::MinFilter::Nearest | gltf::texture::MinFilter::NearestMipmapNearest => (wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest),
                        gltf::texture::MinFilter::Linear | gltf::texture::MinFilter::LinearMipmapNearest => (wgpu::FilterMode::Linear, wgpu::FilterMode::Nearest),
                        gltf::texture::MinFilter::NearestMipmapLinear => (wgpu::FilterMode::Nearest, wgpu::FilterMode::Linear),
                        gltf::texture::MinFilter::LinearMipmapLinear => (wgpu::FilterMode::Linear, wgpu::FilterMode::Linear),
                    }
                }

                fn gltf_sampler_to_wgpu(sampler: gltf::texture::Sampler) -> wgpu::SamplerDescriptor {
                    let mag_filter = gltf_mag_filter_to_wgpu(sampler.mag_filter().unwrap_or(gltf::texture::MagFilter::Linear));
                    let (min_filter, mipmap_filter) = gltf_min_filter_to_wgpu(sampler.min_filter().unwrap_or(gltf::texture::MinFilter::Nearest));

                    wgpu::SamplerDescriptor {
                        address_mode_u: gltf_wrap_to_wgpu(sampler.wrap_s()),
                        address_mode_v: gltf_wrap_to_wgpu(sampler.wrap_t()),
                        address_mode_w: wgpu::AddressMode::ClampToEdge,
                        mag_filter,
                        min_filter,
                        mipmap_filter,
                        ..Default::default()
                    }
                }

                fn gltf_texture_to_wgpu(renderer: &Renderer, resources: &mut Resources, texture: gltf::texture::Texture, textures: &Vec<Handle<Texture>>) -> Handle<Texture> {
                    // texture.sampler(), todo: handle this correctly, texture may have more than one sampler
                    let sampler = renderer.device.create_sampler(&gltf_sampler_to_wgpu(texture.sampler()));
                    let sampler = resources.store(sampler);
                    
                    let image_index = texture.source().index();
                    let texture_handle = textures[image_index].clone();
                    let texture = resources.get_mut(&texture_handle).unwrap();
                    texture.sampler = sampler;

                    texture_handle
                }

                let material = {
                    let material = primitive.material();

                    let mut builder = Material::builder();
    
                    let name = material.name();
                    let double_sided = material.double_sided();
                    let alpha_mode = match material.alpha_mode() {
                        gltf::material::AlphaMode::Opaque => AlphaMode::Opaque,
                        gltf::material::AlphaMode::Mask => AlphaMode::Mask { cutoff: material.alpha_cutoff().unwrap_or(0.5) },
                        gltf::material::AlphaMode::Blend => AlphaMode::Blend,
                    };

                    if let Some(name) = name {
                        builder = builder.name(name);
                    }
                    builder = builder.double_sided(double_sided);
                    builder = builder.alpha_mode(alpha_mode);
                    
                    let pbr = material.pbr_metallic_roughness();
                    
                    if let Some(texture) = pbr.base_color_texture() {
                        builder = builder.albedo_texture(gltf_texture_to_wgpu(renderer, resources, texture.texture(), &import_data.textures));
                    };
                    
                    builder = builder.albedo(pbr.base_color_factor().into());
                    
                    if let Some(texture) = pbr.metallic_roughness_texture() {
                        builder = builder.metallic_roughness_texture(gltf_texture_to_wgpu(renderer, resources, texture.texture(), &import_data.textures));
                    };

                    builder = builder
                        .metallic_factor(pbr.metallic_factor())
                        .roughness_factor(pbr.roughness_factor());

                    if let Some(normal_texture) = material.normal_texture() {
                        builder = builder
                            .normal_texture(gltf_texture_to_wgpu(renderer, resources, normal_texture.texture(), &import_data.textures))
                            .normal_scale(normal_texture.scale());
                    }
                    
                    if let Some(occlusion_texture) = material.occlusion_texture() {
                        builder = builder
                            .occlusion_texture(gltf_texture_to_wgpu(renderer, resources, occlusion_texture.texture(), &import_data.textures))
                            .occlusion_strength(occlusion_texture.strength());
                    }
                    
                    if let Some(texture) = material.emissive_texture() {
                        builder = builder.emissive_texture(gltf_texture_to_wgpu(renderer, resources, texture.texture(), &import_data.textures));
                    };
                    
                    if let Some(texture) = material.occlusion_texture() {
                        builder = builder.emissive_texture(gltf_texture_to_wgpu(renderer, resources, texture.texture(), &import_data.textures));
                    }

                    builder = builder.emissive_factor(material.emissive_factor().into());

                    // let material = Material::new(renderer, resources, name.as_deref(), double_sided, alpha_mode, albedo_texture, albedo, metallic_roughness_texture, metallic_factor, roughness_factor, normal_texture, normal_scale, occlusion_texture, occlusion_strength, emissive_texture, emissive_factor);
                    let material = builder.build(renderer, resources);
                    
                    resources.store(material)
                };

                meshes.push(Mesh::new(renderer, resources, mesh.name(), vertices, indices, Some(material)));
            }
            
            let model = resources.store(Model {
                meshes,
            });

            builder = builder.add_component(model);
        }

        for child in node.children() {
            builder = builder.add_child(visit(renderer, resources, child, import_data));
        }

        builder.build()
    }
    
    let path = path.as_ref();

    let scene_name = match scene.name() {
        Some(name) => name.to_owned(),
        None => match path.file_name() {
            Some(name) => (*name.to_string_lossy()).into(),
            None => "gltf_scene".into(),
        },
    };

    let mut builder = Node::builder(&scene_name);
    
    for node in scene.nodes() {
        // log::trace!("Node #{} has {} children", node.index(), node.children().count());
        builder = builder.add_child(visit(renderer, resources, node, &import_data));
    }

    Ok(builder.build())
}

#[derive(Debug)]
pub enum ModelLoadError {
    IoError(std::io::Error),
    ImageError(image::ImageError),
    TobjError(tobj::LoadError),
}

#[derive(Debug)]
pub enum SceneLoadError {
    SceneNotFound,
    GltfError(gltf::Error),
}

impl std::fmt::Display for ModelLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelLoadError::IoError(err) => err.fmt(f),
            ModelLoadError::ImageError(err) => err.fmt(f),
            ModelLoadError::TobjError(err) => err.fmt(f),
            // ModelLoadError::GltfError(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for ModelLoadError {}

pub(crate) fn compute_tangents(vertices: &mut Vec<ModelVertex>, indices: &[u32]) {
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
        vertices[c[0] as usize].bitangent = 
            (bitangent + glam::Vec3::from(vertices[c[0] as usize].bitangent)).into();
        vertices[c[1] as usize].bitangent = 
            (bitangent + glam::Vec3::from(vertices[c[1] as usize].bitangent)).into();
        vertices[c[2] as usize].bitangent = 
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
}
