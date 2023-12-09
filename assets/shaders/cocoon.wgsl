#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::view,
}

#ifdef TONEMAP_IN_SHADER
#import bevy_core_pipeline::tonemapping
#endif

struct CocoonMaterial {
    is_clue: u32,
};
const SHEDDING_MATERIAL_IS_CLUE_BIT: u32 = 1u;

@group(1) @binding(0) var<uniform> material: CocoonMaterial;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    var pos: vec2<f32> = (mesh.uv.xy * 2. - 1.) * 150.;
    pos.y = pos.y - 100.;
    let color_fill = vec3(0.6);
    let color_border = vec3(0.1);
    let color_halo = vec3(1., 0.6, 0.);
    let color_spot = vec3(0.03);

    var d = sd_capsule(vec2(pos.y, pos.x), 50., 140.);
    var d_spot = sd_circle(vec2(pos.x + 65., pos.y), 20.);
    let spot = sd_color_smooth(d_spot, color_spot);
    // TODO: add texture
    if (material.is_clue & SHEDDING_MATERIAL_IS_CLUE_BIT) != 0u {
        let fill = sd_color_border(d, 10., color_fill, color_border);
        let halo = sd_color_halo(d, 50., color_halo);
        return blend_colors(halo, blend_colors(fill, spot));
    } else {
        let fill = sd_color_border(d, 10., color_fill, color_border);
        return blend_colors(fill, spot);
    }
}

fn blend_colors(dest: vec4<f32>, source: vec4<f32>) -> vec4<f32> {
    return source.a * source.rbga + (1. - source.a) * dest.rgba;
}

fn sd_color_border(d: f32, border: f32, color: vec3<f32>, border_color: vec3<f32>) -> vec4<f32> {
    let new_color = select(color, border_color, abs(d) <= border);
    return vec4(new_color, step(0., -d));
}

fn sd_color_halo(d: f32, width: f32, color: vec3<f32>) -> vec4<f32> {
    let x = pow(d / width, 0.3);
    let alpha = smoothstep(1., 0., x);
    return vec4(color, alpha);
}

fn sd_color_smooth(d: f32, color: vec3<f32>) -> vec4<f32> {
    let alpha = min(1.0, pow(1.0 - d / 400., 400.));
    return vec4(color, alpha);
}

// TODO: credits 

fn sd_circle(pos: vec2<f32>, radius: f32) -> f32 {
    return length(pos) - radius;
}

fn sd_capsule(pos: vec2<f32>, radius: f32, height: f32) -> f32 {
    let pos_abs = vec2(abs(pos.x), pos.y);
    if pos_abs.y < -height / 2. {return length(pos_abs - vec2(0., -height / 2.)) - radius;}
    if pos_abs.y > height / 2. {return length(pos_abs - vec2(0., height / 2.)) - radius;}
    return pos_abs.x - radius;
}