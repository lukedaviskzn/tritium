use wgpu::util::DeviceExt;

use crate::resource::{Resources, Handle};

use super::{Renderer, BindingHolder};

// pub struct Uniform<T: bytemuck::Pod + bytemuck::Zeroable> {
#[derive(Debug)]
pub struct UniformBuffer {
    buffer: Handle<wgpu::Buffer>,
}

impl UniformBuffer {
    pub fn from_value<T: bytemuck::Pod + bytemuck::Zeroable>(renderer: &Renderer, resources: &mut Resources, uniform: T) -> UniformBuffer {
        let buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // let bind_group = resources.store(renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     layout: &renderer.bind_group_layouts[&BindGroupLayoutType::Uniform],
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: buffer.as_entire_binding(),
        //         }
        //     ],
        //     label: None,
        // }));

        let buffer = resources.store(buffer);

        UniformBuffer {
            buffer,
        }
    }

    pub(crate) fn binding_type() -> wgpu::BindingType {
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }

    pub(crate) fn binding_resource(&self) -> BindingHolder {
        BindingHolder::Buffer(self.buffer.clone())
    }

    // pub(crate) fn binding_resource(&self) -> wgpu::BindingResource {
    //     self.buffer.as_entire_binding()
    // }

    // pub fn bind_group(&self) -> Handle<wgpu::BindGroup> {
    //     self.bind_group.clone()
    // }
}

#[derive(Debug)]
pub struct StorageBuffer {
    buffer: Handle<wgpu::Buffer>,
    len_buffer: Handle<wgpu::Buffer>,
}

impl StorageBuffer {
    pub fn from_slice<T: bytemuck::Pod + bytemuck::Zeroable>(renderer: &Renderer, resources: &mut Resources, values: &[T]) -> StorageBuffer {
        let num_items = values.len() as u32;
        
        let buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(values),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let len_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[num_items]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // let bind_group = resources.store(renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     layout: &renderer.bind_group_layouts[&BindGroupLayoutType::StorageArray],
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: buffer.as_entire_binding(),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: len_buffer.as_entire_binding(),
        //         },
        //     ],
        //     label: None,
        // }));

        let buffer = resources.store(buffer);
        let len_buffer = resources.store(len_buffer);

        StorageBuffer {
            // uniforms,
            buffer,
            len_buffer,
            // bind_group,
            // marker: PhantomData,
        }
    }
    
    pub fn from_bytes(renderer: &Renderer, resources: &mut Resources, values: &[u8], item_size: usize) -> StorageBuffer {
        let num_items = (values.len() / item_size) as u32;
        
        let buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: values,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        
        let len_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[num_items]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // let bind_group = resources.store(renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     layout: &renderer.bind_group_layouts[&BindGroupLayoutType::StorageArray],
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: buffer.as_entire_binding(),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: len_buffer.as_entire_binding(),
        //         },
        //     ],
        //     label: None,
        // }));

        let buffer = resources.store(buffer);
        let len_buffer = resources.store(len_buffer);

        StorageBuffer {
            // uniforms,
            buffer,
            len_buffer,
            // bind_group,
            // marker: PhantomData,
        }
    }

    pub fn binding_types() -> Vec<wgpu::BindingType> {
        vec![
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        ]
    }

    pub(crate) fn binding_resources(&self) -> [BindingHolder; 2] {
        [BindingHolder::Buffer(self.buffer.clone()), BindingHolder::Buffer(self.len_buffer.clone())]
    }
}
