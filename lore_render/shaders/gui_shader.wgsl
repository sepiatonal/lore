// Vertex shader
struct CameraUniform {
    matrix: mat4x4<f32>;
};
[[group(1), binding(0)]]
var<uniform> camera: CameraUniform;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
};

struct InstanceInput {
    [[location(5)]] matrix_0: vec4<f32>;
    [[location(6)]] matrix_1: vec4<f32>;
    [[location(7)]] matrix_2: vec4<f32>;
    [[location(8)]] matrix_3: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] normal: vec3<f32>;
    [[location(1)]] uv: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    let instance_matrix = mat4x4<f32>(
        instance.matrix_0,
        instance.matrix_1,
        instance.matrix_2,
        instance.matrix_3,
    );

    // transform normals to match the object's rotation
    // note that this fails if any scaling is done, unless we use the inverse transpose of model_view_proj
    out.normal = model.normal;
    out.uv = model.uv;
    out.clip_position = instance_matrix * vec4<f32>(model.position, 1.0);
    
    return out;
}

[[group(0), binding(0)]]
var texture_view: texture_2d<f32>;
[[group(0), binding(1)]]
var texture_sampler: sampler;

[[stage(fragment)]]
fn fs_main(model: VertexOutput) -> [[location(0)]] vec4<f32> {
    return textureSample(texture_view, texture_sampler, model.uv);
}