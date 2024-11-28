struct VertexIn {
    @location(0) position: vec4f,
    @location(1) uv: vec2f,
};

struct VertexOut {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
};

struct MVP {
    model: mat4x4f,
    view: mat4x4f,
    projection: mat4x4f,
};

@group(0) @binding(0)
var<uniform> mvp: MVP;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

// var<private> v_color: vec4f = vec4f(1.0, 0.0, 0.0, 1.0);

@vertex
fn vs_main(v: VertexIn) -> VertexOut {
    var out: VertexOut;

    out.position = mvp.projection * mvp.view * mvp.model * v.position;
    out.uv = v.uv;

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4f {
    return textureSample(t_diffuse, s_diffuse, in.uv);
}
