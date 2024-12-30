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
   // since texture is u32, need to use integer pixel uv instead of float
   let base_uv = vec2<i32>(input.uv * vec2<f32>(textureDimensions(tex)));

   var neighbors_alive: u32 = 0;
   for (var x: i32 = -1; x <= 1; x++) {
       for (var y: i32 = -1; y <= 1; y++) {
           if (x == 0 && y == 0) {
               continue;
           }
           let offset = vec2<i32>(x, y);
           let uv = base_uv + offset;
           neighbors_alive += textureLoad(tex, uv, 0).x;
       }
   }

   let curr_value = textureLoad(tex, base_uv, 0).x;
   if (neighbors_alive == 3) {
       return 1u;
   } else if (curr_value == 1 && neighbors_alive == 2) {
       return 1u;
   } else {
       return 0u;
   }
}