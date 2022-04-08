use crate::asset_loading::meshes::load_gltf_mesh;
use cgmath::{Vector3, Matrix4, Point3};
use cgmath::prelude::*;
use bytemuck::{
    Pod, Zeroable
};

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
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

    pub fn from_gltf(path: &str) -> Mesh {
        load_gltf_mesh(path).remove(0)
    }

    pub fn change_coord_system(&mut self, old_up: Vector3::<f32>, new_up: Vector3::<f32>) {
        let rot_mat = Matrix4::<f32>::look_at_rh(Point3::<f32>::new(0.0, 0.0, 0.0), Point3::<f32>::from_vec(new_up), old_up);
        for i in 0..self.vertices.len() {
            let pos = self.vertices[i].position;
            let pos_vec4 = Vector3::<f32>::new(pos[0], pos[1], pos[2]).extend(1.0);
            self.vertices[i].position = (rot_mat * pos_vec4).truncate().into();
        }
    }
}
