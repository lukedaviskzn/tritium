#[macro_use] extern crate maplit;

use std::collections::HashMap;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};

pub mod resource;
pub mod renderer;
pub mod input;
pub mod world;
pub mod util;
pub mod engine;
pub mod camera;

const CLEAR_COLOUR: wgpu::Color = wgpu::Color {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
struct VideoConfig {
    vsync: bool,
}

struct EngineState {
    renderer: renderer::Renderer,
    global_root: world::Node,
    keyboard_manager: resource::Handle<input::KeyboardManager>,
    tick_keyboard_manager: resource::Handle<input::KeyboardManager>,
    mouse_manager: resource::Handle<input::MouseManager>,
    tick_mouse_manager: resource::Handle<input::MouseManager>,
    frame_counter: engine::FrameCounter,
    resources: resource::Resources,
    ticks_per_second: u32,
}

impl EngineState {
    async fn new<F: FnMut(&renderer::Renderer, &mut resource::Resources) -> world::Node>(window: Window, mut scene_builder: F) -> EngineState {
        let video_config: VideoConfig = confy::load("wgpu-game-engine", Some("video")).unwrap();

        let renderer = renderer::Renderer::new(window, video_config.vsync).await;
        let mut resources = resource::Resources::new();

        log::info!("Building Scene");
        let current_scene = scene_builder(&renderer, &mut resources);
        log::info!("Scene Built");

        let global_root = world::NodeBuilder::new("global_root")
            // .add_script(camera::CameraUpdateScript)
            .add_script(world::TransformPropagationScript)
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
        
        #[derive(Debug)]
        struct MeshInput {
            vertex_buffer: resource::Handle<wgpu::Buffer>,
            index_buffer: resource::Handle<wgpu::Buffer>,
            material: Option<resource::Handle<wgpu::BindGroup>>,
            num_elements: u32,
        }

        #[derive(Debug)]
        struct ExtractedNode {
            shader: Option<resource::Handle<renderer::Shader>>,
            meshes: Vec<MeshInput>,
            bind_groups: HashMap<String, resource::Handle<wgpu::BindGroup>>,
        }
        
        let mut extracted_data = hashmap!{};

        {
            fn visit(node: &mut world::Node, resources: &mut resource::Resources, extracted_data: &mut HashMap<world::NodeId, ExtractedNode>, renderer: &renderer::Renderer) {
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
                    bind_groups: hashmap!{},
                };
                
                for input in &node_inputs {
                    match input {
                        renderer::RenderInput::Shader(shader) => node_data.shader = Some(shader.clone()),
                        renderer::RenderInput::Mesh {
                            vertex_buffer,
                            index_buffer,
                            material,
                            num_elements,
                        } => node_data.meshes.push(MeshInput {
                            vertex_buffer: vertex_buffer.clone(),
                            index_buffer: index_buffer.clone(),
                            material: if let Some(material) = material {
                                Some(material.clone())
                            } else {
                                None
                            },
                            num_elements: *num_elements,
                        }),
                        renderer::RenderInput::BindGroup(name, bind_group) => {
                            node_data.bind_groups.insert(name.clone(), bind_group.clone());
                        },
                    }
                }

                extracted_data.insert(node.id(), node_data);

                for child in &mut node.desc.children {
                    visit(child, resources, extracted_data, renderer);
                }
            }
    
            visit(&mut self.global_root, &mut self.resources, &mut extracted_data, &self.renderer);
        }

        let mut queue = vec![];

        // let current_camera = self.resources.get_global::<world::NodeId>("current_camera").expect("Global resource 'current_camera' must be set to a valid NodeId with an attached Camera bundle.");

        for (_, ExtractedNode { shader, meshes, bind_groups }) in &extracted_data {
            if meshes.len() > 0 {
                let shader_handle = shader.clone().expect("Failed to render mesh, no shader specified.");
                let shader = self.resources.get(&shader_handle).expect("Invalid shader handle.");

                for mesh in meshes {
                    // todo: a bit inefficient to recalculate order for each mesh
                    let mut ordered_bind_groups = vec![];

                    for input in &shader.inputs {
                        let bind_group = match input {
                            renderer::ShaderInput::MeshMaterial => {
                                mesh.material.as_ref().expect("Shader MeshMaterial input not present, mesh does not have material.").clone()
                            }
                            renderer::ShaderInput::Node { bind_group, .. } => {
                                let bind_group = bind_groups.get(bind_group).expect(&format!("Shader input '{bind_group}' not present in node."));
                                bind_group.clone()
                            },
                            renderer::ShaderInput::Global { resource, bind_group, .. } => {
                                let node_id = self.resources.get_global::<world::NodeId>(resource).expect(&format!("Failed to get global shader input '{resource}.{bind_group}'. No such global NodeId resource '{resource}' exists."));
                                let node_data = extracted_data.get(node_id).expect(&format!("Failed to get global shader input '{resource}.{bind_group}'. No Node exists with NodeId as specified in global resource '{resource}'."));
                                let bind_group = node_data.bind_groups.get(bind_group).expect(&format!("Failed to get global shader input '{resource}.{bind_group}'. The node at '{resource}' does not have bind group '{bind_group}'."));
                                bind_group.clone()
                            },
                        };
        
                        ordered_bind_groups.push(bind_group);
                    }



                    queue.push(renderer::QueuedRenderObject {
                        shader: shader_handle.clone(),
                        vertex_buffer: mesh.vertex_buffer.clone(),
                        index_buffer: mesh.index_buffer.clone(),
                        bind_groups: ordered_bind_groups.clone(),
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
                    view: &self.renderer.window.depth_texture.view,
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
                    
                    for (index, bind_group) in render_object.bind_groups.iter().enumerate() {
                        let bind_group = self.resources.get(bind_group).unwrap();
                        render_pass.set_bind_group(index as u32, &*bind_group, &[]);
                    }
                    
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
    pub async fn new<F: FnMut(&renderer::Renderer, &mut resource::Resources) -> world::Node>(scene_builder: F) -> App {
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
    
                    state.frame_counter.current_frame = state.frame_counter.current_frame.wrapping_add(1);
                    state.frame_counter.last_frame_time = current_time;
                },
                Event::MainEventsCleared => {
                    state.renderer.window.window.request_redraw();
                },
                _ => {}
            }
        }
    );
}
