use image::{DynamicImage, GenericImageView};
use lore_mesh::{Mesh, Vector2, Vector3, Vertex};

pub fn gen_section_from_heightmap(
    heightmap: &DynamicImage,
    max_height: f32,
    cell_size: f32,
) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    // create vertices
    for x in 0..heightmap.width() {
        for z in 0..heightmap.height() {
            let vert_x = x as f32 * cell_size;
            let vert_z = z as f32 * cell_size;

            let pixel = heightmap.get_pixel(x, z);
            let image::Rgba(rgba) = pixel;
            // weird casts to avoid overflow
            let avg_brightness =
                ((rgba[0] as u32 + rgba[1] as u32 + rgba[2] as u32) as f32 / 255.0) / 3.0;
            let vert_y = avg_brightness * max_height;

            let uv_x = (x as f32) / heightmap.width() as f32;
            let uv_y = (z as f32) / heightmap.height() as f32;

            let vert = Vertex {
                position: Vector3::new(vert_x, vert_z, vert_y),
                // TODO calculate this properly
                normal: Vector3::unit_y(),
                tex_coords: Vector2::new(uv_x, uv_y),
            };
            vertices.push(vert);
        }
    }
    // create indices
    for x in 0..heightmap.width() - 1 {
        for z in 0..heightmap.height() - 1 {
            // treat the current vertex as the top-left vertex of the quad
            // first triangle
            indices.push(coords_to_inline(x, z, heightmap.width()));
            indices.push(coords_to_inline(x, z + 1, heightmap.width()));
            indices.push(coords_to_inline(x + 1, z + 1, heightmap.width()));
            // second triangle
            indices.push(coords_to_inline(x, z, heightmap.width()));
            indices.push(coords_to_inline(x + 1, z + 1, heightmap.width()));
            indices.push(coords_to_inline(x + 1, z, heightmap.width()));
        }
    }
    Mesh::new(vertices, indices)
}

fn coords_to_inline(x: u32, y: u32, width: u32) -> u32 {
    x * width + y
}
