use crate::{node::{Component, NodeDescriptor}, util::AsAny, renderer::{RenderInput, Renderable, Renderer, UniformBuffer}, resource::Resources, components::Transform};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: glam::Mat4 = glam::Mat4::from_cols_array(&[
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
]);

pub struct Camera {
    // pub transform: Transform,
    // pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self, transform: &Transform, aspect: f32) -> glam::Mat4 {
        let view = transform.global_matrix().inverse();
        let proj = glam::Mat4::perspective_rh(self.fovy, aspect, self.znear, self.zfar);

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

impl AsAny for Camera {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl Component for Camera {
    fn as_renderable(&self) -> Option<&dyn Renderable> { Some(self) }
}

impl Renderable for Camera {
    fn render_inputs(&self, node: &NodeDescriptor, renderer: &Renderer, resources: &mut Resources) -> Vec<RenderInput> {
        let transform = node.get_component::<Transform>().unwrap_or(&Transform::IDENTITY);

        let uniform = UniformBuffer::from_value(
            renderer, resources,
            CameraUniform::new(self, transform, renderer.window.size.width as f32 / renderer.window.size.height as f32),
        );

        // vec![RenderInput::new("camera", RenderInputStorage::BindGroup(uniform.bind_group()))]
        // vec![RenderInput::BindGroup("camera".into(), uniform.bind_group())]
        // vec![RenderInput::UniformBuffer("camera".into(), uniform)]
        vec![RenderInput::BindingResources("camera".into(), uniform.binding_resource())]
    }
}

// pub(crate) struct CameraUpdateScript;

// impl NodeScript for CameraUpdateScript {
//     fn pre_update(&mut self, node: &mut crate::world::NodeDescriptor, context: &crate::engine::UpdateContext, _resources: &mut crate::resource::Resources) {
//         for child in &mut node.children {
//             child.traverse_mut(&mut |node| if let Some(camera) = node.get_component_mut::<Camera>() {
//                 camera.aspect = context.window_size.x / context.window_size.y;
//             });
//         }
//     }
// }

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 3],
    _padding: u32,
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new(camera: &Camera, transform: &Transform, aspect: f32) -> CameraUniform {
        let (_, _, translation) = transform.global_matrix().to_scale_rotation_translation();

        let view_proj = camera.build_view_projection_matrix(transform, aspect).to_cols_array_2d();

        CameraUniform {
            view_position: translation.into(),
            _padding: 0,
            view_proj,
        }
    }
}
