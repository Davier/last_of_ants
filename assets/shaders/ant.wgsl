#import bevy_sprite::mesh2d_functions
#import bevy_sprite::mesh2d_view_bindings::{view, globals}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
#ifdef VERTEX_POSITIONS
    @location(0) position: vec3<f32>,
#endif
#ifdef VERTEX_NORMALS
    @location(1) normal: vec3<f32>,
#endif
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(3) tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
#endif
    @location(5) instance_color_primary: vec4<f32>,
    @location(6) instance_color_secondary: vec4<f32>,
    @location(7) instance_phase: f32,
};

struct VertexOutput {
    // this is `clip position` when the struct is used as a vertex stage output 
    // and `frag coord` when used as a fragment stage input
    @builtin(position) position: vec4<f32>,
    @location(0) world_sosition: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    #ifdef VERTEX_TANGENTS
    @location(3) world_tangent: vec4<f32>,
    #endif
    #ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
    #endif
    @location(5) instance_color_primary: vec4<f32>,
    @location(6) instance_color_secondary: vec4<f32>,
    @location(7) instance_phase: f32,
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif

#ifdef VERTEX_POSITIONS
    var model = mesh2d_functions::get_model_matrix(vertex.instance_index);
    out.world_sosition = mesh2d_functions::mesh2d_position_local_to_world(
        model,
        vec4<f32>(vertex.position, 1.0)
    );
    out.position = mesh2d_functions::mesh2d_position_world_to_clip(out.world_sosition);
#endif

#ifdef VERTEX_NORMALS
    out.world_normal = mesh2d_functions::mesh2d_normal_local_to_world(vertex.normal, vertex.instance_index);
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh2d_functions::mesh2d_tangent_local_to_world(
        model,
        vertex.tangent
    );
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

    out.instance_color_primary = vertex.instance_color_primary;
    out.instance_color_secondary = vertex.instance_color_secondary;
    out.instance_phase = vertex.instance_phase;
    return out;
}

@fragment
fn fragment(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    if material_is_side == 0u {
        return fragment_top(in);
    } else {
        return fragment_side(in);
    }
}


@group(1) @binding(0) var<uniform> material_is_side: u32;

const PI: f32 = 3.1415;
const animation_speed = 4.; // Cycles per second

fn fragment_side(mesh: VertexOutput) -> vec4<f32> {
    let anim_1: f32 = cos(globals.time * 2. * PI * animation_speed + mesh.instance_phase);
    var pos = (mesh.uv.xy * 2. - 1.) * 150.;
    pos.y = pos.y - 70.;
    // Head
    var d_s = sd_circle(pos + vec2(60., -10.), 30.);
    // Thorax
    var d_p = sd_rounded_box(pos, vec2(50., 25.), 32.);
    // Waist
    d_s = opUnion(d_s, sd_circle(pos + vec2(-55., 0.), 20.));
    // Bottom
    d_s = opUnion(d_s, sd_rounded_box(rotate(pos + vec2(-90., -10.), -PI / 6.), vec2(40., 27.), 40.));
    // Antenna
    d_s = opUnion(d_s, sd_segment(pos, vec2(-100., -20.), vec2(-125., 65.)));
    d_s = opUnion(d_s, sd_segment(pos, vec2(-100., -20.), vec2(-60., 10.)));
    // Leg front
    d_s = opUnion(d_s, sd_segment(pos, vec2(-10., -40.), vec2(-40. + anim_1 * 10., 65.)));
    d_s = opUnion(d_s, sd_segment(pos, vec2(-10., -40.), vec2(10., 20.)));
    // Leg middle
    d_s = opUnion(d_s, sd_segment(pos, vec2(10., -40.), vec2(0. - anim_1 * 10., 65.)));
    d_s = opUnion(d_s, sd_segment(pos, vec2(10., -40.), vec2(20., 20.)));
    // Leg back
    d_s = opUnion(d_s, sd_segment(pos, vec2(30., -40.), vec2(60. + anim_1 * 10., 65.)));
    d_s = opUnion(d_s, sd_segment(pos, vec2(30., -40.), vec2(30., 20.)));

    return combine_ant_parts(mesh, d_p, d_s);
}

fn fragment_top(mesh: VertexOutput) -> vec4<f32> {
    let anim_1: f32 = cos(globals.time * 2. * PI * animation_speed + mesh.instance_phase);
    var pos = (mesh.uv.xy * 2. - 1.) * 150.;
    // Vertical symmetry
    pos.x = abs(pos.x);
    // Head
    var d_s = sd_circle(pos + vec2(0., 60.), 30.);
    // Thorax
    var d_p = sd_rounded_box(pos, vec2(25., 50.), 32.);
    // Waist
    d_s = opUnion(d_s, sd_circle(pos + vec2(0., -55.), 20.));
    // Bottom
    d_s = opUnion(d_s, sd_rounded_box(pos + vec2(0., -90.), vec2(27., 40.), 40.));
    // Antenna
    d_s = opUnion(d_s, sd_segment(pos, vec2(50., -110.), vec2(25., -145.)));
    d_s = opUnion(d_s, sd_segment(pos, vec2(50., -110.), vec2(10., -60.)));
    // Leg front
    d_s = opUnion(d_s, sd_segment(pos, vec2(40., -30.), vec2(80., -40. + anim_1 * 10.)));
    d_s = opUnion(d_s, sd_segment(pos, vec2(40., -30.), vec2(20., 10.)));
    // Leg middle
    d_s = opUnion(d_s, sd_segment(pos, vec2(40., 0.), vec2(85., 15. - anim_1 * 10.)));
    d_s = opUnion(d_s, sd_segment(pos, vec2(40., 0.), vec2(20., 20.)));
    // Leg back
    d_s = opUnion(d_s, sd_segment(pos, vec2(40., 30.), vec2(75., 80. + anim_1 * 10.)));
    d_s = opUnion(d_s, sd_segment(pos, vec2(40., 30.), vec2(20., 30.)));


    return combine_ant_parts(mesh, d_p, d_s);
}

fn combine_ant_parts(mesh: VertexOutput, d_p: f32, d_s: f32) -> vec4<f32> {
    let primary_color = sd_color_smooth(d_p - 3., mesh.instance_color_primary.xyz);
    let secondary_color = sd_color_smooth(d_s - 3., mesh.instance_color_secondary.xyz);
    let alpha = max(primary_color.a, secondary_color.a);
    let color = select(secondary_color.rgb, primary_color.rgb, secondary_color.a < primary_color.a);
    // let color = blend_colors(primary_color, secondary_color);


    // let color_halo = vec3(1., 0.6, 0.);
    // var halo = sd_color_halo(opUnion(d_s, d_p), 50., color_halo);
    // let halo_blink: f32 = cos(globals.time * 2. * PI / 2.) * 0.5 + 0.5;
    // halo.a = halo.a * halo_blink;

    // return blend_colors(halo, vec4(color.r, color.g, color.b, alpha));
    return vec4(color.r, color.g, color.b, alpha);
    // return color;
}


fn sd_color_halo(d: f32, width: f32, color: vec3<f32>) -> vec4<f32> {
    let x = pow(d / width, 0.3);
    let alpha = smoothstep(1., 0., x);
    return vec4(color, alpha);
}

fn blend_colors(dest: vec4<f32>, source: vec4<f32>) -> vec4<f32> {
    return source.a * source.rbga + (1. - source.a) * dest.rgba;
}

fn sd_color_sharp(d: f32, color: vec3<f32>) -> vec4<f32> {
    return vec4(color, step(0., -d));
}

fn sd_color_smooth(d: f32, color: vec3<f32>) -> vec4<f32> {
    let alpha = min(1.0, pow(1.0 - d / 400., 400.));
    return vec4(color, alpha);
}

fn sd_color_border(d: f32, color: vec3<f32>) -> vec4<f32> {
    let border = 5.; //px
    let border_color = vec3(0.5);
    let new_color = select(color, border_color, abs(d) <= border);
    return vec4(new_color, step(-border, -d));
}

fn sd_color_debug(d: f32, color: vec3<f32>) -> vec4<f32> {
	// coloring
    var col = vec3(0.65, 0.85, 1.0);
    if d > 0. {
        col = vec3(0.9, 0.6, 0.3);
    }
    col *= 1.0 - exp(-0.05 * abs(d));
    col *= 0.8 + 0.2 * cos(2. * 3.1415 * d / 10.);
    col = mix(col, vec3(1.0), 1.0 - smoothstep(0.0, 2., abs(d)));

    return vec4(col, 1.0);
}

fn rotate(pos: vec2<f32>, angle: f32) -> vec2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    let rot = mat2x2(c, s, -s, c);
    return rot * pos;
}

// TODO: credits https://iquilezles.org/articles/distfunctions2d/

fn sd_circle(pos: vec2<f32>, radius: f32) -> f32 {
    return length(pos) - radius;
}

fn sd_box(pos: vec2<f32>, size: vec2<f32>) -> f32 {
    let dist = abs(pos) - size;
    return length(max(dist, vec2(0.0))) + min(max(dist.x, dist.y), 0.0);
}

fn sd_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

fn sd_rounded_box(pos: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    let q = abs(pos) - size + radius;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2(0.0))) - radius;
}

fn opUnion(d1: f32, d2: f32) -> f32 {
    return min(d1, d2);
}

fn opSubtraction(d1: f32, d2: f32) -> f32 {
    return max(-d1, d2);
}

fn opIntersection(d1: f32, d2: f32) -> f32 {
    return max(d1, d2);
}

fn opXor(d1: f32, d2: f32) -> f32 {
    return max(min(d1, d2), -max(d1, d2));
}
