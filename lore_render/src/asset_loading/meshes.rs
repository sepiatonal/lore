use crate::mesh::{
    Mesh, Vertex,
};
use gltf;

// https://github.com/KhronosGroup/glTF/blob/main/specification/2.0/figures/gltfOverview-2.0.0b.png
// ^ infographic on the structure of a glTF file

// returns:
// Vec of meshes and the index (into returned textures list) of the texture that goes to that mesh
// Vec of textures
// Vec of object instances and the index (into returned meshes list) of the mesh that the instance is of
pub fn load_gltf(path: &str) -> (Vec<(Mesh, Option<usize>)>, Vec<image::DynamicImage>, Vec<(crate::ObjectInstance, usize)>) {
    let (document, buffers, gltf_images) = match gltf::import(path) {
        Err(_) => {
            println!("Failed to load path {}", path);
            return (Vec::new(), Vec::new(), Vec::new());
        },
        Ok(imported) => imported,
    };

    let mut meshes = Vec::new();
    for gltf_mesh in document.meshes() {
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
            let texture_mayb = prim.material().pbr_metallic_roughness().base_color_texture();
            let texture_id = match texture_mayb {
                Some(texture) => Some(texture.texture().index()),
                None => None,
            };

            meshes.push((Mesh::new(vertices, indices), texture_id));
        }
    }

    let mut images: Vec<image::DynamicImage> = Vec::new();
    for gltf_image in gltf_images {
        if gltf_image.format != gltf::image::Format::R8G8B8A8 {
            panic!("i don't like the image format. {:?}", gltf_image.format);
        };
        let img_mayb = image::ImageBuffer::from_raw(gltf_image.width, gltf_image.height, gltf_image.pixels);
        if let Some(img) = img_mayb {
            images.push(image::DynamicImage::ImageRgba8(img));
        }
    }

    let mut instances = Vec::new();
    for scene in document.scenes() {
        for node in scene.nodes() {
            instances.append(&mut objects_from_node(node));
        }
    }

    (meshes, images, instances)
}

fn objects_from_node(node: gltf::Node) -> Vec<(crate::ObjectInstance, usize)> {
    let mut instances = Vec::new();
    if node.children().count() != 0 {
        for child in node.children() {
            instances.append(&mut objects_from_node(child));
        }
    } else {
        if let gltf::scene::Transform::Decomposed{ translation, rotation, scale, } = node.transform() {
            if let Some(gltf_mesh) = node.mesh() {
                let instance = crate::ObjectInstance {
                    position: translation.into(),
                    rotation: rotation.into(),
                };

                instances.push((instance, gltf_mesh.index()));
            }
        }
    }

    instances
}