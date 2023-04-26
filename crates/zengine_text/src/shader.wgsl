struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@vertex
fn vs_main(in_vert: VertexInput) -> VertexOutput {
    let NORMALIZE = mat4x4(
        1.,
        0.,
        0.,
        0.,
        0.,
        -1.,
        0.,
        0.,
        0.,
        0.,
        -1.,
        0.,
        0.,
        0.,
        0.,
        1.
    );

    var vert_output: VertexOutput;
    vert_output.clip_position = camera.view_proj * NORMALIZE * in_vert.position;
    vert_output.color = in_vert.color;
    vert_output.tex_coords = vec2<f32>(in_vert.tex_coords) / vec2<f32>(textureDimensions(t_diffuse).xy);

    return vert_output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color * textureSample(t_diffuse, s_diffuse, in.tex_coords).x;
}
