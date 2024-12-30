@group(0) @binding(0) var<storage, read> read: array<u32>;
@group(0) @binding(1) var<storage, read_write> write: array<u32>;
@group(0) @binding(2) var<uniform> size: vec2<u32>;

const block_size = 8u;

struct ComputeInput {
    @builtin(global_invocation_id) global_id: vec3<u32>
}

fn get_index(x: u32, y: u32) -> u32 {
  let h = size.y;
  let w = size.x;

  return (y % h) * w + (x % w);
}

fn get_cell(x: u32, y: u32) -> u32 {
    return read[get_index(x, y)];
}

fn count_neighbors(x: u32, y: u32) -> u32 {
    return get_cell(x - 1, y - 1) +
        get_cell(x - 1, y) +
        get_cell(x - 1, y + 1) +
        get_cell(x, y - 1) +
        get_cell(x, y + 1) +
        get_cell(x + 1, y - 1) +
        get_cell(x + 1, y) +
        get_cell(x + 1, y + 1);
}

@compute
@workgroup_size(block_size, block_size, 1)
fn main(input: ComputeInput) {
    let x = input.global_id.x;
    let y = input.global_id.y;

    let neighbors_alive = count_neighbors(x, y);

    write[get_index(x, y)] = u32(neighbors_alive == 3 || (get_cell(x, y) == 1 && neighbors_alive == 2));
}
