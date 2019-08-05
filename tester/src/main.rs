use simple_logger;

use cgmath::{Vector3, Vector2, Matrix4, Point3};
use cgmath::prelude::*;

use lore_render::api::{Instance, MouseEvent};
use lore_render::mesh_loading::load;
use lore_render::{ElementState, VirtualKeyCode, MouseScrollDelta};

pub fn main() {
    simple_logger::init().unwrap();

    let (mut instance, key_input, mouse_input) = Instance::new("Test Window", 640.0, 480.0);
    let shader_program = instance.create_default_shader_program();

    let mut mesh = match load("/home/sour/mounted/big/blender/models/rpg/Anvil.dae") {
        Ok(mut mesh_list) => {
            mesh_list.remove(0)
        },
        Err(msg) => {
            panic!(msg);
        }
    };
    mesh.change_coord_system(Vector3::unit_z(), Vector3::unit_y());

    let mesh_loaded = instance.bind_mesh(&mesh, &shader_program);
    let rendered_object = instance.create_rendered_object(&mesh_loaded);

    let object_position = Vector3::<f32>::zero();
    instance.set_position(&rendered_object, object_position);

    let mut cam_radius = 10.0;
    let mut angle_percent: f32 = 0.0;
    let mut cam_height = 5.0;

    let mut last_mouse_pos = Vector2::<f64>::new(0.0, 0.0);
    while !instance.is_closed() {
        let loop_started = std::time::SystemTime::now();

        for m in mouse_input.try_iter() {
            match m {
                MouseEvent::Move(_, pos, _) => {
                    let pos_v = Vector2::new(pos.x, pos.y);
                    let dif = last_mouse_pos - pos_v;
                    angle_percent += dif.x as f32 * 0.01;
                    cam_height += dif.y as f32 * 0.05;
                    last_mouse_pos = pos_v;
                },
                MouseEvent::Wheel(_, delta, _, _) => {
                    match delta {
                        MouseScrollDelta::LineDelta(_, y) => {
                            cam_radius += y as f32 * 0.01;
                        },
                        MouseScrollDelta::PixelDelta(pos) => {
                            cam_radius += pos.y as f32 * 0.01;
                        },
                    }
                }
                _ => {},
            }
        }
        angle_percent %= 1.0;
        let angle_rad = angle_percent * 2.0 * std::f32::consts::PI;

        let cam_pos = Vector3::<f32>::new(angle_rad.cos() * cam_radius, cam_height, angle_rad.sin() * cam_radius);
        let cam_transform = Matrix4::<f32>::look_at(
            Point3::<f32>::from_vec(cam_pos),
            Point3::<f32>::from_vec(object_position),
            Vector3::<f32>::unit_y()
        );

        instance.set_camera_matrix(cam_transform);
        instance.set_position(&rendered_object, object_position);

        let current_time = std::time::SystemTime::now();
        let elapsed = current_time.duration_since(loop_started).expect("Error getting render loop duration");
        let elapsed_nanos = elapsed.as_nanos();
        let target_time_nanos = 1_000_000_000 / 60;
        if elapsed_nanos < target_time_nanos {
            let remaining_time_nanos = target_time_nanos - elapsed_nanos;
            let remaining_time = std::time::Duration::from_nanos(remaining_time_nanos as u64);
            std::thread::sleep(remaining_time);
        }
    }
}
