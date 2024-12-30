struct CameraUniform {
    mat_0: vec3<f32>,
    mat_1: vec3<f32>,
    _pad_1: f32,
    mat_2: vec3<f32>,
    aspect: f32,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;
@group(0) @binding(1)
var<storage, read> buff: array<u32>;
@group(0) @binding(2)
var<uniform> size: vec2<u32>;

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
    let pos = full_quad[input.vertex_index];
    let view_proj = mat3x3<f32>(
        camera.mat_0,
        camera.mat_1,
        camera.mat_2,
    );
    let position = view_proj * vec3<f32>(pos, 1.0);
    var out: VertexOutput;
    out.clip_position = vec4<f32>(position, 1.0);
    out.uv = pos * 0.5 + 0.5;
    return out;
}

fn get_index(x: u32, y: u32) -> u32 {
  let h = size.y;
  let w = size.x;

  return (y % h) * w + (x % w);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let base_uv = vec2<u32>(input.uv * vec2<f32>(size));
    let index = get_index(base_uv.x, base_uv.y);
    let val = buff[index];
    if (val == 1) {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    } else {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    }
}