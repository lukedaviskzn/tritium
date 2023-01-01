use std::{path::Path, io::{BufReader, BufRead}, vec};

use crate::{resource::{Texture, CubeMap, Material, Handle}};

use super::{Renderer, VertexLayoutType, PositionVertex, Vertex, ModelVertex, UniformBuffer, StorageBuffer};

pub struct Shader {
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) inputs: Vec<ShaderInput>,
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
}

impl Shader {
    fn new(
        renderer: &Renderer,
        // device: &wgpu::Device,
        name: &str,
        shader_inputs: Vec<ShaderInput>,
        // layout: &wgpu::PipelineLayout,
        color_format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
        vertex_type: VertexLayoutType,
        // vertex_layouts: &[wgpu::VertexBufferLayout],
        // bind_group_layouts: &HashMap<BindGroupLayoutType, wgpu::BindGroupLayout>,
        shader: wgpu::ShaderModuleDescriptor,
    ) -> Shader {
        let shader = renderer.device.create_shader_module(shader);

        let bind_group_layout_entries = {
            let mut bind_group_layout_entries = vec![];
            let mut current_binding = 0;
            
            for input in &shader_inputs {
                let binding_types = match input.layout() {
                    BindingResourceType::Material => Material::binding_types(),
                    BindingResourceType::Texture => Texture::binding_types(),
                    BindingResourceType::CubeMap => CubeMap::binding_types(),
                    BindingResourceType::Uniform => vec![UniformBuffer::binding_type()],
                    BindingResourceType::Storage => StorageBuffer::binding_types(),
                };
                
                for binding_type in binding_types {
                    bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
                        binding: current_binding,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: binding_type,
                        count: None,
                    });

                    current_binding += 1;
                }
            }

            bind_group_layout_entries
        };

        let bind_group_layout = renderer.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("'{name}': bind group layout")),
            entries: bind_group_layout_entries.as_slice(),
        });

        let layout = renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("'{name}': pipeline layout")),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
    
        let pipeline = renderer.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("'{name}': render pipeline")),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    match vertex_type {
                        VertexLayoutType::Position => PositionVertex::desc(),
                        VertexLayoutType::Model => ModelVertex::desc(),
                    }
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
                format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Shader {
            pipeline,
            inputs: shader_inputs,
            bind_group_layout,
        }
    }

    pub fn from_resource<P: AsRef<Path>>(
        renderer: &Renderer,
        // device: &wgpu::Device,
        // bind_group_layouts: &HashMap<BindGroupLayoutType, wgpu::BindGroupLayout>,
        // vertex_layouts: &[wgpu::VertexBufferLayout],
        // colour_format: wgpu::TextureFormat,
        // depth_format: Option<wgpu::TextureFormat>,
        path: P,
    ) -> Result<Shader, ShaderLoadError> {
        log::trace!("Loading Pipeline {}", path.as_ref().to_str().unwrap());
        
        let resource: ShaderResource = ron::from_str(&std::fs::read_to_string(&path)
            .map_err(|err| ShaderLoadError::IoError(err))?)
            .map_err(|err| ShaderLoadError::ParseError(err))?;

        let name = format!("{} Shader Module", &resource.name);

        let shader_path = path.as_ref().parent().unwrap_or(&Path::new("./")).join(&resource.shader_file);

        log::trace!("Loading Shader {}", shader_path.to_str().unwrap());

        let shader = wgpu::ShaderModuleDescriptor {
            label: Some(&name),
            source: wgpu::ShaderSource::Wgsl(
                preprocess_wgsl(shader_path)?.into()
            ),
        };

        let colour_format = renderer.window.config.format;
        let depth_format = Some(Texture::DEPTH_FORMAT);

        Ok(Shader::new(renderer, &resource.name, resource.inputs, colour_format, depth_format, resource.vertex_type, shader))
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ShaderResource {
    name: String,
    // color_format: wgpu::TextureFormat,
    // depth_format: Option<wgpu::TextureFormat>,
    inputs: Vec<ShaderInput>,
    vertex_type: VertexLayoutType,
    shader_file: String,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BindingResourceType {
    Material, // many bindings, see material
    Texture, // 0: texture, 1: sampler
    CubeMap, // 0: cubemap, 1: sampler
    Uniform, // any type
    Storage, // { len: u32, data: array<T> }
}

// #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
// pub enum InputAccess {
//     Node,
//     Global,
//     EngineGlobal,
// }

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) enum ShaderInput {
    MeshMaterial,
    Node {
        ty: BindingResourceType,
        res: String,
    },
    GlobalNode {
        ty: BindingResourceType,
        node: String,
        res: String,
    },
    Scene {
        // layout: BindGroupLayoutType,
        collection: String,
    },
    Resource {
        ty: BindingResourceType,
        res: String,
    },
}

impl ShaderInput {
    pub fn layout(&self) -> BindingResourceType {
        match self {
            ShaderInput::MeshMaterial => BindingResourceType::Material,
            ShaderInput::Node { ty, .. } => *ty,
            ShaderInput::GlobalNode { ty, .. } => *ty,
            ShaderInput::Scene { .. } => BindingResourceType::Storage,
            ShaderInput::Resource { ty, .. } => *ty,
            // ShaderInput::Scene { layout, .. } => *layout,
        }
    }
}

#[derive(Debug)]
pub enum ShaderLoadError {
    IoError(std::io::Error),
    ParseError(ron::error::SpannedError),
}

impl std::fmt::Display for ShaderLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderLoadError::IoError(err) => err.fmt(f),
            ShaderLoadError::ParseError(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for ShaderLoadError {}

// pub(crate) trait BindingGenerator {
//     fn binding_resources(&self, renderer: &Renderer, resources: &mut Resources) -> Vec<wgpu::BindingResource>;
// }

#[derive(Debug, Clone)]
pub enum BindingHolder {
    Buffer(Handle<wgpu::Buffer>),
    Texture(Handle<wgpu::TextureView>, Handle<wgpu::Sampler>),
}

fn preprocess_wgsl<P: AsRef<Path>>(path: P) -> Result<String, ShaderLoadError> {
    fn preprocess_internal<P: AsRef<Path>>(path: P, current_binding: &mut u32, collapse: bool) -> Result<String, ShaderLoadError> {
        fn get_args<'a, T: serde::Deserialize<'a>>(line: &'a str, command: &str) -> Result<Option<T>, ()> {
            if !line.starts_with("//!") {
                return Ok(None);
            }

            let line = &line[3..];
            
            let end_index = line[0..].find(|c| match c {
                // find first non-alphanumeric (and not underscore)
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => false,
                _ => true,
            }).unwrap_or(line.len());

            if &line[0..end_index] == command {
                match ron::from_str(line[end_index..].trim()).map_err(|_| ()) {
                    Ok(args) => Ok(Some(args)),
                    Err(_) => Err(()),
                }
            } else {
                Ok(None)
            }
        }
    
        let file = std::fs::File::open(path.as_ref()).map_err(|err| ShaderLoadError::IoError(err))?;
        let reader = BufReader::new(file);
    
        let mut lines = vec![];
    
        for (i, line) in reader.lines().enumerate() {
            let line = line.unwrap().trim().to_owned();

            let error_msg = format!("Invalid macro args on line {} of file '{}'.", i+1, path.as_ref().to_string_lossy());
    
            let line = if let Some((include_path,)) = get_args::<(String,)>(&line, "include").expect(&error_msg) {
                let path = Path::new(path.as_ref()).parent().unwrap_or(&Path::new("./")).join(include_path);
                
                let line = preprocess_internal(path, current_binding, true)?;

                line
            } else if let Some(_) = get_args::<()>(&line, "binding").expect(&error_msg) {
                let line = format!("@group(0) @binding({})", current_binding);
                *current_binding += 1;
                line
            } else if line.starts_with("//") { // remove comment lines
                "".into()
            } else {
                line
            };
            
            lines.push(line);
        }
    
        Ok(lines.join(if collapse { " " } else { "\n" }))
    }

    preprocess_internal(path, &mut 0, false)
}