struct VertexOut {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
};

struct MVP {
    model: mat4x4f,
    view: mat4x4f,
    projection: mat4x4f,
}

@group(0) @binding(0)
var<uniform> mvp: MVP;

var<private> v_color: vec4f = vec4f(1.0, 0.0, 0.0, 1.0);

@vertex
fn vs_main(@location(0) v_position: vec4f) -> VertexOut {
    var out: VertexOut;

    out.position = mvp.projection * mvp.view * mvp.model * out.world_position;
    out.color = v_color;

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4f {
    return in.color;
}
