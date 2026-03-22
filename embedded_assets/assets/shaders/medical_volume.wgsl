#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

struct VolumeParams {
    window_low: f32,
    window_high: f32,
    step_size: f32,
    _padding: f32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> volume_params: VolumeParams;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> bounds_min: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> bounds_max: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var volume_texture: texture_3d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var volume_sampler: sampler;

fn ray_box_intersection(
    ray_origin: vec3<f32>,
    ray_dir: vec3<f32>,
    box_min: vec3<f32>,
    box_max: vec3<f32>,
) -> vec2<f32> {
    let inv_dir = 1.0 / ray_dir;
    let t0 = (box_min - ray_origin) * inv_dir;
    let t1 = (box_max - ray_origin) * inv_dir;
    let tsmaller = min(t0, t1);
    let tbigger = max(t0, t1);
    let t_enter = max(max(tsmaller.x, tsmaller.y), tsmaller.z);
    let t_exit = min(min(tbigger.x, tbigger.y), tbigger.z);
    return vec2(t_enter, t_exit);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let box_min = bounds_min.xyz;
    let box_max = bounds_max.xyz;
    let ray_origin = view.world_position;
    let ray_dir = normalize(in.world_position.xyz - ray_origin);
    let hit = ray_box_intersection(ray_origin, ray_dir, box_min, box_max);

    if hit.x > hit.y {
        discard;
    }

    let extent = box_max - box_min;
    let max_extent = max(max(extent.x, extent.y), extent.z);
    let step_length = max(max_extent * volume_params.step_size, 0.25);
    var t = max(hit.x, 0.0);
    var mip_value = 0.0;

    for (var i = 0u; i < 512u; i = i + 1u) {
        if t > hit.y {
            break;
        }
        let world_position = ray_origin + ray_dir * t;
        let uvw = clamp((world_position - box_min) / extent, vec3(0.0), vec3(1.0));
        let value = textureSampleLevel(volume_texture, volume_sampler, uvw, 0.0).r;
        mip_value = max(mip_value, value);
        t = t + step_length;
    }

    let windowed = clamp(
        (mip_value - volume_params.window_low) / max(volume_params.window_high - volume_params.window_low, 0.0001),
        0.0,
        1.0,
    );

    if windowed <= 0.001 {
        discard;
    }

    let color = vec3(windowed, windowed * 0.97, windowed * 0.92);
    return vec4(color, windowed);
}
