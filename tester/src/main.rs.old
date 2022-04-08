use cgmath::prelude::*;
use cgmath::{Deg, Matrix4, Vector2, Vector3};

use lore::input::InputManager;
use lore::lore_mesh::mesh_loading::load;
use lore::lore_mesh::Mesh;
use lore::lore_render::VirtualKeyCode;
use lore::second_day::mesh_gen::gen_section_from_heightmap;
use lore::second_day::noise::PerlinNoise2DGenerator;
use lore::second_day::{DynamicImage, GenericImageView};

pub fn main() {
    let (mut renderer, key_input, mouse_input) =
        lore::lore_render::api::Instance::new("Test Window", 640.0, 480.0);
    let shader_program = renderer.create_default_shader_program();

    // create and render terrain mesh
    /*
    let (terrain_mesh, heightmap_img) = create_terrain_mesh();
    let mesh_loaded = renderer.bind_mesh(&terrain_mesh, &shader_program);
    let rendered_object = renderer.create_rendered_object(&mesh_loaded);
    let heightmap_tex = renderer.bind_texture(&heightmap_img);

    let object_position = Vector3::<f32>::zero();
    renderer.set_position(&rendered_object, object_position);
    renderer.set_texture(&rendered_object, &heightmap_tex);
    */

    let anvil_mesh = load_anvil_mesh();
    let loaded_anvil_mesh = renderer.bind_mesh(&anvil_mesh, &shader_program);
    let rendered_anvil_mesh = renderer.create_rendered_object(&loaded_anvil_mesh);
    renderer.set_position(&rendered_anvil_mesh, Vector3::new(0.0, 2.0, 0.0));

    let mut input = InputManager::new(key_input, mouse_input);
    input.set_key("Forward", VirtualKeyCode::W);
    input.set_key("Backward", VirtualKeyCode::S);
    input.set_key("Left", VirtualKeyCode::A);
    input.set_key("Right", VirtualKeyCode::D);

    let mut cam_matrix = cgmath::Transform::one();
    let cam_speed = 0.05;

    while !renderer.is_closed() {
        let loop_started = std::time::SystemTime::now();

        input.update();

        // mouse input
        let (dmx, dmy) = input.get_mouse_delta();
        cam_matrix = Matrix4::from_angle_y(Deg(dmx as f32))
            * Matrix4::from_angle_x(Deg(dmy as f32))
            * cam_matrix;

        // key input
        let mut delta_pos = Vector3::<f32>::zero();
        if input.is_key_down("Forward") {
            delta_pos += Vector3::unit_z() * cam_speed;
        }
        if input.is_key_down("Backward") {
            delta_pos -= Vector3::unit_z() * cam_speed;
        }
        if input.is_key_down("Left") {
            delta_pos += Vector3::unit_x() * cam_speed;
        }
        if input.is_key_down("Right") {
            delta_pos -= Vector3::unit_x() * cam_speed;
        }
        cam_matrix = Matrix4::from_translation(delta_pos) * cam_matrix;

        // send matrix to renderer now that rotation and position have been updated
        renderer.set_camera_matrix(cam_matrix);

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
