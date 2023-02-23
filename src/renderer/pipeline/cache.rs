use std::collections::HashMap;

use crate::renderer::{VertexLayoutType, Renderer, PositionVertex, ModelVertex, Vertex};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) struct PipelineProperties {
    pub transparent: bool,
    pub double_sided: bool,
    pub colour_format: wgpu::TextureFormat,
    pub depth_format: Option<wgpu::TextureFormat>,
}

pub(crate) struct PipelineCache {
    pipelines: HashMap<PipelineProperties, wgpu::RenderPipeline>,
    shader: wgpu::ShaderModule,
    vertex_type: VertexLayoutType,
}

impl PipelineCache {
    pub fn new<'a>(
        renderer: &Renderer,
        vertex_type: VertexLayoutType,
        shader: wgpu::ShaderModuleDescriptor,
    ) -> PipelineCache {
        let shader = renderer.device.create_shader_module(shader);

        PipelineCache {
            pipelines: hashmap!{},
            shader,
            vertex_type,
        }
    }

    pub fn prepare_pipeline(&mut self, renderer: &Renderer, index: PipelineProperties, bind_group_layout: &wgpu::BindGroupLayout) {
        let cull_mode = if index.double_sided {
            None
        } else {
            Some(wgpu::Face::Back)
        };
        
        let depth_write_enabled = !index.transparent;
        
        let layout = renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let pipeline = renderer.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &self.shader,
                entry_point: "vs_main",
                buffers: &match self.vertex_type {
                    VertexLayoutType::Position => vec![PositionVertex::desc()],
                    VertexLayoutType::Model => vec![ModelVertex::desc()],
                    VertexLayoutType::None => vec![],
                },
            },
            fragment: Some(wgpu::FragmentState {
                module: &self.shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: index.colour_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: index.depth_format.map(|format| wgpu::DepthStencilState {
                format,
                depth_write_enabled,
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

        self.pipelines.insert(index, pipeline);
    }

    pub fn get_pipeline(&self, index: PipelineProperties) -> Option<&wgpu::RenderPipeline> {
        self.pipelines.get(&index)
    }
}
