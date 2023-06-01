// Vertex shader


struct VertexInput {
    @location(0) position: vec3<f32>
}

struct VertexOutput {
    @builtin(position) frag_coord: vec4<f32>,
    @location(0) position: vec3<f32>,
}

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.frag_coord = vec4<f32>(vertex.position, 1.0);
    out.position = vertex.position;
    return out;
}

// Fragment shader

const focal: f32 = 1.0;
const max_steps: i32 = 50;
const max_dist: f32 = 10.0;
const surface_dist: f32 = 0.001;

fn map(pos: vec3<f32>) -> f32 {
    return length(pos) - 1.0;
}

fn raymarch(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;

    for (var i = 0; i < max_steps; i++) {
        let pos = ro + rd * t;
        let dist = map(pos);

        t = t + dist;

        if dist < surface_dist {
            break;
        }
        if t > max_dist {
            break;
        }
    }

    return t;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    // let resolution: vec2<f32> = vec2<f32>(1280.0, 720.0);
    // let uv = (vertex.frag_coord - 0.5 * vec4<f32>(resolution.xy, 0.0, 0.0)) / resolution.y;
    let uv = vertex.position;
    let ro = vec3<f32>(0.0, 0.0, -5.0);
    let rd = normalize(vec3<f32>(uv.xy, focal));
    let dist = raymarch(ro, rd);

    return vec4<f32>(1.0 - (dist / max_dist));

    // if dist < max_dist {
    //     return vec4<f32>(1.0);
    // } else {
    //     return vec4<f32>(0.0);
    // }
    // var color = vec3<f32>(1.0, 1.0, 0.0);
    // let gamma_color = vec3<f32>(
    //     pow(color.x, 0.4545),
    //     pow(color.y, 0.4545),
    //     pow(color.z, 0.4545)
    // );
}
