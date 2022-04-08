use lore_render::{
    ObjectInstance,
    InputEvent,
};
use lore_render::{
    cgmath::{*, prelude::*},
    RenderingInstance,
    VirtualKeyCode, ElementState,
};

struct State {
    cube_a: (usize, usize),
    cube_b: (usize, usize),
}

pub fn main() {
    lore_render::run(
        setup,
        update,
        input,
    );
}

fn setup(rendering_instance: &mut RenderingInstance) -> State {
    let pl = rendering_instance.create_default_render_pipeline();

    let cube_mesh = {
        let mesh_data = lore_render::Mesh::from_gltf("assets/cube.gltf");
        rendering_instance.bind_mesh(&mesh_data, pl, None)
    };

    let cube_a = rendering_instance.create_object_instance(
        cube_mesh,
        ObjectInstance::from_position(-0.5, 0.0, 0.0),
    );
    let cube_b = rendering_instance.create_object_instance(
        cube_mesh,
        ObjectInstance::from_position(3.0, 0.0, 0.0)
            .with_angle(Vector3::unit_z(), 45.0),
    );

    State {
        cube_a, cube_b,
        
    }
}

fn update(rendering_instance: &mut RenderingInstance, state: &mut State) {

}

fn input(rendering_instance: &mut RenderingInstance, state: &mut State, input: InputEvent) {
    match input {
        InputEvent::Keyboard(key_input) => {
            if let Some(keycode) = key_input.virtual_keycode {
                if keycode == VirtualKeyCode::Space && key_input.state == ElementState::Pressed {
                    rendering_instance.modify_instance(state.cube_a, |inst| {
                        inst.rotation = inst.rotation * Quaternion::<f32>::from_axis_angle(Vector3::unit_y(), Deg(5.0));
                    });
                    rendering_instance.modify_instance(state.cube_b, |inst| {
                        inst.rotation = inst.rotation * Quaternion::<f32>::from_axis_angle(Vector3::unit_x(), Deg(3.0));
                    });
                }
            }
        },
        InputEvent::Mouse(button_state, button) => {

        },
    }
}