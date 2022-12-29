#[macro_use] extern crate maplit;

use std::{collections::HashMap, time::Duration};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};

use crate::renderer::SceneInputItem;

pub mod resource;
pub mod renderer;
pub mod input;
pub mod node;
pub mod util;
pub mod engine;
pub mod camera;
pub mod components;

const CLEAR_COLOUR: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.05,
    b: 0.15,
    a: 1.0,
};

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
struct VideoConfig {
    vsync: bool,
}

struct EngineState {
    renderer: renderer::Renderer,
    global_root: node::Node,
    keyboard_manager: resource::Handle<input::KeyboardManager>,
    tick_keyboard_manager: resource::Handle<input::KeyboardManager>,
    mouse_manager: resource::Handle<input::MouseManager>,
    tick_mouse_manager: resource::Handle<input::MouseManager>,
    frame_counter: engine::FrameCounter,
    resources: resource::Resources,
    ticks_per_second: u32,
    max_fps: Option<u32>,
}

impl EngineState {
    async fn new<F: FnMut(&renderer::Renderer, &mut resource::Resources) -> node::Node>(window: Window, mut scene_builder: F) -> EngineState {
        let video_config: VideoConfig = confy::load("wgpu-game-engine", Some("video")).unwrap();

        let renderer = renderer::Renderer::new(window, video_config.vsync).await;
        let mut resources = resource::Resources::new();

        log::info!("Building Scene");
        let current_scene = scene_builder(&renderer, &mut resources);
        log::info!("Scene Built");

        let global_root = node::Node::builder("global_root")
            // .add_script(camera::CameraUpdateScript)
            .add_script(components::TransformPropagationScript)
            .add_child(current_scene)
            .build();
        
        EngineState {
            renderer,
            global_root,
            keyboard_manager: resources.store(input::KeyboardManager::new()),
            tick_keyboard_manager: resources.store(input::KeyboardManager::new()),
            mouse_manager: resources.store(input::MouseManager::new()),
            tick_mouse_manager: resources.store(input::MouseManager::new()),
            frame_counter: engine::FrameCounter::new(),
            resources,
            ticks_per_second: 60,
            max_fps: Some(120),
        }
    }

    fn input(&mut self, event: &Event<'_, ()>) -> bool {
        match event {
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    if self.renderer.window.focused {
                        let delta = glam::vec2(delta.0 as f32, delta.1 as f32);
                        self.resources.get_mut(&self.mouse_manager).expect("Lost handle to mouse manager.").update_delta(delta);
                        self.resources.get_mut(&self.tick_mouse_manager).expect("Lost handle to mouse manager.").update_delta(delta);
                    }
                },
                DeviceEvent::MouseWheel { delta } => {
                    if self.renderer.window.focused {
                        self.resources.get_mut(&self.mouse_manager).expect("Lost handle to mouse manager.").update_scroll_delta(*delta);
                        self.resources.get_mut(&self.tick_mouse_manager).expect("Lost handle to mouse manager.").update_scroll_delta(*delta);
                    }
                },
                _ => {},
            },
            Event::WindowEvent { window_id, event } => if *window_id == self.renderer.window.window.id() {
                match event {
                    WindowEvent::KeyboardInput { input, .. } => {
                        self.resources.get_mut(&self.keyboard_manager).expect("Lost handle to keyboard manager.").input(input);
                        self.resources.get_mut(&self.tick_keyboard_manager).expect("Lost handle to keyboard manager.").input(input);
                    },
                    WindowEvent::MouseInput { state, button, .. } => {
                        self.resources.get_mut(&self.mouse_manager).expect("Lost handle to mouse manager.").input(*state, *button);
                        self.resources.get_mut(&self.tick_mouse_manager).expect("Lost handle to mouse manager.").input(*state, *button);
                    },
                    WindowEvent::CursorMoved { position, .. } => {
                        let position = glam::vec2(position.x as f32, position.y as f32);
                        self.resources.get_mut(&self.mouse_manager).expect("Lost handle to mouse manager.").update_position(position);
                        self.resources.get_mut(&self.tick_mouse_manager).expect("Lost handle to mouse manager.").update_position(position);
                    },
                    _ => {},
                }
            },
            _ => {},
        }
        
        false
    }

    // fn first_update(&mut self, context: &engine::UpdateContext) {
    //     self.resources.drop_invalid();

    //     self.current_scene.traverse_mut(&mut |node| for script in &mut node.scripts {
    //         script.first_update(&mut node.desc, context, &mut self.resources)
    //     });
    // }

    fn pre_update(&mut self, context: &engine::UpdateContext) {
        self.resources.drop_invalid();
        
        self.global_root.traverse_mut(&mut |node| for script in &mut node.scripts {
            script.pre_update(&mut node.desc, context, &mut self.resources)
        });
    }

    fn update(&mut self, context: &engine::UpdateContext) {
        self.global_root.traverse_mut(&mut |node| for script in &mut node.scripts {
            script.update(&mut node.desc, context, &mut self.resources)
        });
    }

    fn post_update(&mut self, context: &engine::UpdateContext) {
        self.global_root.traverse_mut(&mut |node| for script in &mut node.scripts {
            script.post_update(&mut node.desc, context, &mut self.resources)
        });
    }

    fn pre_tick(&mut self, context: &engine::UpdateContext) {
        self.global_root.traverse_mut(&mut |node| for script in &mut node.scripts {
            script.pre_tick(&mut node.desc, context, &mut self.resources)
        });
    }

    fn tick(&mut self, context: &engine::UpdateContext) {
        self.global_root.traverse_mut(&mut |node| for script in &mut node.scripts {
            script.tick(&mut node.desc, context, &mut self.resources)
        });
    }

    fn post_tick(&mut self, context: &engine::UpdateContext) {
        self.global_root.traverse_mut(&mut |node| for script in &mut node.scripts {
            script.post_tick(&mut node.desc, context, &mut self.resources)
        });
    }

    fn extract(&mut self) -> Vec<renderer::QueuedRenderObject> {
        
        struct MeshInput {
            vertex_buffer: resource::Handle<wgpu::Buffer>,
            index_buffer: resource::Handle<wgpu::Buffer>,
            material: Option<resource::Handle<resource::Material>>,
            num_elements: u32,
        }

        struct ExtractedNode {
            shader: Option<resource::Handle<renderer::Shader>>,
            meshes: Vec<MeshInput>,
            binding_resources: HashMap<String, renderer::BindingHolder>,
        }
        
        // Inputs associated with individual nodes, i.e. Transform
        let mut extracted_nodes = hashmap!{};
        // Inputs associated with the scene as a whole, i.e. All lights
        let mut scene_data = hashmap!{};

        {
            fn visit(node: &mut node::Node, resources: &mut resource::Resources, extracted_nodes: &mut HashMap<node::NodeId, ExtractedNode>, scene_data: &mut HashMap<String, Vec<SceneInputItem>>, renderer: &renderer::Renderer) {
                if node.has_component::<renderer::Invisible>() {
                    return;
                }

                let mut node_inputs = vec![];

                for component in node.get_components() {
                    if let Some(renderable) = component.as_renderable() {
                        let inputs = renderable.render_inputs(&node.desc, renderer, resources);

                        node_inputs.extend(inputs);
                    }
                }
                
                let mut node_data = ExtractedNode {
                    shader: None,
                    meshes: vec![],
                    binding_resources: hashmap!{},
                };
                
                for input in node_inputs {
                    match input {
                        renderer::RenderInput::Shader(shader) => node_data.shader = Some(shader.clone()),
                        renderer::RenderInput::Mesh {
                            vertex_buffer,
                            index_buffer,
                            material,
                            num_elements,
                        } => node_data.meshes.push(MeshInput {
                            vertex_buffer,
                            index_buffer,
                            material,
                            num_elements,
                        }),
                        // renderer::RenderInput::BindGroup(name, bind_group) => {
                        //     node_data.bind_groups.insert(name.clone(), bind_group.clone());
                        // },
                        renderer::RenderInput::BindingResources(name, resources) => {
                            node_data.binding_resources.insert(name, resources);
                        },
                        renderer::RenderInput::SceneInput(name, item) => {
                            if let Some(data) = scene_data.get_mut(&name) {
                                data.push(item);
                            } else {
                                scene_data.insert(name, vec![item]);
                            }
                        },
                    }
                }

                extracted_nodes.insert(node.id(), node_data);

                for child in &mut node.desc.children {
                    visit(child, resources, extracted_nodes, scene_data, renderer);
                }
            }
    
            visit(&mut self.global_root, &mut self.resources, &mut extracted_nodes, &mut scene_data, &self.renderer);
        }

        let empty_storage_buffer = {
            // 1 word, item size of 2 words => num items = 0
            let buffer = renderer::StorageBuffer::from_bytes(&self.renderer, &mut self.resources, &[0; 32], 64);
            buffer.binding_resources()
        };

        let scene_data = {
            let mut new_scene_data = hashmap!{};

            for (collection, items) in scene_data {
                let num_items = items.len();
                
                let data = {
                    let mut data = vec![];
                    let first_type = items[0].typeid;
                    
                    for item in items {
                        if item.typeid != first_type {
                            panic!("Scene input may not have items with different types (including generics).");
                        }
                        data.extend(item.data);
                    }

                    data
                };

                // let points: &[components::PointLightUniform] = bytemuck::cast_slice(&data);
                // log::trace!("{points:?}");
                
                let buffer = renderer::StorageBuffer::from_bytes(&self.renderer, &mut self.resources, &data, data.len() / num_items);

                new_scene_data.insert(collection, buffer.binding_resources());
            }

            new_scene_data
        };

        let mut queue = vec![];

        // let current_camera = self.resources.get_global::<world::NodeId>("current_camera").expect("Global resource 'current_camera' must be set to a valid NodeId with an attached Camera bundle.");

        for (_, ExtractedNode { shader, meshes, binding_resources }) in &extracted_nodes {
            if meshes.len() > 0 {
                let shader_handle = shader.clone().expect("Failed to render mesh, no shader specified.");
                let inputs = {
                    let shader = self.resources.get(&shader_handle).expect("Invalid shader handle.");
                    shader.inputs.clone()
                };

                for mesh in meshes {
                    // todo: a bit inefficient to recalculate order for each mesh
                    let mut ordered_binding_resources = vec![];

                    for input in &inputs {
                        let mut resources = vec![];
                        match input {
                            renderer::ShaderInput::MeshMaterial => {
                                let resource = mesh.material.as_ref().expect("Shader MeshMaterial input not present, mesh does not have material.");
                                let material = self.resources.get(resource).expect("Shader MeshMaterial input failed, mesh holds invalid handle.");
                                
                                resources.extend(material.binding_resources(&self.renderer, &self.resources))
                            }
                            renderer::ShaderInput::Node { bind_group, .. } => {
                                let resource = binding_resources.get(bind_group).expect(&format!("Shader input '{bind_group}' not present in node."));
                                
                                resources.push(resource.clone())
                            },
                            renderer::ShaderInput::Global { resource, bind_group, .. } => {
                                let node_id = self.resources.get_global::<node::NodeId>(resource).expect(&format!("Failed to get global shader input '{resource}.{bind_group}'. No such global NodeId resource '{resource}' exists."));
                                let node_data = extracted_nodes.get(node_id).expect(&format!("Failed to get global shader input '{resource}.{bind_group}'. No Node exists with NodeId as specified in global resource '{resource}'."));
                                let resource = node_data.binding_resources.get(bind_group).expect(&format!("Failed to get global shader input '{resource}.{bind_group}'. The node at '{resource}' does not have bind group '{bind_group}'."));
                                
                                resources.push(resource.clone())
                            },
                            renderer::ShaderInput::Scene { collection, .. } => {
                                // let bind_group = scene_data.get(collection).unwrap_or(&empty_storage_buffer);
                                // let resource = scene_data.get(collection).expect(&format!("Failed to find element of shader scene input '{collection}'. Empty or non-existant."));
                                let resource = scene_data.get(collection).unwrap_or(&empty_storage_buffer);
                                
                                resources.extend(resource.clone());
                            },
                        };
        
                        ordered_binding_resources.extend(resources);
                    }
                    
                    let bind_group_entries = {
                        let mut bind_group_entries = vec![];
                        let mut current_binding = 0;
                        
                        for resource in ordered_binding_resources {
                            match resource {
                                renderer::BindingHolder::Buffer(buffer) => {
                                    let binding = self.resources.get(&buffer).expect("Attempted to bind buffer with invalid handle.");
                                    bind_group_entries.push(wgpu::BindGroupEntry {
                                        binding: current_binding,
                                        resource: binding.as_entire_binding(),
                                    });
                                    current_binding += 1;
                                },
                                renderer::BindingHolder::Texture(view, sampler) => {
                                    let view = self.resources.get(&view).expect("Attempted to bind texture view with invalid handle.");
                                    bind_group_entries.push(wgpu::BindGroupEntry {
                                        binding: current_binding,
                                        resource: wgpu::BindingResource::TextureView(view),
                                    });
                                    current_binding += 1;
                                    
                                    let sampler = self.resources.get(&sampler).expect("Attempted to bind texture sampler with invalid handle.");
                                    bind_group_entries.push(wgpu::BindGroupEntry {
                                        binding: current_binding,
                                        resource: wgpu::BindingResource::Sampler(sampler),
                                    });
                                    current_binding += 1;
                                },
                            }
                        }
            
                        bind_group_entries
                    };

                    let bind_group = self.renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: None,
                        layout: &self.resources.get(&shader_handle).expect("Invalid shader handle.").bind_group_layout,
                        entries: bind_group_entries.as_slice(),
                    });

                    queue.push(renderer::QueuedRenderObject {
                        shader: shader_handle.clone(),
                        vertex_buffer: mesh.vertex_buffer.clone(),
                        index_buffer: mesh.index_buffer.clone(),
                        bind_group,
                        num_indices: mesh.num_elements,
                    });
                }
            }
        }

        queue
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let render_objects = self.extract();

        let output = self.renderer.window.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(CLEAR_COLOUR),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.renderer.window.depth_texture,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            {
                for render_object in &render_objects {
                    let shader = self.resources.get(&render_object.shader).unwrap();
                    let vertex_buffer = self.resources.get(&render_object.vertex_buffer).unwrap();
                    let index_buffer = self.resources.get(&render_object.index_buffer).unwrap();
                    
                    render_pass.set_pipeline(&shader.pipeline);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                    render_pass.set_bind_group(0, &render_object.bind_group, &[]);
                    
                    // for (index, bind_group) in render_object.bind_groups.iter().enumerate() {
                    //     let bind_group = self.resources.get(bind_group).unwrap();
                    //     render_pass.set_bind_group(index as u32, &*bind_group, &[]);
                    // }
                    
                    render_pass.draw_indexed(0..render_object.num_indices, 0, 0..1);
                }
            }
        }

        self.renderer.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
}

pub struct App {
    state: EngineState,
    event_loop: EventLoop<()>,
}

impl App {
    pub async fn new<F: FnMut(&renderer::Renderer, &mut resource::Resources) -> node::Node>(scene_builder: F) -> App {
        let _ = env_logger::try_init();

        let event_loop = EventLoop::new();
        
        let state = {
            let window = WindowBuilder::new().build(&event_loop).unwrap();
            EngineState::new(window, scene_builder).await
        };
        
        App {
            state,
            event_loop,
        }
    }
}

pub fn run(app: App) {
    let App { mut state, event_loop } = app;
    
    event_loop.run(move |event, _, control_flow| 
        if !state.input(&event) {
            match event {
                Event::WindowEvent { window_id, event } if window_id == state.renderer.window.window.id() => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(new_size) => {
                        state.renderer.on_resize(new_size);
                    },
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.renderer.on_resize(*new_inner_size);
                    },
                    WindowEvent::Focused(focused) => {
                        state.renderer.window.focused = focused;
                    }
                    _ => {}
                },
                Event::RedrawRequested(window_id) if window_id == state.renderer.window.window.id() => {
                    let mut current_time = std::time::Instant::now();
                    let delta_time = (current_time - state.frame_counter.last_frame_time).as_secs_f32();
                    let tick_delta_time = 1.0 / state.ticks_per_second as f32;
    
                    let tick_time = state.frame_counter.last_tick_time + std::time::Duration::from_secs_f32(tick_delta_time);
    
                    let window_size = glam::vec2(state.renderer.window.size.width as f32, state.renderer.window.size.height as f32);
    
                    let context = engine::UpdateContext {
                        window_size,
                        window_focused: state.renderer.window.focused,
                        delta_time,
                        keyboard: state.keyboard_manager.clone(),
                        mouse: state.mouse_manager.clone(),
                    };
                    
                    // state.first_update(&context);
                    state.pre_update(&context);
                    state.update(&context);
                    state.post_update(&context);
    
                    {
                        state.resources.get_mut(&state.keyboard_manager).expect("Lost handle to keyboard manager.").reset_input();
                        state.resources.get_mut(&state.mouse_manager).expect("Lost handle to mouse manager.").reset_input();
                    }
                    
                    // Almost at a tick, rendering a frame would be a waste, wait for tick
                    if (tick_time - current_time).as_secs_f32() < delta_time {
                        // log::trace!("Waiting for tick {:.2}ms {}", (tick_time - current_time).as_secs_f32() * 1000.0, state.frame_counter.current_frame);
                        while current_time < tick_time {
                            current_time = std::time::Instant::now();
                        }
                    }
    
                    if current_time >= tick_time {
                        let context = engine::UpdateContext {
                            window_size,
                            window_focused: state.renderer.window.focused,
                            delta_time: tick_delta_time,
                            keyboard: state.tick_keyboard_manager.clone(),
                            mouse: state.tick_mouse_manager.clone(),
                        };
                        
                        state.pre_tick(&context);
                        state.tick(&context);
                        state.post_tick(&context);
    
                        {
                            state.resources.get_mut(&state.tick_keyboard_manager).expect("Lost handle to keyboard manager.").reset_input();
                            state.resources.get_mut(&state.tick_mouse_manager).expect("Lost handle to mouse manager.").reset_input();
                        }
    
                        if state.frame_counter.current_tick % (state.ticks_per_second * 2) as usize == 0 {
                            log::debug!("MS: {:.2}, FPS: {:.2}, TPS (target {:.2}): {:.2}", delta_time * 1000.0, 1.0 / delta_time, 1.0 / tick_delta_time, 1.0 / (current_time - state.frame_counter.last_tick_time).as_secs_f32());
                        }
    
                        state.frame_counter.current_tick = state.frame_counter.current_tick.wrapping_add(1);
                        state.frame_counter.last_tick_time = tick_time;
                    }
                    
                    match state.render() {
                        Ok(_) => {},
                        // reconfigure if surface lost
                        Err(wgpu::SurfaceError::Lost) => state.renderer.reconfigure_surface(),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(_) => {},
                    }

                    if let Some(max_fps) = state.max_fps {
                        let next_frame = current_time + Duration::from_secs_f32(1.0 / max_fps as f32);
                        *control_flow = ControlFlow::WaitUntil(next_frame);
                    }
    
                    state.frame_counter.current_frame = state.frame_counter.current_frame.wrapping_add(1);
                    state.frame_counter.last_frame_time = current_time;
                },
                Event::MainEventsCleared => {
                    state.renderer.window.window.request_redraw();
                },
                // Event::NewEvents(cause) => {
                //     match cause {
                //         StartCause::ResumeTimeReached { .. } | StartCause::WaitCancelled { .. } => state.renderer.window.window.request_redraw(),
                //         _ => {},
                //     };
                // },
                _ => {}
            }
        }
    );
}
