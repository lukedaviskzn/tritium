use std::io::BufReader;

use game_engine::{renderer::{Shader, PositionVertex}, resource::{self, Model, Mesh, CubeMap, Cube}, engine::{ClosureScript, Rgba}, world::{Transform, NodeBuilder, Light}, camera::Camera};
use winit::event::VirtualKeyCode;

#[tokio::main]
async fn main() {
    let app = game_engine::App::new(|renderer, resources| {
        // Pipelines

        let render_pipelines = {
            let main_render_pipeline = Shader::from_resource(
                &renderer,
                "res/pipelines/main.ron",
            ).unwrap();
            let main_render_pipeline = resources.store(main_render_pipeline);
    
            let emissive_render_pipeline = Shader::from_resource(
                &renderer,
                "res/pipelines/emissive.ron",
            ).unwrap();
            let emissive_render_pipeline = resources.store(emissive_render_pipeline);
    
            let skybox_render_pipeline = Shader::from_resource(
                &renderer,
                "res/pipelines/skybox.ron",
            ).unwrap();
            let skybox_render_pipeline = resources.store(skybox_render_pipeline);

            vec![main_render_pipeline, emissive_render_pipeline, skybox_render_pipeline]
        };

        // Resources
        
        let cube_model = {
            let model = resource::load_obj(
                 renderer,
                resources,
                "res/sphere.obj",
            ).unwrap();

            resources.store(model)
        };

        let obj_model = {
            let model = resource::load_obj(
                renderer,
                resources,
                "res/sphere.obj",
            ).unwrap();

            resources.store(model)
        };
        
        let skybox_model = {
            let model = Model {
                meshes: vec![Mesh::new(renderer, resources, "skybox", vec![
                    // 0 1
                    // 3 2
                    PositionVertex { position: [-1.0,  1.0, -1.0] },
                    PositionVertex { position: [ 1.0,  1.0, -1.0] },
                    PositionVertex { position: [ 1.0, -1.0, -1.0] },
                    PositionVertex { position: [-1.0, -1.0, -1.0] },
                    // 4 5
                    // 7 6
                    PositionVertex { position: [-1.0,  1.0,  1.0] },
                    PositionVertex { position: [ 1.0,  1.0,  1.0] },
                    PositionVertex { position: [ 1.0, -1.0,  1.0] },
                    PositionVertex { position: [-1.0, -1.0,  1.0] },
                ], vec![
                    // back face
                    0, 2, 1,
                    0, 3, 2,
                    // front face
                    4, 5, 6,
                    4, 6, 7,
                    // left face
                    4, 3, 0,
                    4, 7, 3,
                    // right face
                    1, 6, 5,
                    1, 2, 6,
                    // top face
                    5, 4, 0,
                    5, 0, 1,
                    // bottom face
                    3, 7, 6,
                    3, 6, 2,
                ], None)],
            };
            
            resources.store(model)
        };
        
        log::trace!("Loading Skybox");
        let skybox = {
            let pos_x = &image::load(BufReader::new(std::fs::File::open("res/skyboxes/grand_canyon/px.png").unwrap()), image::ImageFormat::Png).unwrap();
            let neg_x = &image::load(BufReader::new(std::fs::File::open("res/skyboxes/grand_canyon/nx.png").unwrap()), image::ImageFormat::Png).unwrap();
            let pos_y = &image::load(BufReader::new(std::fs::File::open("res/skyboxes/grand_canyon/py.png").unwrap()), image::ImageFormat::Png).unwrap();
            let neg_y = &image::load(BufReader::new(std::fs::File::open("res/skyboxes/grand_canyon/ny.png").unwrap()), image::ImageFormat::Png).unwrap();
            let pos_z = &image::load(BufReader::new(std::fs::File::open("res/skyboxes/grand_canyon/pz.png").unwrap()), image::ImageFormat::Png).unwrap();
            let neg_z = &image::load(BufReader::new(std::fs::File::open("res/skyboxes/grand_canyon/nz.png").unwrap()), image::ImageFormat::Png).unwrap();
            
            let source = Cube {
                pos_x,
                neg_x,
                pos_y,
                neg_y,
                pos_z,
                neg_z,
            };

            let cubemap = CubeMap::from_image(renderer, resources, source, None, false).unwrap();
            
            resources.store(cubemap)
        };
        log::trace!("Skybox Loaded");

        // Scene

        let model_script = ClosureScript::builder()
            .tick(|node, context, _| {
                let transform = node.get_component_mut::<Transform>().unwrap();
                transform.rotation *= glam::Quat::from_axis_angle(glam::Vec3::Z, 0.05 * context.delta_time);
            }).build();

        let light_origin_script = ClosureScript::builder()
            .tick(|node, context, _| {
                let transform = node.get_component_mut::<Transform>().unwrap();
                transform.rotation *= glam::Quat::from_axis_angle(glam::Vec3::Z, -0.1 * context.delta_time);
            }).build();

        let camera_script = ClosureScript::builder()
            .tick(|node, context, resources| {
                let keyboard = resources.get(&context.keyboard).unwrap();

                let speed = 1.0;
                let angular_speed = 0.1;

                let mut direction = glam::Vec3::ZERO;

                if keyboard.key_pressed(VirtualKeyCode::W) {
                    direction += glam::Vec3::NEG_Z;
                }

                if keyboard.key_pressed(VirtualKeyCode::S) {
                    direction += glam::Vec3::Z;
                }

                if keyboard.key_pressed(VirtualKeyCode::A) {
                    direction += glam::Vec3::NEG_X;
                }

                if keyboard.key_pressed(VirtualKeyCode::D) {
                    direction += glam::Vec3::X;
                }

                if keyboard.key_pressed(VirtualKeyCode::Space) {
                    direction += glam::Vec3::Y;
                }

                if keyboard.key_pressed(VirtualKeyCode::LControl) {
                    direction += glam::Vec3::NEG_Y;
                }

                if direction != glam::Vec3::ZERO {
                    direction = direction.normalize()
                }

                let transform = node.get_component_mut::<Transform>().unwrap();

                let mouse = resources.get(&context.mouse).unwrap();

                let motion = -mouse.motion() * angular_speed * context.delta_time;
                
                let (x, y, _) = transform.rotation.to_euler(glam::EulerRot::YXZ); // discard roll
                transform.rotation = glam::Quat::from_euler(glam::EulerRot::YXZ, motion.x + x, motion.y + y, 0.0);

                let velocity = transform.rotation * direction * speed * context.delta_time;
                transform.translation += velocity;

            }).build();

        let current_scene = NodeBuilder::new("Current Scene")
            .add_child(
                NodeBuilder::new("model")
                .add_component(Transform::IDENTITY)
                .add_component(obj_model.clone())
                .add_component(render_pipelines[0].clone())
                .add_script(model_script)
                .build()
            )
            .add_child(
                NodeBuilder::new("light_origin")
                .add_component(Transform::IDENTITY)
                .add_script(light_origin_script)
                .add_child(
                    NodeBuilder::new("light")
                    .add_component(Transform::from_tranlation_scale(glam::vec3(2.0, 0.0, 0.0), glam::Vec3::splat(0.1)))
                    .add_component(Light::new(Rgba::new(2.0, 0.5, 0.5, 1.0)))
                    .add_child(
                        NodeBuilder::new("light_model")
                        .add_component(Transform::IDENTITY)
                        .add_component(cube_model.clone())
                        .add_component(render_pipelines[1].clone())
                        .build()
                    )
                    .build()
                )
                .build()
            )
            .add_child(
                NodeBuilder::new("camera")
                .add_component(Transform::from_tranlation(glam::vec3(0.0, 0.0, 3.0)))
                .add_component(Camera {
                    fovy: std::f32::consts::FRAC_PI_3,
                    znear: 0.01,
                    zfar: 10000.0,
                })
                .add_script(camera_script)
                .build()
            )
            .add_child(
                NodeBuilder::new("skybox")
                // .add_component(Transform::from_scale(glam::Vec3::splat(5.0)))
                .add_component(Transform::IDENTITY)
                .add_component(skybox_model.clone())
                .add_component(skybox.clone())
                .add_component(render_pipelines[2].clone())
                .build()
            )
            .build();

        resources.set_global("current_camera", current_scene.find_by_name("camera").unwrap().id());
        resources.set_global("current_light", current_scene.find_by_name("light").unwrap().id());

        current_scene
    }).await;
    game_engine::run(app);
}
