use std::{io::BufReader, path::Path};

use tritium::{renderer::Shader, resource::{self, Model, CubeMap, Material, Sampler, CubeSampler}, node::{Node, ClosureScript}, camera::Camera, engine::Rgba, components::{Transform, DirectionalLight, AmbientLight}};
use winit::event::VirtualKeyCode;

#[tokio::main]
async fn main() {
    let app = tritium::App::new(|renderer, resources| {

        // Pipelines

        let main_render_pipeline = Shader::from_resource(
            renderer,
            "pipelines/main.ron",
        ).unwrap();
        let main_render_pipeline = resources.store(main_render_pipeline);

        let skybox_render_pipeline = Shader::from_resource(
            renderer,
            "pipelines/skybox.ron",
        ).unwrap();
        let skybox_render_pipeline = resources.store(skybox_render_pipeline);

        // Resources

        let scene_node = {
            let paths = [
                Path::new("res/materials/antique_veneer1_bl/"),
                Path::new("res/materials/chipping-painted-wall-bl/"),
                Path::new("res/materials/dirt-red-bricks-bl/"),
                Path::new("res/materials/gray-granite-flecks-bl/"),
                Path::new("res/materials/painted-metal-shed-bl/"),
                // Path::new("res/materials/rustediron1-alt2-bl/"),
                // Path::new("res/materials/sharp-boulder-layered-bl/"),
                Path::new("res/materials/soft-blanket-bl/"),
                Path::new("res/materials/mirror/"),
            ];

            let mut scene = Node::builder("Scene");
            
            for (i, path) in paths.into_iter().enumerate() {
                let material = {
                    let albedo_texture = resource::load_texture(renderer, resources, path.join("albedo.png"), false).unwrap();
                    let albedo_sampler = Sampler::new_default(renderer, resources, albedo_texture);
                    let albedo_sampler = resources.store(albedo_sampler);
                    
                    let normal_texture = resource::load_texture(renderer, resources, path.join("normal.png"), false).unwrap();
                    let normal_sampler = Sampler::new_default(renderer, resources, normal_texture);
                    let normal_sampler = resources.store(normal_sampler);
                    
                    let metallic_texture = resource::load_texture(renderer, resources, path.join("metallic.png"), false).unwrap();
                    let metallic_sampler = Sampler::new_default(renderer, resources, metallic_texture);
                    let metallic_sampler = resources.store(metallic_sampler);
                    
                    let roughness_texture = resource::load_texture(renderer, resources, path.join("roughness.png"), false).unwrap();
                    let roughness_sampler = Sampler::new_default(renderer, resources, roughness_texture);
                    let roughness_sampler = resources.store(roughness_sampler);
                    
                    let occlusion_texture = resource::load_texture(renderer, resources, path.join("occlusion.png"), false).unwrap();
                    let occlusion_sampler = Sampler::new_default(renderer, resources, occlusion_texture);
                    let occlusion_sampler = resources.store(occlusion_sampler);
                    
                    let material = Material::builder()
                        .albedo_sampler(albedo_sampler)
                        .normal_sampler(normal_sampler)
                        .metallic_sampler(metallic_sampler)
                        .roughness_sampler(roughness_sampler)
                        .occlusion_sampler(occlusion_sampler)
                        .build(renderer, resources);
                    
                    resources.store(material)
                };
                
                let model = Model::new_sphere(renderer, resources, Some(material), 255, resource::SphereUV::Equirectangular2X);
                let model = resources.store(model);
    
                scene = scene.add_child(
                    Node::builder(&path.file_name().unwrap().to_string_lossy())
                    .add_component(Transform::from_translation_scale(glam::vec3(i as f32 - 2.0, 0.0, 0.0), glam::Vec3::splat(0.5)))
                    .add_component(model)
                    .add_component(main_render_pipeline.clone())
                    .build()
                );
            }

            scene.build()

            // let albedo_texture = resource::load_texture(renderer, resources, "res/earth/albedo.jpg", true).unwrap();
            // let normal_texture = resource::load_texture(renderer, resources, "res/earth/normal.tif", true).unwrap();
            // let roughness_texture = resource::load_texture(renderer, resources, "res/earth/roughness.tif", true).unwrap();

            // let material = Material::builder()
            //     .albedo_texture(albedo_texture)
            //     .normal_texture(normal_texture)
            //     .roughness_texture(roughness_texture)
            //     .metallic_factor(0.0)
            //     .build(renderer, resources);
            // let material = resources.store(material);
            
            // let model = Model::new_sphere(renderer, resources, Some(material), 64, resource::SphereUV::Equirectangular);
            // let model = resources.store(model);

            // let scene = Node::builder("Scene")
            //     .add_component(Transform::from_scale(glam::Vec3::splat(10.0)))
            //     .add_component(model)
            //     .add_component(main_render_pipeline.clone())
            //     .build();
            
            // scene
        };

        // let scene_node = {
        //     // let mut node = resource::load_gltf(renderer, resources, "res/WaterBottle.glb", None).unwrap();
        //     // let mut node = resource::load_gltf(renderer, resources, "res/tests/EmissiveStrengthTest.glb", None).unwrap();
            
        //     // let mut test = resource::load_gltf(renderer, resources, "res/tests/AlphaBlendModeTest.glb", None).unwrap();
        //     // let mut test = resource::load_gltf(renderer, resources, "res/tests/MetalRoughSpheres.glb", None).unwrap();
        //     // let mut test = resource::load_gltf(renderer, resources, "res/tests/MultiUVTest.glb", None).unwrap();
        //     // let mut test = resource::load_gltf(renderer, resources, "res/tests/NormalTangentMirrorTest.glb", None).unwrap();
        //     // let mut test = resource::load_gltf(renderer, resources, "res/tests/NormalTangentTest.glb", None).unwrap();
        //     // let mut test = resource::load_gltf(renderer, resources, "res/tests/TextureCoordinateTest.glb", None).unwrap();
        //     // let mut test = resource::load_gltf(renderer, resources, "res/tests/TextureEncodingTest.glb", None).unwrap();
        //     let mut test = resource::load_gltf(renderer, resources, "res/tests/TextureLinearInterpolationTest.glb", None).unwrap();
        //     // let mut test = resource::load_gltf(renderer, resources, "res/tests/TextureSettingsTest.glb", None).unwrap();
            
        //     test.traverse_if_mut(&mut |node| node.has_component::<Handle<Model>>(), &mut |node| {
        //         node.add_component(main_render_pipeline.clone());
        //     });
        //     test
        // };
        
        let skybox_model = {
            let model = Model::new_inverted_cube(renderer, resources, None);
            resources.store(model)
        };
        
        log::trace!("Loading Skybox");
        // let skybox = {
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
            let image = image::load(BufReader::new(std::fs::File::open("res/skyboxes/milkyway/Milkyway_BG.jpg").unwrap()), image::ImageFormat::Jpeg).unwrap();
            let cubemap = CubeMap::from_equirectangular(renderer, resources, &image, None, true).unwrap();
            let cubemap = resources.store(cubemap);
            let cubemap = CubeSampler::new_default(renderer, resources, cubemap);
            
            // From equirectangular hdr
            // let decoder = image::codecs::hdr::HdrDecoder::new(BufReader::new(std::fs::File::open("res/skyboxes/grand_canyon/equirectangular.hdr").unwrap())).unwrap();
            // let width = decoder.metadata().width;
            // let height = decoder.metadata().height;
            // let pixels = decoder.read_image_hdr().unwrap().iter().flat_map(|p| p.0).collect::<Vec<_>>();
            // let image = image::DynamicImage::ImageRgb32F(image::Rgb32FImage::from_vec(width, height, pixels).unwrap());

            // let cubemap = CubeMap::from_equirectangular(renderer, resources, &image, None, true).unwrap();
            
            // resources.store(cubemap)
            resources.set_global("tritium::skybox", cubemap);
            
            let decoder = image::codecs::hdr::HdrDecoder::new(BufReader::new(std::fs::File::open("res/skyboxes/milkyway/Milkyway_Light.hdr").unwrap())).unwrap();
            let width = decoder.metadata().width;
            let height = decoder.metadata().height;
            let pixels = decoder.read_image_hdr().unwrap().iter().flat_map(|p| p.0).collect::<Vec<_>>();
            let image = image::DynamicImage::ImageRgb32F(image::Rgb32FImage::from_vec(width, height, pixels).unwrap());

            let cubemap = CubeMap::from_equirectangular(renderer, resources, &image, None, true).unwrap();
            let cubemap = resources.store(cubemap);
            let cubemap = CubeSampler::new_default(renderer, resources, cubemap);

            resources.set_global("tritium::irradiance", cubemap);
            
            let decoder = image::codecs::hdr::HdrDecoder::new(BufReader::new(std::fs::File::open("res/skyboxes/milkyway/Milkyway_small.hdr").unwrap())).unwrap();
            let width = decoder.metadata().width;
            let height = decoder.metadata().height;
            let pixels = decoder.read_image_hdr().unwrap().iter().flat_map(|p| p.0).collect::<Vec<_>>();
            let image = image::DynamicImage::ImageRgb32F(image::Rgb32FImage::from_vec(width, height, pixels).unwrap());

            let cubemap = CubeMap::from_equirectangular(renderer, resources, &image, None, true).unwrap();
            let cubemap = resources.store(cubemap);
            let cubemap = CubeSampler::new_default(renderer, resources, cubemap);

            resources.set_global("tritium::reflections", cubemap);
        // };
        log::trace!("Skybox Loaded");

        // Scene

        // let light_origin_script = ClosureScript::builder()
        //     .tick(|node, context, _| {
        //         let transform = node.get_component_mut::<Transform>().unwrap();
        //         transform.rotation *= glam::Quat::from_axis_angle(glam::Vec3::Y, -0.5 * context.delta_time);
        //     }).build();

        let camera_script = ClosureScript::builder()
            .tick(|node, context, resources| {
                let keyboard = context.keyboard.get(resources);

                let speed = if keyboard.key_pressed(VirtualKeyCode::LShift) {
                    2.0
                } else {
                    0.5
                };
                
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

                let mouse = context.mouse.get(resources);

                let motion = -mouse.motion() * angular_speed * context.delta_time;
                
                let (x, y, _) = transform.rotation.to_euler(glam::EulerRot::YXZ); // discard roll
                transform.rotation = glam::Quat::from_euler(glam::EulerRot::YXZ, motion.x + x, motion.y + y, 0.0);

                let velocity = transform.rotation * direction * speed * context.delta_time;
                transform.translation += velocity;

            }).build();

        let current_scene = Node::builder("Current Scene")
            .add_child(scene_node)
            .add_child(
                Node::builder("sun")
                .add_component(Transform::from_rotation(glam::Quat::from_axis_angle(glam::vec3(-2.0, 0.0, -1.0).normalize(), -std::f32::consts::PI / 2.9)))
                .add_component(DirectionalLight::new(Rgba::new(1.0, 1.0, 1.0, 10.0)))
                .add_component(AmbientLight::new(Rgba::new(1.0, 1.0, 1.0, 0.25)))
                .build()
            )
            .add_child(
                Node::builder("camera")
                .add_component(Transform::from_translation(glam::vec3(0.0, 0.0, 3.0)))
                .add_component(Camera::Perspective {
                    fovy: std::f32::consts::FRAC_PI_3,
                    znear: 0.1,
                    zfar: Some(10000.0),
                })
                .add_script(camera_script)
                .build()
            )
            .add_child(
                Node::builder("skybox")
                .add_component(skybox_model.clone())
                // .add_component(skybox.clone())
                .add_component(skybox_render_pipeline.clone())
                .build()
            )
            .build();

        resources.set_global("current_camera", current_scene.find_by_name("camera").unwrap().id());

        current_scene
    }).await;
    tritium::run(app);
}
