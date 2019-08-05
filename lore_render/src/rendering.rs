use std::ffi::CString;
use std::ptr;
use std::str;
use std::mem;
use std::sync::{Arc, RwLock};

use core::ffi::c_void;

use memoffset::offset_of;

use slab::Slab;

use gl::types::*;

use cgmath::{Matrix4, Vector3, Deg};
use cgmath::prelude::*;

use image::{DynamicImage, GenericImageView};

use crate::mesh::{Mesh, Vertex};
use crate::image_loading::default_texture;

pub(crate) enum DataUpdate {
    MeshCreate(LoadedMeshTicket, Mesh, ShaderProgramTicket),
    RenderedObjectCreate(RenderedObjectTicket, LoadedMeshTicket),
    PositionSet(RenderedObjectTicket, Vector3<f32>),
    MatrixSet(RenderedObjectTicket, Matrix4<f32>),
    VertShaderCreate(ShaderTicket, &'static str),
    FragShaderCreate(ShaderTicket, &'static str),
    // program ticket (empty, to be filled), vert shader, Option<frag shader>
    ShaderProgramCreate(ShaderProgramTicket, ShaderTicket, Option<ShaderTicket>),
    CameraSet(Matrix4<f32>),
    TextureCreate(TextureTicket, DynamicImage),
}

struct LoadedMesh {
    vao_id: GLuint,
    vbo_id: GLuint,
    ebo_id: GLuint,
    indices_len: i32,
    shader_program: usize,
}

impl LoadedMesh {
    fn new(vao_id: GLuint, vbo_id: GLuint, ebo_id: GLuint, indices_len: i32, shader_program: usize) -> LoadedMesh {
        LoadedMesh {
            vao_id,
            vbo_id,
            ebo_id,
            indices_len,
            shader_program,
        }
    }
}

struct RenderedObject {
    mesh: usize,
    matrix: Matrix4<f32>,
    texture: usize,
}

impl RenderedObject {
    fn new(mesh: usize) -> RenderedObject {
        RenderedObject {
            mesh,
            matrix: Matrix4::identity(),
            // probably the default texture since that was the first thing inserted in loaded_textures and will never be removed
            texture: 0,
        }
    }
}

#[derive(Clone)]
pub struct RenderedObjectTicket {
    id: Arc<RwLock<Option<usize>>>,
}

impl RenderedObjectTicket {
    pub(crate) fn new() -> RenderedObjectTicket {
        RenderedObjectTicket {
            id: Arc::new(RwLock::new(None)),
        }
    }
}


#[derive(Clone)]
pub struct ShaderTicket {
    id: Arc<RwLock<Option<usize>>>,
}

impl ShaderTicket {
    pub(crate) fn new() -> ShaderTicket {
        ShaderTicket {
            id: Arc::new(RwLock::new(None)),
        }
    }
}

#[derive(Clone)]
pub struct ShaderProgramTicket {
    id: Arc<RwLock<Option<usize>>>,
}

impl ShaderProgramTicket {
    pub(crate) fn new() -> ShaderProgramTicket {
        ShaderProgramTicket {
            id: Arc::new(RwLock::new(None)),
        }
    }
}

#[derive(Clone)]
pub struct LoadedMeshTicket {
    id: Arc<RwLock<Option<usize>>>,
}

impl LoadedMeshTicket {
    pub(crate) fn new() -> LoadedMeshTicket {
        LoadedMeshTicket {
            id: Arc::new(RwLock::new(None))
        }
    }
}

#[derive(Clone)]
pub struct TextureTicket {
    id: Arc<RwLock<Option<usize>>>,
}

impl TextureTicket {
    pub(crate) fn new() -> TextureTicket {
        TextureTicket {
            id: Arc::new(RwLock::new(None))
        }
    }
}

pub(crate) struct DrawingInstance {
    shaders: Slab<GLuint>,
    shader_programs: Slab<GLuint>,
    rendered_objects: Slab<RenderedObject>,
    loaded_meshes: Slab<LoadedMesh>,
    loaded_textures: Slab<GLuint>,
    camera: Matrix4<f32>,
}

impl DrawingInstance {
    pub(crate) fn new() -> DrawingInstance {
        DrawingInstance {
            shaders: Slab::new(),
            shader_programs: Slab::new(),
            loaded_meshes: Slab::new(),
            rendered_objects: Slab::new(),
            loaded_textures: Slab::new(),
            camera: Matrix4::<f32>::identity(),
        }
    }

    fn create_shader(&mut self, shader_src: &str, ty: GLenum) -> usize {
        unsafe {
            let shader = gl::CreateShader(ty);
            let c_src = CString::new(shader_src.as_bytes()).expect("Error creating C string for shader source");
            gl::ShaderSource(shader, 1, &c_src.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            let mut status = 0 as GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
            // if error, panic
            if status != (1 as GLint) {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1);
                gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
                panic!("{}", str::from_utf8(&buf).expect("Shader error not utf8"));
            }

            self.shaders.insert(
                shader
            )
        }
    }

    fn create_vert_shader(&mut self, shader_src: &str) -> usize {
        self.create_shader(shader_src, gl::VERTEX_SHADER)
    }

    fn create_frag_shader(&mut self, shader_src: &str) -> usize {
        self.create_shader(shader_src, gl::FRAGMENT_SHADER)
    }

    fn create_shader_program(&mut self, vert: GLuint, frag: Option<GLuint>) -> usize {
        unsafe {
            let program = gl::CreateProgram();
            gl::AttachShader(program, vert);
            if let Some(f) = frag {
                gl::AttachShader(program, f);
            }
            gl::LinkProgram(program);

            let mut status = 0 as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
            // if error, panic
            if status != (1 as GLint) {
                let mut len = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1);
                gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
                panic!("{}", str::from_utf8(&buf).expect("Shader error not utf8"));
            }

            self.shader_programs.insert(
                program
            )
        }
    }

    fn bind_mesh(&mut self, mesh: &Mesh, shader_program: usize) -> usize {
        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            let vert_size = mem::size_of::<Vertex>();
            let verts_len = (mesh.vertices.len() * vert_size) as isize;
            let verts_start = &mesh.vertices[0] as *const Vertex as *const c_void;
            gl::BufferData(gl::ARRAY_BUFFER, verts_len, verts_start, gl::STATIC_DRAW);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            let ind_size = mem::size_of::<u32>();
            let inds_len = (mesh.indices.len() * ind_size) as isize;
            let inds_start = &mesh.indices[0] as *const u32 as *const c_void;
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, inds_len, inds_start, gl::STATIC_DRAW);

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, vert_size as i32, offset_of!(Vertex, position) as *const c_void);
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::TRUE, vert_size as i32, offset_of!(Vertex, normal) as *const c_void);
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, vert_size as i32, offset_of!(Vertex, tex_coords) as *const c_void);
        }

        self.loaded_meshes.insert(
            LoadedMesh::new(vao, vbo, ebo, mesh.indices.len() as i32, shader_program)
        )
    }

    fn bind_texture(&mut self, texture: &DynamicImage) -> usize {
        unsafe {
            // Texture wrapping
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            // Texture filtering
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            let mut tid = 0;
            gl::GenTextures(1, &mut tid);
            gl::BindTexture(gl::TEXTURE_2D, tid);

            let data = texture.raw_pixels();
            let lod = 0;
            let internal_format = gl::RGB as i32;
            let width = texture.width() as i32;
            let height = texture.height() as i32;
            // The OpenGL documentation unironically just says "this must be 0" for the border parameter
            let magic_number = 0;
            let format = gl::RGB;
            let data_type = gl::UNSIGNED_BYTE;
            gl::TexImage2D(gl::TEXTURE_2D, lod, internal_format, width, height, magic_number, format, data_type, &data[0] as *const u8 as *const c_void);

            gl::GenerateMipmap(gl::TEXTURE_2D);

            self.loaded_textures.insert(tid)
        }
    }

    fn update_mesh_matrix(&mut self, rendered_object_id: usize, matrix: Matrix4<f32>) {
        self.rendered_objects[rendered_object_id].matrix = matrix;
    }

    // TODO implement delete_shader_program
    fn delete_shader_program() {}

    // TODO implement delete_mesh
    fn delete_mesh() {}

    pub(crate) fn handle_data_update(&mut self, update: DataUpdate) {
        match update {
            DataUpdate::MeshCreate(lmt, mesh, sp_t) => {
                let sp = sp_t.id.read()
                    .expect("Error reading shader program")
                    .expect("Error reading shader program");
                let gl_id = self.bind_mesh(&mesh, sp);
                *lmt.id.write().expect("Error writing new mesh id") = Some(gl_id);
            },
            DataUpdate::RenderedObjectCreate(rot, lmt) => {
                let lm = lmt.id.read()
                    .expect("Error reading loaded mesh")
                    .expect("Error reading loaded mesh");
                let ro = RenderedObject::new(lm);
                let ro_id = self.rendered_objects.insert(ro);
                *rot.id.write().expect("Error writing new rendered object id") = Some(ro_id);
            },
            DataUpdate::PositionSet(rot, position) => {
                let roid = rot.id.read()
                    .expect("Error getting mesh id for an operation")
                    .expect("Attempted to update RenderedObject before creating it");
                let mut matrix = self.rendered_objects[roid].matrix;
                matrix.w.x = position.x;
                matrix.w.y = position.y;
                matrix.w.z = position.z;
                self.update_mesh_matrix(roid, matrix);
            },
            DataUpdate::MatrixSet(rot, matrix) => {
                let roid = rot.id.read()
                    .expect("Error getting mesh id for an operation")
                    .expect("Attempted to update RenderedObject before creating it");
                self.update_mesh_matrix(roid, matrix);
            },
            DataUpdate::VertShaderCreate(st, src) => {
                let s_id = self.create_vert_shader(src);
                *st.id.write().expect("Error writing new shader id") = Some(s_id);
            },
            DataUpdate::FragShaderCreate(st, src) => {
                let s_id = self.create_frag_shader(src);
                *st.id.write().expect("Error writing new shader id") = Some(s_id);
            },
            DataUpdate::ShaderProgramCreate(spt, vert_t, frag_t) => {
                // v_id is the usize representing the index of the vert shader's GL id, in self.shaders
                let v_id = vert_t.id.read()
                    .expect("Error reading shader ID")
                    .expect("Error reading shader ID");
                // vert is the actual GL id
                let vert = *self.shaders.get(v_id).expect("Attempted to make shader program using nonexistent shader");

                // essentially the same as the above, but with the option wrapper around frag
                let frag = match frag_t {
                    Some(f) => {
                        let f_id = f.id.read()
                            .expect("Error reading shader ID")
                            .expect("Attempted to make shader program using nonexistent shader");
                        Some(*self.shaders.get(f_id).expect("Attempted to make shader program using nonexistent shader"))
                    },
                    None => None,
                };

                let sp_id = self.create_shader_program(vert, frag);
                *spt.id.write().expect("Error writing new shader id") = Some(sp_id);
            },
            DataUpdate::CameraSet(cam_mat) => {
                self.camera = cam_mat;
            },
            DataUpdate::TextureCreate(tt, img) => {
                let tt_id = self.bind_texture(&img);
                *tt.id.write().expect("Error writing new texture id") = Some(tt_id);
            },
        }
    }

    pub(crate) fn setup_rendering(&mut self) {
        unsafe {
            // Depth testing
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);
        }
        // default texture
        self.bind_texture(&default_texture());
    }

    pub(crate) fn draw(&self) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            for (_, ro) in self.rendered_objects.iter() {
                let loaded_mesh = self.loaded_meshes.get(ro.mesh).expect("Rendered object references a mesh that does not exist");
                let shader_program_gl_id = *match self.shader_programs.get(loaded_mesh.shader_program) {
                    Some(g) => g,
                    None => {
                        // if the referenced mesh doesn't exist, just don't render this object
                        continue;
                    },
                };

                gl::UseProgram(shader_program_gl_id);

                gl::UniformMatrix4fv(
                    gl::GetUniformLocation(shader_program_gl_id, CString::new("view").expect("CString::new failed").as_ptr()),
                    1,
                    gl::FALSE,
                    self.camera.as_ptr(),
                );

                // TODO proper projection configuration
                let proj_mat = cgmath::perspective(Deg(45.0), 640.0 / 480.0, 0.1, 100.0);
                gl::UniformMatrix4fv(
                    gl::GetUniformLocation(shader_program_gl_id, CString::new("projection").expect("CString::new failed").as_ptr()),
                    1,
                    gl::FALSE,
                    proj_mat.as_ptr(),
                );

                gl::UniformMatrix4fv(
                    gl::GetUniformLocation(shader_program_gl_id, CString::new("model").expect("CString::new failed").as_ptr()),
                    1,
                    gl::FALSE,
                    ro.matrix.as_ptr(),
                );

                let texture = self.loaded_textures[ro.texture];
                gl::BindTexture(gl::TEXTURE_2D, texture);

                gl::BindVertexArray(loaded_mesh.vao_id);
                let indices_amount = loaded_mesh.indices_len;
                gl::DrawElements(gl::TRIANGLES, indices_amount, gl::UNSIGNED_INT, ptr::null());
            }
        }
    }
}

impl Drop for DrawingInstance {
    fn drop(&mut self) {
        unsafe {
            for (_, s) in self.shaders.into_iter() {
                    gl::DeleteShader(*s);
            }
            for (_, p) in self.shader_programs.into_iter() {
                    gl::DeleteProgram(*p);
            }
            for (_, mesh) in self.loaded_meshes.into_iter() {
                gl::DeleteVertexArrays(1, &mesh.vao_id);
                gl::DeleteBuffers(1, &mesh.vbo_id);
                gl::DeleteBuffers(1, &mesh.ebo_id);
            }
        }
    }
}