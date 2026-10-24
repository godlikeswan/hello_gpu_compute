@group(0) @binding(0)
var<storage, read_write> data: array<u32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= arrayLength(&data)) {
        return;
    }

    var x = index;
    var hash: u32 = 0u;

    // Fast PRNG bit-mixing loop (strictly integer-based)
    for (var i = 0u; i < 300u; i = i + 1u) {
        x = x ^ (x << 13u);
        x = x ^ (x >> 17u);
        x = x ^ (x << 5u);
        hash = hash + x + i;
    }

    data[index] = hash;
}