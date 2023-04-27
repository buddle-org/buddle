struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tex_coords: vec2<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    return VertexOutput(vec4<f32>(in.position.xyz, 1.0), in.tex_coords);
}

@group(0) @binding(0)
var t_accum: texture_2d<f32>;
@group(0) @binding(1)
var s_accum: sampler;

@group(1) @binding(0)
var t_reveal: texture_2d<f32>;
@group(1) @binding(1)
var s_reveal: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var accumulation: vec4<f32> = textureSample(t_accum, s_accum, in.tex_coords);
    var revealage: f32 = textureSample(t_reveal, s_reveal, in.tex_coords).r;

    if revealage == 1.0 {
        discard;
    }

    var average_color: vec3<f32> = accumulation.rgb / max(accumulation.a, 0.00001);

    return vec4<f32>(average_color, 1.0 - revealage);
}
