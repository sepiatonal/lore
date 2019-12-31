use crate::rendering::rendering::*;
use lore_mesh::Mesh;
use cgmath::{Vector3, Matrix4};
use image::DynamicImage;

pub(crate) trait DataUpdate: Send + Sync {
    fn apply(&self, inst: &mut DrawingInstance);
}

pub(crate) struct TextureSet {
    pub(crate) ticket: RenderedObjectTicket,
    pub(crate) texture_ticket: TextureTicket,
}

impl DataUpdate for TextureSet {
    fn apply(&self, inst: &mut DrawingInstance) {
        let roid = self.ticket.id.read()
            .expect("Error getting RenderedObject id for an operation")
            .expect("Attempted to update RenderedObject before creating it");
        let tid = self.texture_ticket.id.read()
            .expect("Error getting Texture id for an operation")
            .expect("Attempted to use uncreated Texture");
        inst.rendered_objects[roid].texture = tid;
    }
}

pub(crate) struct TextureCreate {
    pub(crate) ticket: TextureTicket,
    pub(crate) image: DynamicImage,
}

impl DataUpdate for TextureCreate {
    fn apply(&self, inst: &mut DrawingInstance) {
        let tt_id = inst.bind_texture(&self.image);
        *self.ticket.id.write().expect("Error writing new texture id") = Some(tt_id);
    }
}

pub(crate) struct CameraMatrixSet {
    pub(crate) matrix: Matrix4<f32>,
}

impl DataUpdate for CameraMatrixSet {
    fn apply(&self, inst: &mut DrawingInstance) {
        inst.camera = self.matrix;
    }
}

pub(crate) struct ShaderProgramCreate {
    pub(crate) ticket: ShaderProgramTicket,
    pub(crate) vert_shader_ticket: ShaderTicket,
    pub(crate) frag_shader_ticket: Option<ShaderTicket>,
}

impl DataUpdate for ShaderProgramCreate {
    fn apply(&self, inst: &mut DrawingInstance) {
        // v_id is the usize representing the index of the vert shader's GL id, in self.shaders
        let v_id = self.vert_shader_ticket.id.read()
            .expect("Error reading shader ID")
            .expect("Error reading shader ID");
        // vert is the actual GL id
        let vert = *inst.shaders.get(v_id).expect("Attempted to make shader program using nonexistent shader");

        // essentially the same as the above, but with the option wrapper around frag
        let frag = match &self.frag_shader_ticket {
            Some(f) => {
                let f_id = f.id.read()
                    .expect("Error reading shader ID")
                    .expect("Attempted to make shader program using nonexistent shader");
                Some(*inst.shaders.get(f_id).expect("Attempted to make shader program using nonexistent shader"))
            },
            None => None,
        };

        let sp_id = inst.create_shader_program(vert, frag);
        *self.ticket.id.write().expect("Error writing new shader id") = Some(sp_id);
    }
}

pub(crate) struct FragmentShaderCreate {
    pub(crate) ticket: ShaderTicket,
    pub(crate) source: &'static str,
}

impl DataUpdate for FragmentShaderCreate {
    fn apply(&self, inst: &mut DrawingInstance) {
        let s_id = inst.create_frag_shader(self.source);
        *self.ticket.id.write().expect("Error writing new shader id") = Some(s_id);
    }
}

pub(crate) struct VertexShaderCreate {
    pub(crate) ticket: ShaderTicket,
    pub(crate) source: &'static str,
}

impl DataUpdate for VertexShaderCreate {
    fn apply(&self, inst: &mut DrawingInstance) {
        let s_id = inst.create_vert_shader(self.source);
        *self.ticket.id.write().expect("Error writing new shader id") = Some(s_id);
    }
}

pub(crate) struct MatrixSet {
    pub(crate) ticket: RenderedObjectTicket,
    pub(crate) matrix: Matrix4<f32>,
}

impl DataUpdate for MatrixSet {
    fn apply(&self, inst: &mut DrawingInstance) {
        let roid = self.ticket.id.read()
            .expect("Error getting mesh id for an operation")
            .expect("Attempted to update RenderedObject before creating it");
        inst.update_mesh_matrix(roid, self.matrix);
    }
}

pub(crate) struct PositionSet {
    pub(crate) ticket: RenderedObjectTicket,
    pub(crate) position: Vector3<f32>,
}

impl DataUpdate for PositionSet {
    fn apply(&self, inst: &mut DrawingInstance) {
        let roid = self.ticket.id.read()
            .expect("Error getting mesh id for an operation")
            .expect("Attempted to update RenderedObject before creating it");
        let mut matrix = inst.rendered_objects[roid].matrix;
        matrix.w.x = self.position.x;
        matrix.w.y = self.position.y;
        matrix.w.z = self.position.z;
        inst.update_mesh_matrix(roid, matrix);
    }
}

pub(crate) struct MeshCreate {
    pub(crate) ticket: LoadedMeshTicket,
    pub(crate) mesh: Mesh,
    pub(crate) shader_program_ticket: ShaderProgramTicket,
}

impl DataUpdate for MeshCreate {
    fn apply(&self, inst: &mut DrawingInstance) {
        let sp = self.shader_program_ticket.id.read()
            .expect("Error reading shader program")
            .expect("Error reading shader program");
        let gl_id = inst.bind_mesh(&self.mesh, sp);
        *self.ticket.id.write().expect("Error writing new mesh id") = Some(gl_id);
    }
}

pub(crate) struct RenderedObjectCreate {
    pub(crate) ticket: RenderedObjectTicket,
    pub(crate) mesh_ticket: LoadedMeshTicket,
}

impl DataUpdate for RenderedObjectCreate {
    fn apply(&self, inst: &mut DrawingInstance) {
        let lm = self.mesh_ticket.id.read()
            .expect("Error reading loaded mesh")
            .expect("Error reading loaded mesh");
        let ro = RenderedObject::new(lm);
        let ro_id = inst.rendered_objects.insert(ro);
        *self.ticket.id.write().expect("Error writing new rendered object id") = Some(ro_id);
    }
}
