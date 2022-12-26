use crate::{util::AsAny, engine::{EngineScript, UpdateContext}, resource::Resources, world::Node, renderer::{Renderable, RenderInput, Uniform, Renderer}};

use super::{Component, NodeDescriptor};

#[derive(Debug)]
pub struct Transform {
    pub translation: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
    global_matrix: glam::Mat4,
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        translation: glam::Vec3::ZERO,
        rotation: glam::Quat::IDENTITY,
        scale: glam::Vec3::ONE,
        global_matrix: glam::Mat4::IDENTITY,
    };

    pub fn new(translation: glam::Vec3, rotation: glam::Quat, scale: glam::Vec3) -> Transform {
        let matrix = glam::Mat4::from_scale_rotation_translation(scale, rotation, translation);
        Transform {
            translation,
            scale,
            rotation,
            global_matrix: matrix,
        }
    }

    pub fn from_tranlation(translation: glam::Vec3) -> Transform {
        Transform::new(translation, glam::Quat::IDENTITY, glam::Vec3::ONE)
    }

    pub fn from_tranlation_rotation(translation: glam::Vec3, rotation: glam::Quat) -> Transform {
        Transform::new(translation, rotation, glam::Vec3::ONE)
    }

    pub fn from_tranlation_scale(translation: glam::Vec3, scale: glam::Vec3) -> Transform {
        Transform::new(translation, glam::Quat::IDENTITY, scale)
    }

    pub fn from_rotation(rotation: glam::Quat) -> Transform {
        Transform::new(glam::Vec3::ZERO, rotation, glam::Vec3::ONE)
    }

    pub fn from_rotation_scale(rotation: glam::Quat, scale: glam::Vec3) -> Transform {
        Transform::new(glam::Vec3::ZERO, rotation, scale)
    }

    pub fn from_scale(scale: glam::Vec3) -> Transform {
        Transform::new(glam::Vec3::ZERO, glam::Quat::IDENTITY, scale)
    }

    pub fn matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    pub fn update_global_matrix(&mut self, parent_matrix: glam::Mat4) {
        self.global_matrix = parent_matrix * self.matrix();
    }

    pub fn global_matrix(&self) -> glam::Mat4 {
        self.global_matrix
    }
}

impl Default for Transform {
    fn default() -> Transform {
        Self::IDENTITY
    }
}

impl AsAny for Transform {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl Component for Transform {
    fn as_renderable(&self) -> Option<&dyn Renderable> {
        Some(self)
    }
}

impl Renderable for Transform {
    fn render_inputs(&self, _node: &NodeDescriptor, renderer: &Renderer, resources: &mut Resources) -> Vec<RenderInput> {
        let uniform = Uniform::new(
            renderer, resources,
            TransformUniform::new(self),
        );
        
        // vec![RenderInput::new("transform", RenderInputStorage::BindGroup(uniform.bind_group()))]
        vec![RenderInput::BindGroup("transform".into(), uniform.bind_group())]
    }
}

pub trait HasTransform {
    fn transform(&self) -> &Transform;
    fn transform_mut(&mut self) -> &mut Transform;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformUniform {
    model_matrix: [[f32; 4]; 4],
}

impl TransformUniform {
    pub fn new(transform: &Transform) -> TransformUniform {
        TransformUniform {
            model_matrix: transform.global_matrix.to_cols_array_2d(),
        }
    }

    pub fn update(&mut self, transform: &Transform) {
        self.model_matrix = transform.global_matrix.to_cols_array_2d();
    }
}

pub(crate) struct TransformPropagationScript;

impl TransformPropagationScript {
    fn update_transforms(&mut self, node: &mut NodeDescriptor) {
        fn visit(node: &mut Node, parent_matrix: glam::Mat4) {
            let matrix = if let Some(transform) = node.get_component_mut::<Transform>() {
                transform.update_global_matrix(parent_matrix);

                transform.global_matrix()
            } else {
                parent_matrix
            };
            
            for child in &mut node.desc.children {
                visit(child, matrix);
            }
        }

        for child in &mut node.children {
            visit(child, glam::Mat4::IDENTITY);
        }
    }
}

impl EngineScript for TransformPropagationScript {
    fn post_update(&mut self, node: &mut NodeDescriptor, _context: &UpdateContext, _resources: &mut Resources) {
        self.update_transforms(node);
    }

    fn post_tick(&mut self, node: &mut NodeDescriptor, _context: &UpdateContext, _resources: &mut Resources) {
        self.update_transforms(node);
    }
}
