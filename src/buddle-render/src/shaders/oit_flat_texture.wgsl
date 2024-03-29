struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tex_coords: vec2<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) position: vec3<f32>,
    @location(3) normal: vec3<f32>
};

struct FragmentOutput {
    @location(0) accum: vec4<f32>,
    @location(1) reveal: f32,
}

struct CameraData {
    view_matrix: mat4x4<f32>,
    proj_matrix: mat4x4<f32>,
    position: vec3<f32>,
};

struct ModelMatrices {
    mvp: mat4x4<f32>,
    model_matrix: mat4x4<f32>,
    normal_matrix: mat4x4<f32>
};

@group(0) @binding(0)
var<uniform> camera: CameraData;

@group(1) @binding(0)
var<uniform> model: ModelMatrices;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = model.mvp * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    out.tex_coords = in.tex_coords;
    out.position = (model.model_matrix * vec4<f32>(in.position, 1.0)).xyz;
    out.normal = in.normal;
    return out;
}

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords) * vec4<f32>(in.color, 1.0);

    var weight: f32 = clamp(pow(min(1.0, color.a * 10.0) + 0.01, 3.0) * 1e8 *
                         pow(1.0 - in.clip_position.z * 0.9, 3.0), 1e-2, 3e3);

    return FragmentOutput(
        vec4<f32>(color.rgb * color.a, color.a) * weight,
        color.a
    );
}
