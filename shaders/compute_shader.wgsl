@group(0) @binding(0) var<storage, read> spheres: array<Sphere>;
@group(0) @binding(1) var<uniform> globals: Globals;
@group(0) @binding(2) var texture: texture_storage_2d<rgba8unorm, write>;

struct Sphere {
    pos: vec3<f32>,
    radius: f32,
};

struct Globals {
    screen_dim: vec2<u32>,
    max_steps: u32,
    max_dist: f32,
    camera_pos: vec3<f32>,
    surface_dist: f32,
    light_pos: vec3<f32>,
    focal_length: f32,
};

@compute @workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) coord: vec3<u32>) {

    // Left handed coordinate system, x right, y up, z in
    let uv = vec2<f32>(
        f32(coord.x) / f32(globals.screen_dim.x) * 2.0 - 1.0,
        (1.0 - f32(coord.y) / f32(globals.screen_dim.y)) * 2.0 - 1.0
    );

    let ro = globals.camera_pos;
    let rd = normalize(vec3<f32>(uv.xy, globals.focal_length));
    let dist = raymarch(ro, rd);



    var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    if dist < globals.max_dist {
        color = vec4<f32>(1.0);
    }
    color = 1.0 - vec4<f32>(vec3<f32>(dist / globals.max_dist), 1.0);

    // color = vec4<f32>(uv, 0.0, 1.0);
    textureStore(texture, coord.xy, color);
}

fn raymarch(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;

    for (var i = 0u; i < globals.max_steps; i++) {
        let pos = ro + rd * t;
        let dist = map(pos);

        t = t + dist;

        if dist < globals.surface_dist {
            break;
        }
        if t > globals.max_dist {
            break;
        }
    }

    return t;
}

fn map(pos: vec3<f32>) -> f32 {
    var min_dist = globals.max_dist;
    for (var i = 0; i < 2; i++) {
        let sphere = spheres[i];
        let dist = sphere_sdf(pos, sphere.pos, sphere.radius);
        if dist < min_dist {
            min_dist = dist;
        }
    }
    return min_dist;
}

fn sphere_sdf(pos: vec3<f32>, translation: vec3<f32>, radius: f32) -> f32 {
    return length(pos - translation) - radius;
}

// const focal: f32 = 1.0;
// const max_steps: i32 = 50;
// const max_dist: f32 = 10.0;
// const surface_dist: f32 = 0.001;
