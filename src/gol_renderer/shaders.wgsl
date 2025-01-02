struct CameraUniform {
    view_proj: mat4x4<f32>,
    quad_transform: mat4x4<f32>
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;
@group(0) @binding(1)
var tex: texture_2d<u32>;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32
}
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>
}

const full_quad: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(1.0, 1.0)
);

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let pos = full_quad[input.vertex_index];
    out.clip_position = camera.view_proj * camera.quad_transform * vec4<f32>(pos, 1.0, 1.0);

    let uv_flipped = pos * 0.5 + 0.5;
    out.uv = vec2(uv_flipped.x, 1.0 - uv_flipped.y);

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // since texture is u32, need to use integer pixel uv instead of float
    let base_uv = vec2<i32>(input.uv * vec2<f32>(textureDimensions(tex)));
    let val = textureLoad(tex, base_uv, 0).x;
    if (val == 1) {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    } else {
        return vec4<f32>(0.005, 0.005, 0.005, 1.0);
    }
}