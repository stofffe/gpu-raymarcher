@group(0) @binding(0) var<storage, read_write> pixels: array<u32>;
@group(0) @binding(1) var<uniform> globals: Globals;
@group(0) @binding(2) var texture: texture_storage_2d<rgba8unorm, write>;

struct Globals {
    width: u32,
    height: u32,
    test: f32,
};

@compute @workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    // pixels[i] = pixels[i] + 1u;
    pixels[i] = u32(globals.test);

    textureStore(texture, vec2<u32>(i, i), vec4<f32>(0.0, 1.0, 1.0, 1.0));
}
