struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tex_coords: vec2<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) position: vec3<f32>,
    @location(2) normal: vec3<f32>,
};

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
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
