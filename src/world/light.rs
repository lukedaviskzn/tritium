use crate::{engine::Rgba, util::AsAny, renderer::{Renderable, Uniform, RenderInput, Renderer}, resource::Resources};

use super::{Component, Transform, NodeDescriptor};

pub struct Light(pub Rgba);

impl Light {
    pub fn new(colour: Rgba) -> Light {
        Light(colour)
    }
}

impl AsAny for Light {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl Component for Light {
    fn as_renderable(&self) -> Option<&dyn Renderable> {
        Some(self)
    }
}

impl Renderable for Light {
    fn render_inputs(&self, node: &NodeDescriptor, renderer: &Renderer, resources: &mut Resources) -> Vec<RenderInput> {
        let transform = node.get_component::<Transform>().unwrap_or(&Transform::IDENTITY);

        let uniform = Uniform::new(renderer, resources, LightUniform::new(transform, self.0));

        // vec![RenderInput::new("light", RenderInputStorage::BindGroup(uniform.bind_group()))]
        vec![RenderInput::BindGroup("light".into(), uniform.bind_group())]
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    position: [f32; 3],
    _padding0: u32,
    colour: [f32; 4],
}

impl LightUniform {
    pub fn new(transform: &Transform, colour: Rgba) -> LightUniform {
        LightUniform {
            position: transform.global_matrix().to_scale_rotation_translation().2.into(),
            _padding0: 0,
            colour: colour.into(),
        }
    }
}
