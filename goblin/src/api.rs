use std::thread;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{Sender, Receiver, channel};

use image::DynamicImage;

use glutin::{
    ContextBuilder,
    event::{Event, WindowEvent, KeyboardInput, ElementState, MouseButton, ModifiersState, MouseScrollDelta, TouchPhase, DeviceId},
    window::{WindowBuilder},
    event_loop::{EventLoop, ControlFlow},
    dpi::LogicalPosition,
};

use cgmath::{Vector3, Matrix4};

use lore_mesh::{Mesh};
use crate::rendering::rendering::*;
use crate::rendering::data_updates::*;

// TODO consider a helper function for single-use meshes (that is, meshes that will only have one instance) that combines BindMesh and CreateRenderedObject

pub enum MouseEvent {
    Button(DeviceId, ElementState, MouseButton, ModifiersState),
    Wheel(DeviceId, MouseScrollDelta, TouchPhase, ModifiersState),
    Move(DeviceId, LogicalPosition, ModifiersState),
}

pub struct Instance {
    data_update_channel: Sender<Box<dyn DataUpdate>>,
    // becomes true when render thread closes
    end_switch: Arc<RwLock<bool>>,
    rendered_object_tickets: Vec<RenderedObjectTicket>,
    loaded_mesh_tickets: Vec<LoadedMeshTicket>,
    shader_tickets: Vec<ShaderTicket>,
    shader_program_tickets: Vec<ShaderProgramTicket>,
    texture_tickets: Vec<TextureTicket>,
}

impl Instance {
    pub fn new(title: &'static str, width: f64, height: f64) -> (Instance, Receiver<KeyboardInput>, Receiver<MouseEvent>) {
        let (data_update_sender, data_update_receiver) = channel();
        let (keyboard_input_sender, keyboard_input_receiver) = channel();
        let (mouse_input_sender, mouse_input_receiver) = channel();

        let mut drawing_instance = DrawingInstance::new();
        let end_switch = Arc::new(RwLock::new(false));
        let end_switch_for_render_thread = end_switch.clone();

        let ret = Instance {
            rendered_object_tickets: Vec::new(),
            loaded_mesh_tickets: Vec::new(),
            data_update_channel: data_update_sender,
            shader_tickets: Vec::new(),
            shader_program_tickets: Vec::new(),
            texture_tickets: Vec::new(),
            end_switch,
        };

        let _ = thread::spawn(move || {
            let events_loop = EventLoop::new();
            let window_builder = WindowBuilder::new()
                .with_title(title)
                .with_inner_size(glutin::dpi::LogicalSize::new(width, height));
            let context_wrapped = ContextBuilder::new()
                .build_windowed(window_builder, &events_loop)
                .expect("Trouble creating opengl context");
            let windowed_context = unsafe { context_wrapped.make_current() }.expect("Trouble making opengl context current");

            // TODO make this togglable and default to visible
            windowed_context.window().set_cursor_visible(false);

            gl::load_with(|s| windowed_context.get_proc_address(s) as *const _);

            drawing_instance.setup_rendering();

            // TODO modifiable fps
            let fps = 60;
            events_loop.run(move |event, _, control_flow| {
                let loop_started = std::time::SystemTime::now();

                match event {
                    Event::LoopDestroyed => {
                        let switch = end_switch_for_render_thread.write();
                        *switch.expect("Error getting close switch") = true;
                    },
                    Event::WindowEvent { ref event, .. } => match event {
                        WindowEvent::Resized(logical_size) => {
                            let dpi_factor = windowed_context.window().hidpi_factor();
                            let size = logical_size.to_physical(dpi_factor);
                            windowed_context.resize(size);
                            let w = size.width as i32;
                            let h = size.height as i32;
                            unsafe {
                                gl::Viewport(0, 0, w, h);
                            }
                        }
                        WindowEvent::RedrawRequested => {
                            for du in data_update_receiver.try_iter() {
                                du.apply(&mut drawing_instance);
                            }

                            drawing_instance.draw();
                            windowed_context.swap_buffers().unwrap();
                        }
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        },
                        WindowEvent::KeyboardInput { input: key, .. } => {
                            keyboard_input_sender.send(*key).expect("Error sending keyboard input");
                        },
                        WindowEvent::CursorMoved { device_id: d_id, position: pos, modifiers: modif } => {
                            mouse_input_sender.send(MouseEvent::Move(*d_id, *pos, *modif)).expect("Error sending mouse input");
                        },
                        WindowEvent::MouseWheel { device_id: d_id, delta: delt, phase: ph, modifiers: modif } => {
                            mouse_input_sender.send(MouseEvent::Wheel(*d_id, *delt, *ph, *modif)).expect("Error sending mouse input");
                        },
                        _ => (),
                    },
                    Event::EventsCleared => {
                        windowed_context.window().request_redraw();
                    },
                    _ => {},
                }

                let current_time = std::time::SystemTime::now();
                let elapsed = current_time.duration_since(loop_started).expect("Error getting render loop duration");
                let elapsed_nanos = elapsed.as_nanos();
                let target_time_nanos = 1_000_000_000 / fps;
                if elapsed_nanos < target_time_nanos {
                    let remaining_time_nanos = target_time_nanos - elapsed_nanos;
                    let remaining_time = std::time::Duration::from_nanos(remaining_time_nanos as u64);
                    let end_instant = std::time::Instant::now().checked_add(remaining_time).expect("Error getting render loop end instant");
                    *control_flow = ControlFlow::WaitUntil(end_instant);
                } else {
                    *control_flow = ControlFlow::Wait;
                }
            });
        });

        (ret, keyboard_input_receiver, mouse_input_receiver)
    }

    pub fn bind_mesh(&mut self, mesh: &Mesh, shader_program_ticket: &ShaderProgramTicket) -> LoadedMeshTicket {
        self.loaded_mesh_tickets.push(
            LoadedMeshTicket::new()
        );
        let lmt = self.loaded_mesh_tickets.last().unwrap().clone();
        let du = MeshCreate {
            ticket: lmt.clone(),
            mesh: mesh.clone(),
            shader_program_ticket: shader_program_ticket.clone(),
        };
        self.data_update_channel.send(
            Box::new(du)
        ).expect("Error sending data update");
        lmt
    }

    pub fn create_rendered_object(&mut self, lmt: &LoadedMeshTicket) -> RenderedObjectTicket {
        self.rendered_object_tickets.push(
            RenderedObjectTicket::new()
        );
        let rot = self.rendered_object_tickets.last().unwrap().clone();
        let du = RenderedObjectCreate {
            ticket: rot.clone(),
            mesh_ticket: lmt.clone(),
        };
        self.data_update_channel.send(
            Box::new(du)
        ).expect("Error sending data update");
        rot
    }

    pub fn set_position(&mut self, rot: &RenderedObjectTicket, position: Vector3<f32>) {
        let du = PositionSet {
            ticket: rot.clone(),
            position: position,
        };
        self.data_update_channel.send(
            Box::new(du)
        ).expect("Error sending data update");
    }

    pub fn set_matrix(&mut self, rot: &RenderedObjectTicket, matrix: Matrix4<f32>) {
        let du = MatrixSet {
            ticket: rot.clone(),
            matrix: matrix,
        };
        self.data_update_channel.send(
            Box::new(du)
        ).expect("Error sending data update");
    }

    pub fn create_default_shader_program(&mut self) -> ShaderProgramTicket {
        let v_src = r#"
            #version 330 core

            layout(location = 0) in vec3 v_position;
            layout(location = 1) in vec3 v_normal;
            layout(location = 2) in vec2 v_uv;

            uniform mat4 projection;
            uniform mat4 view;
            uniform mat4 model;

            out vec3 normal;
            out vec2 tex_coord;

            void main() {
                gl_Position = projection * view * model * vec4(v_position, 1.0);
                normal = v_normal;
                tex_coord = v_uv;
            }
        "#;
        let f_src = r#"
            #version 330 core

            in vec3 normal;
            in vec2 tex_coord;

            uniform sampler2D albedo;

            out vec4 color;

            void main() {
                vec4 object_color = texture(albedo, tex_coord);
                vec3 light_dir = normalize(vec3(0.5, -1.0, 0.5));
                float ambient_light = 1.00;
                // float light_intensity = max(dot(normal, light_dir), 0.0) + ambient_light;
                float light_intensity = ambient_light;
                color = object_color * light_intensity;
            }
        "#;
        self.shader_tickets.push(
            ShaderTicket::new()
        );
        let vert_t = self.shader_tickets.last().unwrap().clone();
        let vert_t_2 = vert_t.clone();
        self.shader_tickets.push(
            ShaderTicket::new()
        );
        let frag_t = self.shader_tickets.last().unwrap().clone();
        let frag_t_2 = frag_t.clone();
        self.shader_program_tickets.push(
            ShaderProgramTicket::new()
        );
        let sp_t = self.shader_program_tickets.last().unwrap().clone();
        let ret_sp_t = sp_t.clone();

        let vs_du = VertexShaderCreate {
            ticket: vert_t,
            source: v_src,
        };
        let fs_du = FragmentShaderCreate {
            ticket: frag_t,
            source: f_src,
        };
        self.data_update_channel.send(
            Box::new(vs_du)
        ).expect("Error sending data update");
        self.data_update_channel.send(
            Box::new(fs_du)
        ).expect("Error sending data update");

        let sp_du = ShaderProgramCreate {
            ticket: sp_t,
            vert_shader_ticket: vert_t_2,
            frag_shader_ticket: Some(frag_t_2),
        };
        self.data_update_channel.send(
            Box::new(sp_du)
        ).expect("Error sending data update");

        ret_sp_t
    }

    pub fn set_camera_matrix(&mut self, cam_mat: Matrix4<f32>) {
        let du = CameraMatrixSet {
            matrix: cam_mat,
        };
        self.data_update_channel.send(
            Box::new(du)
        ).expect("Error sending data update");
    }

    pub fn bind_texture(&mut self, texture: &DynamicImage) -> TextureTicket {
        self.texture_tickets.push(
            TextureTicket::new()
        );
        let tt = self.texture_tickets.last().unwrap().clone();
        let du = TextureCreate {
            ticket: tt.clone(),
            image: texture.clone(),
        };
        self.data_update_channel.send(
            Box::new(du)
        ).expect("Error sending data update");
        tt
    }

    pub fn set_texture(&mut self, rendered_object: &RenderedObjectTicket, texture: &TextureTicket) {
        let du = TextureSet {
            ticket: rendered_object.clone(),
            texture_ticket: texture.clone(),
        };
        self.data_update_channel.send(
            Box::new(du)
        ).expect("Error sending data update");
    }

    pub fn is_closed(&self) -> bool {
        let switch = self.end_switch.read().expect("end switch poisoned");
        *switch
    }
}
