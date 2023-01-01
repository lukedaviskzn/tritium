use std::io::BufReader;

use image::GenericImageView;
use tritium::{renderer::Shader, resource::{self, Model, CubeMap, Texture, Material, Handle}, node::{Node, ClosureScript}, camera::Camera, engine::Rgba, components::{Transform, PointLight, DirectionalLight, AmbientLight}};
use winit::event::VirtualKeyCode;

#[tokio::main]
async fn main() {
    let app = tritium::App::new(|renderer, resources| {

        // Pipelines

        let main_render_pipeline = Shader::from_resource(
            &renderer,
            "pipelines/main.ron",
        ).unwrap();
        let main_render_pipeline = resources.store(main_render_pipeline);

        // let emissive_render_pipeline = Shader::from_resource(
        //     &renderer,
        //     "pipelines/emissive.ron",
        // ).unwrap();
        // let emissive_render_pipeline = resources.store(emissive_render_pipeline);

        let skybox_render_pipeline = Shader::from_resource(
            &renderer,
            "pipelines/skybox.ron",
        ).unwrap();
        let skybox_render_pipeline = resources.store(skybox_render_pipeline);

        // Resources

        // let model = {
        //     let model = resource::load_obj(
        //         renderer,
        //         resources,
        //         "res/sponza/sponza.obj",
        //         // "res/cube.obj",
        //     ).unwrap();
            
        //     // let material = Material::new(renderer, resources, "plane_material", None, Rgba::WHITE, None, 1.0);
        //     // let material = resources.store(material);
            
        //     // let model = Model::new_plane(renderer, resources, Some(material));
            
        //     resources.store(model)
        // };

        // let scene_node = {
        //     let mut node = resource::load_gltf(renderer, resources, "res/sponza_gltf/Sponza.gltf", Some(0)).unwrap();
            
        //     node.traverse_if_mut(&mut |node| node.has_component::<Handle<Model>>(), &mut |node| {
        //         node.add_component(main_render_pipeline.clone());
        //     });
            
        //     node
        // };

        let scene_node = {
            let albedo_image = image::load(BufReader::new(std::fs::File::open("res/materials/dirt-red-bricks/dirty-red-bricks_albedo.png").unwrap()), image::ImageFormat::Png).unwrap();
            let normal_image = image::load(BufReader::new(std::fs::File::open("res/materials/dirt-red-bricks/dirty-red-bricks_normal-ogl.png").unwrap()), image::ImageFormat::Png).unwrap();
            let metallic_image = image::load(BufReader::new(std::fs::File::open("res/materials/dirt-red-bricks/dirty-red-bricks_metallic.png").unwrap()), image::ImageFormat::Png).unwrap();
            let roughness_image = image::load(BufReader::new(std::fs::File::open("res/materials/dirt-red-bricks/dirty-red-bricks_roughness.png").unwrap()), image::ImageFormat::Png).unwrap();
            let occlusion_image = image::load(BufReader::new(std::fs::File::open("res/materials/dirt-red-bricks/dirty-red-bricks_ao.png").unwrap()), image::ImageFormat::Png).unwrap();

            let (metallic_roughness_bytes, size) = {
                let size = metallic_image.dimensions();
                let metallic_image = metallic_image.to_rgba8();
                let roughness_image = roughness_image.to_rgba8();
                
                let metallic_bytes = metallic_image.as_raw();
                let roughness_bytes = roughness_image.as_raw();
                let mut bytes = vec![0; metallic_bytes.len()];
                for i in 0..(metallic_bytes.len() / 4) {
                    let index = i * 4;
                    bytes[index + 1] = roughness_bytes[index + 1];
                    bytes[index + 2] = metallic_bytes[index + 2];
                    bytes[index + 3] = 255;
                }
                (bytes, size)
            };

            log::trace!("{:?} {:?}", metallic_roughness_bytes.len(), size);

            let metallic_roughness_image = image::DynamicImage::ImageRgba8(image::RgbaImage::from_raw(size.0, size.1, metallic_roughness_bytes).unwrap());

            let albedo_texture = Texture::from_image(renderer, resources, &albedo_image, None, false);
            let normal_texture = Texture::from_image(renderer, resources, &normal_image, None, false);
            let metallic_roughness_texture = Texture::from_image(renderer, resources, &metallic_roughness_image, None, false);
            let occlusion_texture = Texture::from_image(renderer, resources, &occlusion_image, None, false);

            let albedo_texture = resources.store(albedo_texture);
            let normal_texture = resources.store(normal_texture);
            let metallic_roughness_texture = resources.store(metallic_roughness_texture);
            let occlusion_texture = resources.store(occlusion_texture);

            let material = Material::builder()
                .albedo_texture(albedo_texture)
                .normal_texture(normal_texture)
                .metallic_roughness_texture(metallic_roughness_texture)
                .occlusion_texture(occlusion_texture)
                .build(renderer, resources);
            let material = resources.store(material);
            
            let model = Model::new_sphere(renderer, resources, Some(material), 12);
            let model = resources.store(model);

            Node::builder("Model Node")
            .add_component(Transform::IDENTITY)
            .add_component(model)
            .add_component(main_render_pipeline.clone())
            .build()
        };
        
        let light_model = {
            let mut model = resource::load_obj(
                 renderer,
                resources,
                "res/sphere.obj",
            ).unwrap();

            // let material = Material::new(renderer, resources, Some("light_material"), false, resource::AlphaMode::Mask { cutoff: 0.1 }, None, Rgba::BLACK, None, 1.0, 0.5, None, 1.0, None, 1.0, None, Rgba::new(1.0, 0.9, 0.8, 1.0));
            let material = Material::builder()
                .name("light_material")
                .albedo(Rgba::BLACK)
                .emissive_factor(Rgba::new(1.0, 0.9, 0.8, 1.0))
                .build(renderer, resources);
            let material = resources.store(material);

            model.meshes[0].material = Some(material);
            
            // let material = Material::new(renderer, resources, "light_material", None, Rgba::WHITE, None, 1.0);
            // let material = resources.store(material);
            // model.meshes[0].material = Some(material);
            
            // let model = Model::new_cube(renderer, resources, Some(material));
            
            resources.store(model)
        };
        
        let skybox_model = {
            let model = Model::new_inverted_cube(renderer, resources, None);
            resources.store(model)
        };
        
        log::trace!("Loading Skybox");
        let skybox = {
            // From face images
            // let pos_x = &image::load(BufReader::new(std::fs::File::open("res/skyboxes/grand_canyon/px.png").unwrap()), image::ImageFormat::Png).unwrap();
            // let neg_x = &image::load(BufReader::new(std::fs::File::open("res/skyboxes/grand_canyon/nx.png").unwrap()), image::ImageFormat::Png).unwrap();
            // let pos_y = &image::load(BufReader::new(std::fs::File::open("res/skyboxes/grand_canyon/py.png").unwrap()), image::ImageFormat::Png).unwrap();
            // let neg_y = &image::load(BufReader::new(std::fs::File::open("res/skyboxes/grand_canyon/ny.png").unwrap()), image::ImageFormat::Png).unwrap();
            // let pos_z = &image::load(BufReader::new(std::fs::File::open("res/skyboxes/grand_canyon/pz.png").unwrap()), image::ImageFormat::Png).unwrap();
            // let neg_z = &image::load(BufReader::new(std::fs::File::open("res/skyboxes/grand_canyon/nz.png").unwrap()), image::ImageFormat::Png).unwrap();
            
            // let source = Cube {
            //     pos_x,
            //     neg_x,
            //     pos_y,
            //     neg_y,
            //     pos_z,
            //     neg_z,
            // };

            // let cubemap = CubeMap::from_images(renderer, resources, source, None, false).unwrap();

            // From equirectangular jpeg
            // let image = image::load(BufReader::new(std::fs::File::open("res/skyboxes/milkyway/Milkyway_BG.jpg").unwrap()), image::ImageFormat::Jpeg).unwrap();
            // let cubemap = CubeMap::from_equirectangular(renderer, resources, &image, None, true).unwrap();
            
            // From equirectangular hdr
            let decoder = image::codecs::hdr::HdrDecoder::new(BufReader::new(std::fs::File::open("res/skyboxes/grand_canyon/equirectangular.hdr").unwrap())).unwrap();
            let width = decoder.metadata().width;
            let height = decoder.metadata().height;
            let pixels = decoder.read_image_hdr().unwrap().iter().flat_map(|p| p.0).collect::<Vec<_>>();
            let image = image::DynamicImage::ImageRgb32F(image::Rgb32FImage::from_vec(width, height, pixels).unwrap());

            let cubemap = CubeMap::from_equirectangular(renderer, resources, &image, None, true).unwrap();
            
            resources.store(cubemap)
        };
        log::trace!("Skybox Loaded");

        // Scene

        // let point_light_group = LightGroup {
        //     lights: vec![
        //         PointLight::new(Rgba::new(0.0, 0.0, 1.0, 3.0)),
        //         PointLight::new(Rgba::new(0.0, 1.0, 0.0, 3.0)),
        //     ],
        // };

        // let light_origin_script = ClosureScript::builder()
        //     .tick(|node, context, _| {
        //         let transform = node.get_component_mut::<Transform>().unwrap();
        //         transform.rotation *= glam::Quat::from_axis_angle(glam::Vec3::Y, -0.5 * context.delta_time);
        //     }).build();

        let camera_script = ClosureScript::builder()
            .tick(|node, context, resources| {
                let keyboard = resources.get(&context.keyboard).unwrap();

                let speed = 5.0;
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

        let current_scene = Node::builder("Current Scene")
            // .add_child(
            //     Node::builder("model")
            //     .add_component(Transform::from_scale(glam::Vec3::splat(0.01)))
            //     // .add_component(Transform::IDENTITY)
            //     .add_component(model.clone())
            //     .add_component(main_render_pipeline.clone())
            //     .build()
            // )
            .add_child(scene_node)
            .add_child(
                Node::builder("sun")
                .add_component(Transform::from_rotation(glam::Quat::from_axis_angle(glam::vec3(-2.0, 0.0, -1.0).normalize(), -std::f32::consts::PI / 2.9)))
                .add_component(DirectionalLight::new(Rgba::new(1.0, 1.0, 1.0, 20.0)))
                .add_component(AmbientLight::new(Rgba::new(1.0, 1.0, 1.0, 0.25)))
                .build()
            )
            .add_child(
                // Node::builder("light_origin")
                // .add_component(Transform::IDENTITY)
                // .add_script(light_origin_script)
                // .add_child(
                    Node::builder("light")
                    .add_component(Transform::from_tranlation_scale(glam::vec3(0.0, 2.0, 0.0), glam::Vec3::splat(0.25)))
                    // .add_component(Transform::from_tranlation_scale(glam::vec3(2.0, 0.1, 0.0), glam::Vec3::splat(0.25)))
                    .add_component(PointLight::new(Rgba::new(1.0, 0.8, 0.6, 10.0)))
                    .add_child(
                        Node::builder("light_model")
                        .add_component(Transform::IDENTITY)
                        .add_component(light_model.clone())
                        // .add_component(render_pipelines[1].clone())
                        .add_component(main_render_pipeline.clone())
                        .build()
                    )
                    .build()
                // )
                // .build()
            )
            .add_child(
                Node::builder("camera")
                .add_component(Transform::from_tranlation(glam::vec3(0.0, 0.0, 3.0)))
                .add_component(Camera::Perspective {
                    fovy: std::f32::consts::FRAC_PI_3,
                    znear: 0.1,
                    zfar: Some(10000.0),
                })
                .add_child(
                    Node::builder("cam_light")
                    .add_component(Transform::IDENTITY)
                    // .add_component(PointLight::new(Rgba::new(1.0, 1.0, 1.0, 0.5)))
                    .build()
                )
                .add_script(camera_script)
                .build()
            )
            .add_child(
                Node::builder("skybox")
                .add_component(skybox_model.clone())
                .add_component(skybox.clone())
                // .add_component(render_pipelines[2].clone())
                .add_component(skybox_render_pipeline.clone())
                .build()
            )
            .build();

        resources.set_global("current_camera", current_scene.find_by_name("camera").unwrap().id());
        resources.set_global("current_light", current_scene.find_by_name("light").unwrap().id());

        current_scene
    }).await;
    tritium::run(app);
}
