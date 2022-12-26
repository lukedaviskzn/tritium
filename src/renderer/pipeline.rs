use std::path::Path;

use crate::{resource::Texture};

use super::{Renderer, VertexLayoutType, PositionVertex, Vertex, ModelVertex};

pub struct Shader {
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) inputs: Vec<ShaderInput>,
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

        let mut bind_group_layout_order = vec![];

        for input in &shader_inputs {
            bind_group_layout_order.push(&renderer.bind_group_layouts[&input.layout()]);
        }

        let layout = renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(name),
            bind_group_layouts: &bind_group_layout_order,
            push_constant_ranges: &[],
        });
    
        let pipeline = renderer.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
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
                    blend: Some(wgpu::BlendState {
                        alpha: wgpu::BlendComponent::REPLACE,
                        color: wgpu::BlendComponent::REPLACE,
                    }),
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
            .map_err(|e| ShaderLoadError::IoError(e))?)
            .map_err(|e| ShaderLoadError::ParseError(e))?;

        let name = format!("{} Shader Module", &resource.name);

        let shader_path = path.as_ref().parent().unwrap_or(&Path::new("./")).join(&resource.shader_file);

        log::trace!("Loading Shader {}", shader_path.to_str().unwrap());

        let shader = wgpu::ShaderModuleDescriptor {
            label: Some(&name),
            source: wgpu::ShaderSource::Wgsl(
                std::fs::read_to_string(shader_path).map_err(|e| ShaderLoadError::IoError(e))?.into()
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
pub enum BindGroupLayoutType {
    Material,
    Uniform,
    Texture,
    CubeMap,
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
        layout: BindGroupLayoutType,
        bind_group: String,
    },
    Global {
        layout: BindGroupLayoutType,
        resource: String,
        bind_group: String,
    },
}

impl ShaderInput {
    pub fn layout(&self) -> BindGroupLayoutType {
        match self {
            ShaderInput::MeshMaterial => BindGroupLayoutType::Material,
            ShaderInput::Node { layout, .. } => *layout,
            ShaderInput::Global { layout, .. } => *layout,
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
