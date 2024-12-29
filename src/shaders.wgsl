struct CameraUniform {
    mat_0: vec3<f32>,
    mat_1: vec3<f32>,
    _pad_1: f32,
    mat_2: vec3<f32>,
    aspect: f32,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32
}
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>
}

const full_quad: array<vec2<f32>, 3> = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0, 1.0),
);

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let pos = full_quad[input.vertex_index];
    let view_proj = mat3x3<f32>(
        camera.mat_0,
        camera.mat_1,
        camera.mat_2,
    );
    let position = view_proj * vec3<f32>(pos, 1.0);
    var out: VertexOutput;
    out.clip_position = vec4<f32>(position, 1.0);
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}