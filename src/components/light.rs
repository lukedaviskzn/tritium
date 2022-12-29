use crate::{engine::Rgba, util::AsAny, renderer::{Renderable, RenderInput, Renderer, SceneInputItem}, resource::Resources, node::{Component, NodeDescriptor}};

use super::Transform;

pub struct PointLight(pub Rgba);

impl PointLight {
    pub fn new(colour: Rgba) -> PointLight {
        PointLight(colour)
    }
}

impl AsAny for PointLight {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl Component for PointLight {
    fn as_renderable(&self) -> Option<&dyn Renderable> { Some(self) }
}

impl Renderable for PointLight {
    fn render_inputs(&self, node: &NodeDescriptor, _renderer: &Renderer, _resources: &mut Resources) -> Vec<RenderInput> {
        let transform = node.get_component::<Transform>().expect("Attempted to render point light without missing transform.");

        let uniform = PointLightUniform::new(transform, self.0);
        
        vec![RenderInput::SceneInput("point_lights".into(), SceneInputItem::new(uniform))]
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLightUniform {
    position: [f32; 3],
    _padding0: u32,
    colour: [f32; 4],
}

impl PointLightUniform {
    pub fn new(transform: &Transform, colour: Rgba) -> PointLightUniform {
        PointLightUniform {
            position: transform.global_matrix().to_scale_rotation_translation().2.into(),
            _padding0: 0,
            colour: colour.into(),
        }
    }
}

/// Initial direction is directly downwards (-Y)
pub struct DirectionalLight(pub Rgba);

impl DirectionalLight {
    pub fn new(colour: Rgba) -> DirectionalLight {
        DirectionalLight(colour)
    }
}

impl AsAny for DirectionalLight {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl Component for DirectionalLight {
    fn as_renderable(&self) -> Option<&dyn Renderable> { Some(self) }
}

impl Renderable for DirectionalLight {
    fn render_inputs(&self, node: &NodeDescriptor, _renderer: &Renderer, _resources: &mut Resources) -> Vec<RenderInput> {
        let transform = node.get_component::<Transform>().expect("Attempted to render point light without missing transform.");

        let uniform = DirectionalLightUniform::new(transform, self.0);
        
        vec![RenderInput::SceneInput("directional_lights".into(), SceneInputItem::new(uniform))]
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DirectionalLightUniform {
    direction: [f32; 3],
    _padding0: u32,
    colour: [f32; 4],
}

impl DirectionalLightUniform {
    pub fn new(transform: &Transform, colour: Rgba) -> DirectionalLightUniform {
        let rotation = transform.global_matrix().to_scale_rotation_translation().1;
        let direction = rotation * glam::Vec3::NEG_Y;
        DirectionalLightUniform {
            direction: direction.into(),
            _padding0: 0,
            colour: colour.into(),
        }
    }
}
