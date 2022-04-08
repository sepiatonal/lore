use crate::mesh::{
    Mesh, Vertex,
};
use gltf;

// https://github.com/KhronosGroup/glTF/blob/main/specification/2.0/figures/gltfOverview-2.0.0b.png
// ^ infographic on the structure of a glTF file

pub fn load_gltf_mesh(path: &str) -> Vec<Mesh> {
    let (document, buffers, images) = match gltf::import(path) {
        Err(_) => {
            println!("Failed to load path {}", path);
            return Vec::new();
        },
        Ok(imported) => imported,
    };
    let mut gltf_meshes: Vec<gltf::Mesh> = Vec::new();
    for scene in document.scenes() {
        for node in scene.nodes() {
            gltf_meshes.append(&mut meshes_from_node(node));
        }
    }

    let mut meshes = Vec::<Mesh>::new();
    for gltf_mesh in gltf_meshes {
        for prim in gltf_mesh.primitives() {
            let mut vertices = Vec::<Vertex>::new();
            let mut indices = Vec::<u32>::new();
            
            let reader = prim.reader(|buffer| Some(&buffers[buffer.index()]));
            if let Some(iter) = reader.read_positions() {
                for vert_pos in iter {
                    vertices.push(Vertex {
                        position: vert_pos,
                        normal: [0.0, 0.0, 0.0],
                        tex_coords: [0.0, 0.0],
                    })
                }
            }
            if let Some(iter) = reader.read_normals() {
                for (i, norm) in iter.enumerate() {
                    vertices[i].normal = norm;
                }
            }
            if let Some(iter) = reader.read_tex_coords(0) { // i have no idea what that parameter does
                for (i, tx) in iter.into_f32().enumerate() {
                    vertices[i].tex_coords = tx;
                }
            }
            if let Some(iter) = reader.read_indices() {
                for ind in iter.into_u32() {
                    indices.push(ind);
                }
            }
            meshes.push(Mesh::new(vertices, indices));
        }
    }

    meshes
}

fn meshes_from_node(node: gltf::Node) -> Vec<gltf::Mesh> {
    let mut meshes: Vec<gltf::Mesh> = Vec::new();
    if node.children().count() != 0 {
        for child in node.children() {
            meshes.append(&mut meshes_from_node(child));
        }
    } else {
        match node.mesh() {
            Some(mesh) => meshes.push(mesh),
            None => {},
        }
    }

    meshes
}