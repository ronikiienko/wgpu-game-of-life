@group(0) @binding(0) var tex: texture_2d<u32>;
@group(0) @binding(1) var tex_sampler: sampler;

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
    out.clip_position = vec4<f32>(pos, 0.0, 1.0);

    let uv_flipped = pos * 0.5 + 0.5;
    out.uv = vec2(uv_flipped.x, 1.0 - uv_flipped.y);
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) u32 {
   let base_uv = vec2<i32>(input.uv * vec2<f32>(textureDimensions(tex)));

    let neighbors_alive = textureLoad(tex, base_uv + vec2<i32>(-1, -1), 0).x +
        textureLoad(tex, base_uv + vec2<i32>(-1, 0), 0).x +
        textureLoad(tex, base_uv + vec2<i32>(-1, 1), 0).x +
        textureLoad(tex, base_uv + vec2<i32>(0, -1), 0).x +
        textureLoad(tex, base_uv + vec2<i32>(0, 1), 0).x +
        textureLoad(tex, base_uv + vec2<i32>(1, -1), 0).x +
        textureLoad(tex, base_uv + vec2<i32>(1, 0), 0).x +
        textureLoad(tex, base_uv + vec2<i32>(1, 1), 0).x;

   let curr_value = textureLoad(tex, base_uv, 0).x;

   return u32(neighbors_alive == 3 || (curr_value == 1 && neighbors_alive == 2));
}