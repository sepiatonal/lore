use cgmath::{Vector3, Vector2, Matrix4, Point3};
use cgmath::prelude::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coords: Vector2<f32>,
}

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Mesh {
        Mesh {
            vertices,
            indices,
        }
    }

    pub fn change_coord_system(&mut self, old_up: Vector3::<f32>, new_up: Vector3::<f32>) {
        let rot_mat = Matrix4::<f32>::look_at(Point3::<f32>::new(0.0, 0.0, 0.0), Point3::<f32>::from_vec(new_up), old_up);
        for i in 0..self.vertices.len() {
            let pos_vec4 = self.vertices[i].position.extend(1.0);
            self.vertices[i].position = (rot_mat * pos_vec4).truncate();
        }
    }
}
