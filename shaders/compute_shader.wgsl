@group(0) @binding(0) var<storage, read> shapes: array<Shape>;
@group(0) @binding(1) var<uniform> g: Globals;
@group(0) @binding(2) var texture: texture_storage_2d<rgba8unorm, write>;
 
struct Shape {
    pos: vec3<f32>,
    id: u32,
    v1: vec3<f32>,
    f1: f32,
};

const SPHERE_ID: u32 = 0u;
const BOX_EXACT_ID: u32 = 1u;
const PLANE_ID: u32 = 2u;


// Box exact
// id: 1
// v1 b

// Plane
// id: 2
// v1: normal

struct Globals {
    screen_dim: vec2<u32>,
    camera_pos: vec3<f32>,
    camera_rot: mat3x3<f32>,
    light_pos: vec3<f32>,
    focal_length: f32,
    time: f32,
    shape_amount: u32,
};

const max_steps: u32 = 100u;
const max_dist: f32 = 50.0;
const surface_dist: f32 = 0.0001;
const epsilon: f32 = 0.00001; // surface_dist * 0.1
const shadow_step: f32 = 0.005; // surface_dist * 50
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
const fog_inesity: f32 = 2.0;

const stack_size: u32 = 10u;

@compute @workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) coord: vec3<u32>) {

    // Left handed coordinate system, x right, y up, z in
    let uv = vec2<f32>(
        f32(coord.x) / f32(g.screen_dim.x) * 2.0 - 1.0,
        (1.0 - f32(coord.y) / f32(g.screen_dim.y)) * 2.0 - 1.0
    );

    let ro = g.camera_pos; // + vec3<f32>(g.time, 0.0, 0.0);
    let rd = normalize(g.camera_rot * vec3<f32>(uv.xy, g.focal_length));
    let color = raymarch_color(ro, rd);

    textureStore(texture, coord.xy, vec4<f32>(color, 1.0));

    // var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    // if dist < g.max_dist {
    //    color = vec4<f32>(1.0);
    // }
    // color = 1.0 - vec4<f32>(vec3<f32>(dist / g.max_dist), 1.0);

    // color = vec4<f32>(uv, 0.0, 1.0);
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
        //return vec3(1.0);
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

    let fog = 1.0 - length(g.camera_pos - pos) / max_dist;

    var color = vec3<f32>(0.0, 1.0, 1.0);

    let light = (ambient + back + fresnel) * occlusion + (diffuse + specular * occlusion) * shadow;
    color *= light * fog;

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

struct SE {
    op_type: i32, // 0 un, 1 in, 2, sub, 3 sun, 4 sin, 5 ssub
    op_amount: i32,
    dist: f32,
}

fn map(pos: vec3<f32>) -> f32 {
    var stack = array<SE, 10>();
    var si = 0; // stack index
    stack[si] = SE(0, i32(g.shape_amount), max_dist);
    var i = 0;

    while true {
        // Pop operation result from stack
        // Return if stack empty
        if stack[si].op_amount == 0 {
            if si == 0 {
                break;
            } else {
                si--;
                stack[si].dist = min(stack[si].dist, stack[si+1].dist);
                continue;
            }
        }
        stack[si].op_amount--;

        switch shapes[i].id {
            // Push union to stack
            case 3u: {
                si++;
                stack[si] = SE(0,2, max_dist);
            }
            // Push intersection to stack
            case 4u: {
                si++;
                stack[si] = SE(1,2, -1.0);
            }
            // Perform current operation on stack
            default: {
                switch stack[si].op_type {
                    // Union
                    case 0: {
                        stack[si].dist = min(stack[si].dist, shape_dist(pos, i));
                    }
                    // Intersection
                    case 1: {
                        stack[si].dist = max(stack[si].dist, shape_dist(pos, i));
                    }
                    default: {}
                }
            }
        }

        i++;
    }
    return stack[si].dist;
}

fn shape_dist(pos: vec3<f32>, i: i32) -> f32 {
    let shape = shapes[i];
    switch shape.id {
        case 0u: {
            return sphere_sdf(pos, shape);
        }
        case 1u: {
            return box_exact_sdf(pos, shape);
        }
        case 2u: {
            return plane_sdf(pos, shape);
        }
        default: {
            return max_dist;
        }
    }
}

// f1: radius
fn sphere_sdf(pos: vec3<f32>, shape: Shape) -> f32 {
    return length(pos - shape.pos) - shape.f1;
}

// v1: b
fn box_exact_sdf(pos: vec3<f32>, shape: Shape) -> f32 {
    let q = abs(pos - shape.pos) - shape.v1;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// v1: normal
fn plane_sdf(pos: vec3<f32>, shape: Shape) -> f32 {
    return dot((pos - shape.pos), shape.v1);
}

// 
// fn get_dist_2(pos: vec3<f32>, idx: u32, op: i32, dist: f32) -> f32 {
//     let shape = shapes[idx];
//     let id = shape.id;
// 
//     switch op {
//         // Normal
//         case 0: {
//             return min(dist)
//         }
// 
//     }
// 
//     switch id {
//         case 
// 
//     }
// }
// 
// fn get_dist_3(pos: vec3<f32>, idx: u32) -> f32 {
//     let shape = shapes[idx];
//     let id = shape.id;
// 
//     switch id {
//         case 0u: {
//             return sphere_sdf(pos, shape);
//         }
//     }
// }
// 
// fn get_dist(pos: vec3<f32>, i: u32) -> f32 {
//     let id = shapes[i].id;
//     switch id {
//         case 0u: {
//             return sphere_sdf(pos, i);
//         }
//         case 1u: {
//             // return box_exact_sdf(pos, shape);
//         }
//         case 2u: {
//             //return plane_sdf(pos, shape);
//         }
//         case 3u: {
//             return union_sdf(pos, i);
//         }
//         default: {}
//     };
//     return max_dist;
// }

// fn sphere_sdf(pos: vec3<f32>, i: u32) -> f32 {
//     let shape = shapes[i];
//     return length(pos - shape.pos) - shape.f1;
// }

// f1: radius
// 
// // v1: b
// fn box_exact_sdf(pos: vec3<f32>, shape: Shape) -> f32 {
//     let q = abs(pos - shape.pos) - shape.v1;
//     return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
// }
// 
// // v1: normal
// fn plane_sdf(pos: vec3<f32>, shape: Shape) -> f32 {
//     return dot((pos - shape.pos), shape.v1);
// }
// 
// fn union_sdf(pos: vec3<f32>, shape1: Shape, shape2: Shape) -> f32 {
//     return min()
// }

// fn plane_sdf2(pos: vec3<f32>, normal: vec3<f32>, translation: vec3<f32>) -> f32 {
//     return dot((pos - translation), normal);
//     // return dot(pos, normal) - dist_along_normal;
// }
// fn sphere_sdf(pos: vec3<f32>, translation: vec3<f32>, radius: f32) -> f32 {
//     return length(pos - translation) - radius;
// }

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
