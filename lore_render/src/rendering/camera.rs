/**
 * a `Camera` is an abstract Camera. it cannot be used directly for rendering.
 * `Camera` might be useful for math, or for having multiple cameras that you
 * swap between.
 * 
 * a `RenderableCamera` is a camera with buffers and bindings that can actually
 * be used for rendering. typically, only one instance of RenderableCamera should
 * exist.
 */

use cgmath;
use wgpu::util::DeviceExt;

pub(in crate::rendering) struct Camera {
    pub pos: cgmath::Point3<f32>, // camera position
    pub target: cgmath::Point3<f32>, // point the camera looks at
    pub up: cgmath::Vector3<f32>, // "up" vector for deciding camera roll
    aspect: f32,
    fovy: f32, // degrees
    znear: f32,
    zfar: f32,
}

pub(in crate::rendering) struct RenderableCamera {
    pub camera: Camera,
    uniform: CameraUniform,
    #[allow(dead_code)] // buffer must be stored since bind_group references it
    // TODO make sure this actually has to be stored
    buffer: wgpu::Buffer,
    // TODO only one bind_group_layout will ever exist per program,
    // so bind_group_layout should actually go in RenderingInstance
    // instead of here. even if there are two cameras, they should
    // have the same bind_group_layout.
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            pos: (0.0, 0.0, -10.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: (0.0, 1.0, 0.0).into(),
            aspect: 1.0, // TODO aspect is wrong, should by window width/height
            fovy: 60.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
    
    pub fn matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.pos, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        proj * view
    }
}

impl RenderableCamera {
    pub fn new(device: &wgpu::Device) -> Self {
        let camera = Camera::new();
        let uniform = CameraUniform {
            matrix: camera.matrix().into(),
        };
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("camera_buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        Self {
            camera,
            uniform,
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn update(&mut self, queue: &mut wgpu::Queue) {
        self.uniform.matrix = self.camera.matrix().into();
        // TODO this is not maximum efficiency. look into map_write_async
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(in crate::rendering) struct CameraUniform {
    matrix: [[f32; 4]; 4],
}