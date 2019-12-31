use cgmath::prelude::*;
use cgmath::{Deg, Matrix4, Vector2, Vector3};

use lore_render::api::{Instance, MouseEvent};
use lore_render::mesh_loading::load;
use lore_render::{ElementState, Mesh, VirtualKeyCode};

use second_day::mesh_gen::gen_section_from_heightmap;
use second_day::noise::PerlinNoise2DGenerator;
use second_day::{DynamicImage, GenericImageView};

pub fn main() {
    let (mut instance, key_input, mouse_input) = Instance::new("Test Window", 640.0, 480.0);
    let shader_program = instance.create_default_shader_program();

    let (terrain_mesh, heightmap_img) = create_terrain_mesh();
    let mesh_loaded = instance.bind_mesh(&terrain_mesh, &shader_program);
    let rendered_object = instance.create_rendered_object(&mesh_loaded);
    let heightmap_tex = instance.bind_texture(&heightmap_img);
    instance.set_texture(&rendered_object, &heightmap_tex);

    let anvil_mesh = load_anvil_mesh();
    let loaded_anvil_mesh = instance.bind_mesh(&anvil_mesh, &shader_program);
    let rendered_anvil_mesh = instance.create_rendered_object(&loaded_anvil_mesh);
    instance.set_position(&rendered_anvil_mesh, Vector3::new(0.0, 2.0, 0.0));

    let object_position = Vector3::<f32>::zero();
    instance.set_position(&rendered_object, object_position);

    let mut cam_matrix = cgmath::Transform::one();
    let cam_speed = 0.05;
    let mut last_mouse_pos = Vector2::<f64>::new(0.0, 0.0);
    while !instance.is_closed() {
        let loop_started = std::time::SystemTime::now();

        for m in mouse_input.try_iter() {
            if let MouseEvent::Move(_, pos, _) = m {
                let pos_v = Vector2::new(pos.x, pos.y);
                let dif = last_mouse_pos - pos_v;
                last_mouse_pos = pos_v;
                cam_matrix = Matrix4::from_angle_y(Deg(-dif.x as f32))
                    * Matrix4::from_angle_x(Deg(-dif.y as f32))
                    * cam_matrix;
            }
        }

        let mut delta_pos = Vector3::<f32>::zero();
        for k in key_input.try_iter() {
            if k.state == ElementState::Released {
                continue;
            }
            if let Some(vk) = k.virtual_keycode {
                match vk {
                    VirtualKeyCode::Q => {
                        delta_pos += Vector3::unit_y() * cam_speed;
                    }
                    VirtualKeyCode::E => {
                        delta_pos -= Vector3::unit_y() * cam_speed;
                    }
                    VirtualKeyCode::W => {
                        delta_pos += Vector3::unit_z() * cam_speed;
                    }
                    VirtualKeyCode::S => {
                        delta_pos -= Vector3::unit_z() * cam_speed;
                    }
                    VirtualKeyCode::A => {
                        delta_pos += Vector3::unit_x() * cam_speed;
                    }
                    VirtualKeyCode::D => {
                        delta_pos -= Vector3::unit_x() * cam_speed;
                    }
                    _ => {}
                }
                cam_matrix = Matrix4::from_translation(delta_pos) * cam_matrix;
            }
        }

        instance.set_camera_matrix(cam_matrix);

        let current_time = std::time::SystemTime::now();
        let elapsed = current_time
            .duration_since(loop_started)
            .expect("Error getting render loop duration");
        let elapsed_nanos = elapsed.as_nanos();
        let target_time_nanos = 1_000_000_000 / 60;
        if elapsed_nanos < target_time_nanos {
            let remaining_time_nanos = target_time_nanos - elapsed_nanos;
            let remaining_time = std::time::Duration::from_nanos(remaining_time_nanos as u64);
            std::thread::sleep(remaining_time);
        }
    }

    fn load_anvil_mesh() -> Mesh {
        let mut mesh = match load("/home/sour/mounted/big/blender/models/rpg/Anvil.dae") {
            Ok(mut mesh_list) => mesh_list.remove(0),
            Err(msg) => {
                panic!(msg);
            }
        };
        mesh.change_coord_system(Vector3::unit_z(), Vector3::unit_y());
        mesh
    }

    fn create_terrain_mesh() -> (Mesh, DynamicImage) {
        let noise_gen = PerlinNoise2DGenerator::new(2_312_503_425);
        let heightmap = noise_gen.image(0, 0, 256, 256, 0.1);
        /*for x in 0..256 {
            println!();
            for y in 0..256 {
                let second_day::Rgba(p) = heightmap.get_pixel(x, y);
                print!("{:?} ", p[0]);
            }
        }*/
        (gen_section_from_heightmap(&heightmap, 25.0, 0.5), heightmap)
    }
}
