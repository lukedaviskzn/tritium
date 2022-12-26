use wgpu::util::DeviceExt;

use crate::resource::{Handle, Resources};

use super::{Renderer, BindGroupLayoutType};

pub struct Uniform<T: bytemuck::Pod + bytemuck::Zeroable> {
    uniform: T,
    buffer: Handle<wgpu::Buffer>,
    bind_group: Handle<wgpu::BindGroup>,
}

impl<T: bytemuck::Pod + bytemuck::Zeroable> Uniform<T> {
    pub fn new(renderer: &Renderer, resources: &mut Resources, uniform: T) -> Uniform<T> {
        let buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = resources.store(renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &renderer.bind_group_layouts[&BindGroupLayoutType::Uniform],
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: None,
        }));

        let buffer = resources.store(buffer);

        Uniform {
            uniform,
            buffer,
            bind_group,
        }
    }

    pub fn bind_group_layout_descriptor<'a>() -> wgpu::BindGroupLayoutDescriptor<'a> {
        wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: None,
        }
    }

    pub fn bind_group(&self) -> Handle<wgpu::BindGroup> {
        self.bind_group.clone()
    }

    pub fn update(&mut self, queue: &wgpu::Queue, resources: &Resources, new_value: T) {
        self.uniform = new_value;
        queue.write_buffer(resources.get(&self.buffer).unwrap(), 0, bytemuck::cast_slice(&[self.uniform]));
    }

    pub fn mutate<F: FnMut(&mut T) -> ()>(&mut self, queue: &wgpu::Queue, resources: &Resources, mut f: F) {
        f(&mut self.uniform);
        queue.write_buffer(resources.get(&self.buffer).unwrap(), 0, bytemuck::cast_slice(&[self.uniform]));
    }

    pub fn contents(&self) -> &T {
        &self.uniform
    }
}
