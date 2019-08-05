use image::{DynamicImage, GenericImageView};
use lore_mesh::{Mesh, Vertex, Vector3, Vector2};

pub fn gen_section_from_heightmap(heightmap: DynamicImage, max_height: f32, cell_size: f32) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    // create vertices
    for x in 0..heightmap.width() {
        for z in 0..heightmap.height() {
            let vert_x = x as f32 * cell_size;
            let vert_z = z as f32 * cell_size;

            let pixel = heightmap.get_pixel(x, z);
            let image::Rgba(rgba) = pixel;
            let avg_brightness = (f32::from(rgba[0] + rgba[1] + rgba[2]) / 255.0) / 3.0;
            let vert_y = avg_brightness * max_height;

            let vert = Vertex {
                position: Vector3::new(vert_x, vert_z, vert_y),
                // TODO calculate this properly
                normal: Vector3::unit_y(),
                tex_coords: Vector2::new(x as f32, z as f32),
            };
            vertices.push(vert);
        }
    }
    // create indices
    for x in 0..heightmap.width() {
        for z in 0..heightmap.height() {
            
        }
    }
    Mesh::new(vertices, indices)
}
