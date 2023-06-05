@group(0) @binding(0) var<storage, read> spheres: array<Sphere>;
@group(0) @binding(1) var<uniform> g: Globals;
@group(0) @binding(2) var texture: texture_storage_2d<rgba8unorm, write>;
 
// const colors: array<vec3<f32>, 4> = array<vec3<f32>, 4>(
//     vec3<f32>(1.0, 1.0, 1.0),
//     vec3<f32>(1.0, 0.0, 0.0),
//     vec3<f32>(0.0, 1.0, 0.0),
//     vec3<f32>(0.0, 0.0, 1.0),
// );

struct Sphere {
    pos: vec3<f32>,
    radius: f32,
};

struct Globals {
    screen_dim: vec2<u32>,
    camera_pos: vec3<f32>,
    light_pos: vec3<f32>,
    focal_length: f32,
    time: f32,
};

const max_steps: u32 = 100u;
const max_dist: f32 = 50.0;
const surface_dist: f32 = 0.0001;
const epsilon: f32 = 0.00001;
const shadow_step: f32 = 0.005;
const soft_shadow_sharpness: f32 = 8.0;
const specular_sharpness: f32 = 10.0;
const specular_intensity: f32 = 0.3;
const diffuse_intensity: f32 = 0.7;
const occlusion_intensity: f32 = 1.0;
const occlusion_init_step = 0.01;
const occlusion_step_scale = 0.01;
const occlusion_weight_drop = 0.85;
const ambient_intensity: f32 = 0.05;
const back_intensity: f32 = 0.05;
const fresnel_intensity: f32 = 0.15;

@compute @workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) coord: vec3<u32>) {

    // Left handed coordinate system, x right, y up, z in
    let uv = vec2<f32>(
        f32(coord.x) / f32(g.screen_dim.x) * 2.0 - 1.0,
        (1.0 - f32(coord.y) / f32(g.screen_dim.y)) * 2.0 - 1.0
    );

    let ro = g.camera_pos + vec3<f32>(g.time, 0.0, 0.0);
    let rd = normalize(vec3<f32>(uv.xy, g.focal_length));
    let color = raymarch_color(ro, rd);

    // var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    // if dist < g.max_dist {
    //    color = vec4<f32>(1.0);
    // }
    // color = 1.0 - vec4<f32>(vec3<f32>(dist / g.max_dist), 1.0);

    // color = vec4<f32>(uv, 0.0, 1.0);
    textureStore(texture, coord.xy, vec4<f32>(color, 1.0));
}

fn raymarch(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;

    for (var i = 0u; i < max_steps; i++) {
        let pos = ro + rd * t;
        let dist = map(pos);

        t += dist;

        if dist < surface_dist {
            break;
        }
        if t > max_dist {
            break;
        }
    }
    return t;
}

fn raymarch_color(ro: vec3<f32>, rd: vec3<f32>) -> vec3<f32> {
    let dist = raymarch(ro, rd);
    if dist < max_dist {
        let pos = ro + rd * dist;
        return hit(pos, rd);
    } else {
        return miss();
    }
}

fn hit(pos: vec3<f32>, rd: vec3<f32>) -> vec3<f32> {
    let normal = normal(pos);
    let light_dir = normalize(g.light_pos - pos);
    let reflected_dir = normalize(reflect(-light_dir, normal));
    let view_dir = normalize(-rd);

    let ambient = ambient_intensity;
    let specular = specular_intensity * pow(clamp(dot(reflected_dir, view_dir), 0.0, 1.0), specular_sharpness);
    let diffuse = diffuse_intensity * clamp(dot(light_dir, normal), 0.0, 1.0);
    let fresnel = fresnel_intensity * pow(1.0 + dot(rd, normal), 5.0);
    let back = back_intensity * clamp(dot(normal, -light_dir), 0.0, 1.0);

    let shadow = soft_shadow(pos, soft_shadow_sharpness);
    let occlusion = ambient_occlusion(pos, normal);

    var color = vec3<f32>(0.0, 1.0, 1.0);

    let light = (ambient + back + fresnel) * occlusion + (diffuse + specular * occlusion) * shadow;
    color *= light;

    // Gamma correction
    color = pow(color, vec3<f32>(0.4545));

    return color;
}

fn miss() -> vec3<f32> {
    return vec3<f32>(0.0, 0.0, 0.0);
}

// TODO check parameters 0.01, 0.01 and steps
fn ambient_occlusion(pos: vec3<f32>, normal: vec3<f32>) -> f32 {
    var occlusion = 0.0;
    var weight = 1.0;
    for (var i = 0; i < 8; i++) {
        let len = occlusion_init_step + occlusion_step_scale * f32(i * i);
        let dist = map(pos + normal * len);
        occlusion += (len - dist) * weight;
        weight *= occlusion_weight_drop;
    }
    return 1.0 - clamp(occlusion_intensity * occlusion, 0.0, 1.0);
}

fn normal(pos: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(epsilon, 0.0);
    let center = map(pos);
    let diff = vec3<f32>(
        map(pos + e.xyy) - center,
        map(pos + e.yxy) - center,
        map(pos + e.yyx) - center,
    );
    return normalize(diff);
}

fn hard_shadow(pos: vec3<f32>) -> f32 {
    let light_dir = normalize(g.light_pos - pos);
    let light_dist = length(g.light_pos - pos);
    let start_pos = pos + light_dir * shadow_step;

    let dist = raymarch(start_pos, light_dir);
    if dist < light_dist {
        return 0.0;
    } else {
        return 1.0;
    }
}

fn soft_shadow(pos: vec3<f32>, k: f32) -> f32 {
    let light_dir = normalize(g.light_pos - pos);
    let light_dist = length(g.light_pos - pos);

    var shadow = 1.0;
    var ph = 1e20;
    var t = shadow_step;
    for (var i = 0u; i < max_steps; i++) {
        let pos = pos + light_dir * t;
        let dist = map(pos);

        let y = dist * dist / (2.0 * ph);
        let d = sqrt(dist * dist - y * y);
        shadow = min(shadow, k * d / max(0.0, t - y));
        ph = dist;
        t += dist;

        if t >= light_dist {
            break;
        }
        if dist < surface_dist {
            break;
        }
    }

    shadow = clamp(shadow, 0.0, 1.0);
    return shadow;
}


fn map(pos: vec3<f32>) -> f32 {
    var min_dist = max_dist;
    for (var i = 0u; i < arrayLength(&spheres); i++) {
        let sphere = spheres[i];
        let dist = sphere_sdf(pos, sphere.pos, sphere.radius);
        min_dist = min(min_dist, dist);
    }

    let plane = plane_sdf(pos, vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0, -1.0, 0.0));
    min_dist = min(min_dist, plane);

    let plane2 = plane_sdf(pos, vec3(1.0, 0.0, 0.0), vec3<f32>(-3.0, 0.0, 0.0));
    min_dist = min(min_dist, plane2);

    return min_dist;
}

fn plane_sdf(pos: vec3<f32>, normal: vec3<f32>, translation: vec3<f32>) -> f32 {
    return dot((pos - translation), normal);
    // return dot(pos, normal) - dist_along_normal;
}

fn sphere_sdf(pos: vec3<f32>, translation: vec3<f32>, radius: f32) -> f32 {
    return length(pos - translation) - radius;
}

// fn soft_shadow(pos: vec3<f32>, k: f32) -> f32 {
//     let light_dir = normalize(g.light_pos - pos);
//     let light_dist = length(g.light_pos - pos);
// 
//     var t = g.shadow_step;
//     var shadow = 1.0;
// 
//     for (var i = 0u; i < g.max_steps; i++) {
//         if t >= light_dist {
//             return shadow;
//         }
// 
//         let pos = pos + light_dir * t;
//         let dist = map(pos);
// 
//         if dist < g.surface_dist {
//             return 0.0;
//         }
// 
//         shadow = min(shadow, k * dist / t);
//         t = t + dist;
//     }
// 
//     return shadow;
