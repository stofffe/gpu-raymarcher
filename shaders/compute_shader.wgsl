@group(0) @binding(0) var<storage, read_write> v_input: array<u32>;

@compute @workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    v_input[i] = v_input[i] + 1u;
}
