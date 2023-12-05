#import bevy_sprite::mesh2d_vertex_output::VertexOutput
fn sd_circle(pos: vec2<f32>, radius: f32) -> f32 {
    return length(pos) - radius;
}

fn sd_box(pos: vec2<f32>, size: vec2<f32>) -> f32 {
    let dist = abs(pos) - size;
    return length(max(dist,vec2(0.0))) + min(max(dist.x,dist.y),0.0);
}

fn sd_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p-a;
    let ba = b-a;
    let h = clamp( dot(pa,ba)/dot(ba,ba), 0.0, 1.0 );
    return length( pa - ba*h );
}

fn sd_rounded_box(pos: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    let q = abs(pos) - size + radius;
    return min(max(q.x,q.y),0.0) + length(max(q,vec2(0.0))) - radius;
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


fn sd_color_border_highlighted(d: f32, color: vec3<f32>, color_highlight: vec3<f32>, distance_cursor: f32) -> vec4<f32> {
    let border = 5.; //px
    let border_color = mix(vec3(0.5), color_highlight, 1. - distance_cursor);
    let new_color = select(color, border_color, abs(d) <= border);
    // return vec4(new_color, step(-border, -d)); // Symmetric border
    return vec4(new_color, step(0., -d)); // Inward border
}

fn sd_color_debug(d: f32, color: vec3<f32>) -> vec4<f32> {
	// coloring
    var col = vec3(0.65,0.85,1.0);
    if (d > 0.) {
        col = vec3(0.9,0.6,0.3);
    }
    col *= 1.0 - exp(-0.05*abs(d));
	col *= 0.8 + 0.2*cos(2.*3.1415*d/10.);
	col = mix( col, vec3(1.0), 1.0-smoothstep(0.0,2.,abs(d)) );

	return vec4(col,1.0);
}

fn opUnion(d1: f32, d2: f32) -> f32 {
    return min(d1,d2);
}
fn opSubtraction(d1: f32, d2: f32) -> f32 {
    return max(-d1,d2);
}
fn opIntersection(d1: f32, d2: f32) -> f32 {
    return max(d1,d2);
}
fn opXor(d1: f32, d2: f32) -> f32 {
    return max(min(d1,d2),-max(d1,d2));
}

fn rotate(pos: vec2<f32>, angle: f32) -> vec2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    let rot = mat2x2(c, s, -s, c);
    return rot * pos;
}

@group(1) @binding(0) var<uniform> material_color: vec4<f32>;

const PI: f32 = 3.1415;

@fragment
fn fragment_side(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let pos = (mesh.uv.xy * 2. - 1.) * 150.;
    let grid = vec2<u32>(pos / 10.);
    let grid_sum = grid.x + grid.y;
    let checker = (grid_sum % 2u) == 1u;
    var color = select(vec4(0.5, 0.5, 0.5, 1.), vec4(1.), checker);
    // Head
    var d = sd_circle(pos + vec2(60., -10.), 30.);
    // Thorax
    d = opUnion(d, sd_rounded_box(pos, vec2(50., 25.), 32.));
    // Waist
    d = opUnion(d, sd_circle(pos + vec2(-55., 0.), 20.));
    // Bottom
    d = opUnion(d, sd_rounded_box(rotate(pos + vec2(-90., -10.), -PI/6.), vec2(40., 27.), 40.));
    // Antenna
    d = opUnion(d, sd_segment(pos, vec2(-100., -20.), vec2(-125., 65.)));
    d = opUnion(d, sd_segment(pos, vec2(-100., -20.), vec2(-60., 10.)));
    // Leg front
    d = opUnion(d, sd_segment(pos, vec2(-10., -40.), vec2(-40., 65.)));
    d = opUnion(d, sd_segment(pos, vec2(-10., -40.), vec2(10., 20.)));
    // Leg middle
    d = opUnion(d, sd_segment(pos, vec2(10., -40.), vec2(0., 65.)));
    d = opUnion(d, sd_segment(pos, vec2(10., -40.), vec2(30., 20.)));
    // Leg back
    d = opUnion(d, sd_segment(pos, vec2(30., -40.), vec2(60., 65.)));
    d = opUnion(d, sd_segment(pos, vec2(30., -40.), vec2(30., 20.)));
    // return sd_color_debug(d, color.xyz);
    return sd_color_sharp(d - 3., vec3(0.));
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var pos = (mesh.uv.xy * 2. - 1.) * 150.;
    pos.x = abs(pos.x);
    let grid = vec2<u32>(pos / 10.);
    let grid_sum = grid.x + grid.y;
    let checker = (grid_sum % 2u) == 1u;
    var color = select(vec4(0.5, 0.5, 0.5, 1.), vec4(1.), checker);
    // Head
    var d = sd_circle(pos + vec2(0., 60.), 30.);
    // Thorax
    d = opUnion(d, sd_rounded_box(pos, vec2(25., 50.), 32.));
    // Waist
    d = opUnion(d, sd_circle(pos + vec2(0., -55.), 20.));
    // Bottom
    d = opUnion(d, sd_rounded_box(pos + vec2(0., -90.), vec2(27., 40.), 40.));
    // Antenna
    d = opUnion(d, sd_segment(pos, vec2(50., -110.), vec2(25., -145.)));
    d = opUnion(d, sd_segment(pos, vec2(50., -110.), vec2(10., -60.)));
    // Leg front
    d = opUnion(d, sd_segment(pos, vec2(40., -30.), vec2(80., -40.)));
    d = opUnion(d, sd_segment(pos, vec2(40., -30.), vec2(20., 10.)));
    // Leg middle
    d = opUnion(d, sd_segment(pos, vec2(40., 0.), vec2(85., 15.)));
    d = opUnion(d, sd_segment(pos, vec2(40., 0.), vec2(20., 20.)));
    // Leg back
    d = opUnion(d, sd_segment(pos, vec2(40., 30.), vec2(75., 80.)));
    d = opUnion(d, sd_segment(pos, vec2(40., 30.), vec2(20., 30.)));

    // return sd_color_debug(d, color.xyz);
    return sd_color_sharp(d - 3., vec3(0.));
}
