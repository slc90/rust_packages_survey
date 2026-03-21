#import bevy_pbr::forward_io::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> render_params: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> bounds_min: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> bounds_max: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var volume_texture: texture_3d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var volume_sampler: sampler;

fn intersect_box(ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> vec2<f32> {
	let inv_dir = 1.0 / ray_dir;
	let t0 = (bounds_min.xyz - ray_origin) * inv_dir;
	let t1 = (bounds_max.xyz - ray_origin) * inv_dir;
	let t_min = min(t0, t1);
	let t_max = max(t0, t1);
	let near_t = max(max(t_min.x, t_min.y), t_min.z);
	let far_t = min(min(t_max.x, t_max.y), t_max.z);
	return vec2<f32>(near_t, far_t);
}

fn sample_volume(world_pos: vec3<f32>) -> f32 {
	let uvw = (world_pos - bounds_min.xyz) / max(bounds_max.xyz - bounds_min.xyz, vec3<f32>(0.0001));
	return textureSampleLevel(volume_texture, volume_sampler, clamp(uvw, vec3<f32>(0.0), vec3<f32>(1.0)), 0.0).r;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
	let value = sample_volume(in.world_position.xyz);
	if value < render_params.x || value > render_params.y {
		discard;
	}

	let intensity = smoothstep(render_params.x, render_params.y, value);
	return vec4<f32>(vec3<f32>(intensity), intensity);
}
