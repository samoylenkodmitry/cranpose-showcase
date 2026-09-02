#![allow(non_snake_case)]

use std::f32::consts::TAU;

use cranpose::prelude::*;
use cranpose_ui_graphics::{GraphicsLayer, RenderEffect, RuntimeShader};

use crate::model::{BodyClass, CelestialBody};
use crate::motion::AmbientMotion;

/// One WGSL module shared by all fourteen bodies: analytic ray-sphere
/// intersection and Lambert/specular lighting driven by a sun-direction
/// uniform, procedural terrain (rocky bodies) or latitude-banded flow noise
/// (gas giants), an independently rotating cloud/haze layer, a ring disc
/// intersected in the same space that casts its own shadow onto the sphere,
/// and atmospheric rim glow. Every body differs only by the uniform payload
/// `build_shader` fills in, so the renderer compiles this pipeline once for
/// the whole app instead of once per body.
const PLANET_WGSL: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn fullscreen_vs(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;
    let x = f32(i32(vertex_index & 1u) * 2 - 1);
    let y = f32(i32(vertex_index >> 1u) * 2 - 1);
    output.uv = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));
    output.position = vec4<f32>(x, y, 0.0, 1.0);
    return output;
}

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(1) @binding(0) var<uniform> u: array<vec4<f32>, 64>;

fn get_float(index: u32) -> f32 {
    return u[index / 4u][index % 4u];
}

fn get_vec3(index: u32) -> vec3<f32> {
    return vec3<f32>(get_float(index), get_float(index + 1u), get_float(index + 2u));
}

const CAM_DIST: f32 = 4.0;
const FOCAL: f32 = 3.0209;
const SPHERE_R: f32 = 0.78;
const OUTER_GLOW_R: f32 = 1.3;
const VIEW_TILT: f32 = 0.24;

fn rotate_x(v: vec3<f32>, a: f32) -> vec3<f32> {
    let c = cos(a);
    let s = sin(a);
    return vec3<f32>(v.x, v.y * c - v.z * s, v.y * s + v.z * c);
}

fn rotate_y(v: vec3<f32>, a: f32) -> vec3<f32> {
    let c = cos(a);
    let s = sin(a);
    return vec3<f32>(v.x * c + v.z * s, v.y, -v.x * s + v.z * c);
}

fn hash3(p: vec3<f32>) -> f32 {
    return fract(sin(dot(p, vec3<f32>(12.9898, 78.233, 37.719))) * 43758.5453);
}

fn noise3(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let w = f * f * (3.0 - 2.0 * f);
    let n000 = hash3(i + vec3<f32>(0.0, 0.0, 0.0));
    let n100 = hash3(i + vec3<f32>(1.0, 0.0, 0.0));
    let n010 = hash3(i + vec3<f32>(0.0, 1.0, 0.0));
    let n110 = hash3(i + vec3<f32>(1.0, 1.0, 0.0));
    let n001 = hash3(i + vec3<f32>(0.0, 0.0, 1.0));
    let n101 = hash3(i + vec3<f32>(1.0, 0.0, 1.0));
    let n011 = hash3(i + vec3<f32>(0.0, 1.0, 1.0));
    let n111 = hash3(i + vec3<f32>(1.0, 1.0, 1.0));
    let nx00 = mix(n000, n100, w.x);
    let nx10 = mix(n010, n110, w.x);
    let nx01 = mix(n001, n101, w.x);
    let nx11 = mix(n011, n111, w.x);
    let nxy0 = mix(nx00, nx10, w.y);
    let nxy1 = mix(nx01, nx11, w.y);
    return mix(nxy0, nxy1, w.z);
}

fn fbm3(p: vec3<f32>) -> f32 {
    var value = noise3(p) * 0.5;
    value = value + noise3(p * 2.02) * 0.25;
    value = value + noise3(p * 4.07) * 0.125;
    return value;
}

@fragment
fn effect_fs(input: VertexOutput) -> @location(0) vec4<f32> {
    let base = textureSample(input_texture, input_sampler, input.uv);
    let effect_rect = vec4<f32>(get_float(248u), get_float(249u), get_float(250u), get_float(251u));
    let size_px = effect_rect.zw;
    let half_min = 0.5 * min(size_px.x, size_px.y);
    if (half_min <= 0.0) {
        return base;
    }
    let tex_size = vec2<f32>(textureDimensions(input_texture));
    let local_px = input.uv * tex_size - effect_rect.xy;
    let center_px = 0.5 * size_px;
    var ndc = (local_px - center_px) / half_min;
    ndc.y = -ndc.y;

    let rotation_phase = get_float(0u);
    let cloud_phase = get_float(1u);
    let sun_dir = normalize(get_vec3(2u));
    let axial_tilt = get_float(5u);
    let body_class = get_float(6u);
    let has_ring = get_float(7u) > 0.5;
    let top_color = get_vec3(8u);
    let bottom_color = get_vec3(11u);
    let glow_color = get_vec3(14u);
    let surface_scale = max(get_float(17u), 0.001);
    let warp_strength = get_float(18u);
    let specular_strength = get_float(19u);
    let specular_power = max(get_float(20u), 1.0);
    let band_scale = get_float(21u);
    let ring_inner = get_float(22u);
    let ring_outer = get_float(23u);
    let ring_tilt = get_float(24u);
    let ring_color = get_vec3(25u);
    let seed = get_float(28u);
    let cloud_coverage = clamp(get_float(29u), 0.0, 1.0);
    let emissive_strength = get_float(30u);
    let rim_strength = get_float(31u);

    let ray_origin = rotate_x(vec3<f32>(0.0, 0.0, -CAM_DIST), VIEW_TILT);
    let ray_dir = normalize(rotate_x(vec3<f32>(ndc.x, ndc.y, FOCAL), VIEW_TILT));
    let view_dir = -ray_dir;

    let b = dot(ray_origin, ray_dir);
    let c = dot(ray_origin, ray_origin) - 1.0;
    let disc = b * b - c;
    let t_sphere = -b - sqrt(max(disc, 0.0));
    let hit_point = ray_origin + t_sphere * ray_dir;
    let normal = hit_point;

    let ndc_len = length(ndc);
    let r = ndc_len / SPHERE_R;
    let px = 1.5 / half_min;
    let sphere_coverage = 1.0 - smoothstep(SPHERE_R - px, SPHERE_R + px, ndc_len);
    let sphere_hit = disc >= 0.0 && t_sphere > 0.0 && sphere_coverage > 0.001;

    let ring_normal = normalize(vec3<f32>(0.0, cos(ring_tilt), sin(ring_tilt)));
    var ring_hit = false;
    var t_ring = 1.0e9;
    var ring_alpha = 0.0;
    var ring_lit_color = vec3<f32>(0.0, 0.0, 0.0);
    if (has_ring) {
        let denom = dot(ray_dir, ring_normal);
        if (abs(denom) > 1.0e-4) {
            let t = -dot(ray_origin, ring_normal) / denom;
            if (t > 0.0) {
                let p = ray_origin + t * ray_dir;
                let d = length(p);
                let edge = 0.035;
                let cover = smoothstep(ring_inner - edge, ring_inner + edge, d)
                    * smoothstep(ring_outer + edge, ring_outer - edge, d);
                if (cover > 0.001) {
                    ring_hit = true;
                    t_ring = t;
                    let band_uv = (d - ring_inner) / max(ring_outer - ring_inner, 0.001);
                    let banding = 0.55 + 0.45 * noise3(vec3<f32>(band_uv * 24.0 + seed, 0.0, 0.0));
                    let lit = 0.35 + 0.65 * abs(dot(ring_normal, sun_dir));
                    ring_lit_color = ring_color * banding * lit;
                    ring_alpha = cover * 0.94;
                }
            }
        }
    }

    var sphere_color = vec3<f32>(0.0, 0.0, 0.0);
    if (sphere_hit) {
        var n_obj = rotate_x(normal, -axial_tilt);
        n_obj = rotate_y(n_obj, -rotation_phase);

        var surface_color = bottom_color;
        if (body_class < 0.5) {
            let t = fbm3(n_obj * surface_scale + vec3<f32>(seed, seed, seed));
            let tn = clamp(t * (0.7 + warp_strength), 0.0, 1.0);
            surface_color = mix(bottom_color, top_color, smoothstep(0.28, 0.72, tn));
        } else if (body_class < 1.5) {
            var cn = rotate_x(normal, -axial_tilt);
            cn = rotate_y(cn, -cloud_phase);
            let warp = fbm3(cn * 1.8 + vec3<f32>(seed, seed, seed)) * warp_strength;
            let band = sin(cn.y * band_scale + warp * 5.0) * 0.5 + 0.5;
            surface_color = mix(bottom_color, top_color, smoothstep(0.12, 0.88, band));
        } else {
            let flow = fbm3(n_obj * surface_scale + vec3<f32>(cloud_phase * 0.6, cloud_phase * 0.4, seed));
            surface_color = mix(bottom_color, top_color, clamp(flow * 1.4, 0.0, 1.0)) * max(emissive_strength, 1.0);
        }

        if (cloud_coverage > 0.005 && body_class < 1.5) {
            var cln = rotate_x(normal, -axial_tilt);
            cln = rotate_y(cln, -cloud_phase);
            let cval = fbm3(cln * (surface_scale * 1.6 + 3.0) + vec3<f32>(seed * 1.7, seed * 1.7, seed * 1.7));
            let cmask = smoothstep(1.0 - cloud_coverage, 1.15 - cloud_coverage, cval);
            let cloud_tint = mix(vec3<f32>(1.0, 1.0, 1.0), glow_color, 0.25 + 0.35 * cloud_coverage);
            surface_color = mix(surface_color, cloud_tint, cmask * 0.92);
        }

        var ring_shadow = 0.0;
        if (has_ring) {
            let denom_s = dot(sun_dir, ring_normal);
            if (abs(denom_s) > 1.0e-4) {
                let t_s = -dot(hit_point, ring_normal) / denom_s;
                if (t_s > 0.0) {
                    let sp = hit_point + t_s * sun_dir;
                    let ds = length(sp);
                    if (ds >= ring_inner && ds <= ring_outer) {
                        ring_shadow = 0.85;
                    }
                }
            }
        }

        if (body_class < 1.5) {
            let wrap = 0.18;
            let ndotl = dot(normal, sun_dir);
            var diffuse = clamp((ndotl + wrap) / (1.0 + wrap), 0.0, 1.0);
            diffuse = diffuse * (1.0 - ring_shadow);
            let ambient_fill = 0.05;
            var lit = surface_color * (ambient_fill + diffuse * (1.0 - ambient_fill));

            let halfv = normalize(sun_dir + view_dir);
            let spec = pow(max(dot(normal, halfv), 0.0), specular_power) * specular_strength * diffuse;
            lit = lit + vec3<f32>(spec, spec, spec);

            let rim = pow(1.0 - clamp(dot(normal, view_dir), 0.0, 1.0), 4.0) * rim_strength * 1.4;
            lit = lit + glow_color * rim;

            sphere_color = lit;
        } else {
            let rim = pow(1.0 - clamp(dot(normal, view_dir), 0.0, 1.0), 2.0) * rim_strength;
            let hot = surface_color + glow_color * rim * 1.5;
            sphere_color = hot / (vec3<f32>(1.0, 1.0, 1.0) + hot * 0.35);
        }
    }

    var local_color = vec3<f32>(0.0, 0.0, 0.0);
    var local_alpha = 0.0;
    if (sphere_hit && ring_hit) {
        if (t_ring < t_sphere) {
            local_color = mix(sphere_color, ring_lit_color, ring_alpha);
        } else {
            local_color = sphere_color;
        }
        local_alpha = sphere_coverage;
    } else if (sphere_hit) {
        local_color = sphere_color;
        local_alpha = sphere_coverage;
    } else if (ring_hit) {
        local_color = ring_lit_color;
        local_alpha = ring_alpha;
    } else if (r > 1.0 && r < OUTER_GLOW_R && rim_strength > 0.005) {
        let g = clamp(1.0 - (r - 1.0) / (OUTER_GLOW_R - 1.0), 0.0, 1.0);
        local_color = glow_color;
        local_alpha = rim_strength * pow(g, 2.2) * 0.55;
    }

    let edge_fade = 1.0 - smoothstep(0.90, 1.0, ndc_len);
    local_alpha = local_alpha * edge_fade;

    let out_a = local_alpha + base.a * (1.0 - local_alpha);
    let out_rgb = local_color * local_alpha + base.rgb * (1.0 - local_alpha);
    return vec4<f32>(out_rgb, out_a);
}
"#;

mod uniform {
    pub const ROTATION_PHASE: usize = 0;
    pub const CLOUD_PHASE: usize = 1;
    pub const SUN_DIR: usize = 2;
    pub const AXIAL_TILT: usize = 5;
    pub const BODY_CLASS: usize = 6;
    pub const HAS_RING: usize = 7;
    pub const TOP_COLOR: usize = 8;
    pub const BOTTOM_COLOR: usize = 11;
    pub const GLOW_COLOR: usize = 14;
    pub const SURFACE_SCALE: usize = 17;
    pub const WARP_STRENGTH: usize = 18;
    pub const SPECULAR_STRENGTH: usize = 19;
    pub const SPECULAR_POWER: usize = 20;
    pub const BAND_SCALE: usize = 21;
    pub const RING_INNER: usize = 22;
    pub const RING_OUTER: usize = 23;
    pub const RING_TILT: usize = 24;
    pub const RING_COLOR: usize = 25;
    pub const SEED: usize = 28;
    pub const CLOUD_COVERAGE: usize = 29;
    pub const EMISSIVE_STRENGTH: usize = 30;
    pub const RIM_STRENGTH: usize = 31;
}

fn rgb01(c: (u8, u8, u8)) -> (f32, f32, f32) {
    (c.0 as f32 / 255.0, c.1 as f32 / 255.0, c.2 as f32 / 255.0)
}

fn set_vec3(shader: &mut RuntimeShader, index: usize, v: (f32, f32, f32)) {
    shader.set_float(index, v.0);
    shader.set_float(index + 1, v.1);
    shader.set_float(index + 2, v.2);
}

fn body_class_index(class: BodyClass) -> f32 {
    match class {
        BodyClass::Rocky => 0.0,
        BodyClass::GasGiant => 1.0,
        BodyClass::Star => 2.0,
    }
}

/// The sun direction every sphere shares: a fixed upper-left key light with
/// a small perpetual wobble, so a resting card still reads as alive the way
/// the old baked highlight's drift did — except this one actually moves the
/// light, not a painted-on specular blob.
fn sun_direction(sheen: f32) -> (f32, f32, f32) {
    let wobble = (sheen - 0.5) * 0.6;
    let x = -0.55 + wobble * 0.25;
    let y = 0.6;
    let z = -0.55;
    let len = (x * x + y * y + z * z).sqrt();
    (x / len, y / len, z / len)
}

fn build_shader_template(body: &CelestialBody) -> RuntimeShader {
    let mut shader = RuntimeShader::new(PLANET_WGSL);
    let axial_tilt_rad = body.axial_tilt_deg.to_radians();

    shader.set_float(uniform::ROTATION_PHASE, 0.0);
    shader.set_float(uniform::CLOUD_PHASE, 0.0);
    set_vec3(&mut shader, uniform::SUN_DIR, sun_direction(0.0));
    shader.set_float(uniform::AXIAL_TILT, axial_tilt_rad);
    shader.set_float(uniform::BODY_CLASS, body_class_index(body.body_class));
    shader.set_float(uniform::HAS_RING, if body.has_ring { 1.0 } else { 0.0 });
    set_vec3(&mut shader, uniform::TOP_COLOR, rgb01(body.top_color));
    set_vec3(&mut shader, uniform::BOTTOM_COLOR, rgb01(body.bottom_color));
    set_vec3(&mut shader, uniform::GLOW_COLOR, rgb01(body.glow_color));
    shader.set_float(uniform::SURFACE_SCALE, body.surface_scale);
    shader.set_float(uniform::WARP_STRENGTH, body.warp_strength);
    shader.set_float(uniform::SPECULAR_STRENGTH, body.specular_strength);
    shader.set_float(uniform::SPECULAR_POWER, body.specular_power);
    shader.set_float(uniform::BAND_SCALE, body.band_scale);
    shader.set_float(uniform::RING_INNER, body.ring_inner_frac);
    shader.set_float(uniform::RING_OUTER, body.ring_outer_frac);
    shader.set_float(uniform::RING_TILT, body.ring_tilt_deg.to_radians());
    set_vec3(&mut shader, uniform::RING_COLOR, rgb01(body.ring_color));
    shader.set_float(uniform::SEED, body.seed);
    shader.set_float(uniform::CLOUD_COVERAGE, body.cloud_coverage);
    shader.set_float(uniform::EMISSIVE_STRENGTH, body.emissive_strength);
    shader.set_float(uniform::RIM_STRENGTH, body.rim_strength);
    shader
}

fn build_shader(
    template: &RuntimeShader,
    rotation_phase: f32,
    cloud_phase: f32,
    sun_dir: (f32, f32, f32),
) -> RuntimeShader {
    let mut shader = template.clone();
    shader.set_float(uniform::ROTATION_PHASE, rotation_phase);
    shader.set_float(uniform::CLOUD_PHASE, cloud_phase);
    set_vec3(&mut shader, uniform::SUN_DIR, sun_dir);
    shader
}

fn planet_layer(shader: RuntimeShader) -> GraphicsLayer {
    GraphicsLayer {
        render_effect: Some(RenderEffect::runtime_shader(shader)),
        ..Default::default()
    }
}

/// A real 3D-shaded sphere for one celestial body: analytic ray-sphere
/// intersection, a Lambert-plus-specular day/night terminator driven by a
/// sun-direction uniform, procedural terrain or band noise, an optional
/// cloud/haze layer that spins at its own rate, a ring system that casts
/// its own shadow onto the sphere, and atmospheric rim glow — all from one
/// shared runtime WGSL shader parameterized per body. `ambient` drives the
/// rotation (`drift`) and the light's idle wobble (`sheen`) so a resting
/// card still reads as alive.
#[composable]
pub fn PlanetSphere(modifier: Modifier, body: &'static CelestialBody, ambient: AmbientMotion) {
    let template = rememberKeyed(body.name, |_| build_shader_template(body));
    Box(
        modifier.graphics_layer(move || {
            let rotation = ambient.planet_rotation();
            let shader = build_shader(
                &template,
                rotation * body.rotation_turns * TAU,
                rotation * body.cloud_turns * TAU,
                sun_direction(ambient.sheen()),
            );
            planet_layer(shader)
        }),
        BoxSpec::default(),
        || {},
    );
}

#[cfg(test)]
mod tests {
    use super::{planet_layer, PLANET_WGSL};
    use cranpose_ui_graphics::RuntimeShader;

    #[test]
    fn transparent_planet_pixels_are_not_rendered_as_a_backdrop_effect() {
        let layer = planet_layer(RuntimeShader::new(PLANET_WGSL));

        assert!(layer.render_effect.is_some());
        assert!(layer.backdrop_effect.is_none());
    }
}
