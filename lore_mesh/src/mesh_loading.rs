// TODO refactor collada_loader with fresh eyes
use std::path::Path;

use collada;
use collada::document::ColladaDocument;
use collada::PrimitiveElement::{Triangles, Polylist};
use collada::Object;

use cgmath::{Vector3, Vector2};

use crate::mesh::{Mesh, Vertex};

pub fn load(path: &str) -> Result<Vec<Mesh>, &'static str> {
    if path.ends_with(".dae") {
        convert_collada(path)
    } else {
        Err("Filetype not supported")
    }
}

// assumes that all vertices in verts have normal values of [0, 0, 0]
// Note: piston_collada gives us face normals, not vertex normals
// We get the sum of face normals bordering each vertex, then average them to get vertex normals
fn calculate_normals(verts: &mut Vec<Vertex>, o: &collada::Object, tris: &collada::Triangles) {
    // every time a vertex is indicated, add the associated face normal
    for (a, b, c) in tris.vertices.iter() {
        for (v, _, norm) in [a, b, c].iter() {
            let v = *v;
            if let Some(n) = norm {
                let n = *n;
                verts[v].normal[0] += o.normals[n].x as f32;
                verts[v].normal[1] += o.normals[n].y as f32;
                verts[v].normal[2] += o.normals[n].z as f32;
            };
        }
    }
    // normalize all normals
    for v in verts {
        let (x, y, z) = (v.normal[0], v.normal[1], v.normal[2]);
        let mag = (x * x + y * y + z * z).sqrt();
        v.normal[0] = x / mag;
        v.normal[1] = y / mag;
        v.normal[2] = z / mag;
    }
}

// will overwrite verts[all].tex_coords
fn populate_tex_coords_and_get_indices(verts: &mut Vec<Vertex>, o: &Object, tris: &collada::Triangles) -> Vec<u32> {
    // to be returned
    let mut indices: Vec<u32> = Vec::new();
    // this is a list of bools that tell us if we've yet populated tex_coords to each vertex
    // false means the vertex is not populated, true means it has been populated
    let mut verts_checklist = vec![false; verts.len()];
    for (a, b, c) in tris.vertices.iter() {
        for (v, tex, _) in [a, b, c].iter() {
            let v = *v;
            if let Some(t) = tex {
                let t = *t;
                // we need a new vertex if the indicated vertex already has a tex_coord which is different from the new tex_coord
                let txn = Vector2::new(o.tex_vertices[t].x as f32, o.tex_vertices[t].y as f32);
                let txo = verts[v].tex_coords;
                let need_new_vertex = verts_checklist[v] && txo != txn;
                // the index of the vertex being handled. This might be vert, or might be the index of a new vertex
                let index = if need_new_vertex {
                    verts.len()
                } else {
                    v
                };
                if need_new_vertex {
                    verts.push(verts[v]);
                    // have to make sure these stay the same length
                    verts_checklist.push(false);
                }
                verts[index].tex_coords = Vector2::new(o.tex_vertices[t].x as f32, o.tex_vertices[t].y as f32);
                verts_checklist[index] = true;

                indices.push(index as u32);
            } else {
                indices.push(v as u32);
            }
        }
    }
    indices
}

// Some notes:
// The normals given per-vert by piston_collada are actually face normals, so we have to calculate the vert normals
// Some vertices have multiple different texture coords (different for each face) and some do not.
// We need to duplicate the ones that have different texture coords, but we'd like to not duplicate the ones that don't have different texture coords
fn convert_collada(path: &str) -> Result<Vec<Mesh>, &'static str> {
    let doc_maybe = ColladaDocument::from_path(
        Path::new(path)
    );
    let doc = match doc_maybe {
        Ok(d) => d,
        Err(_) => {
            return Err("Error loading file");
        },
    };

    let objects = match doc.get_obj_set() {
        Some(o) => o,
        None => {
            return Err("File contained no objects");
        },
    };
    // this is the final list we plan to return
    let mut abstract_meshes = Vec::new();
    // o is the data, m is indexes referencing the data
    for o in objects.objects.iter() {
        for g in o.geometry.iter() {
            for m in g.mesh.iter() {
                // here we convert the vertices to vertices of our format, from piston_collada's format
                let mut verts = Vec::new();
                for v in o.vertices.iter() {
                    verts.push(
                        // we leave normal and tex_coords as 0 for now, to be filled in later
                        Vertex {
                            position: Vector3::new(v.x as f32, v.y as f32, v.z as f32),
                            normal: Vector3::new(0.0, 0.0, 0.0),
                            tex_coords: Vector2::new(0.0, 0.0),
                        }
                    );
                }

                let tris = match m {
                    Polylist(_) => panic!("Polylists are not supported"),
                    Triangles(t) => t,
                };
                calculate_normals(&mut verts, &o, &tris);
                let indices = populate_tex_coords_and_get_indices(&mut verts, &o, &tris);

                abstract_meshes.push(
                    Mesh {
                        vertices: verts,
                        indices,
                    }
                );
            }
        }
    }

    if abstract_meshes.is_empty() {
        Err("File contained no meshes")
    } else {
        Ok(abstract_meshes)
    }
}
