struct VertexOut {
    @location(0) color: vec4f,
    @builtin(position) position: vec4f,
};

@group(0) @binding(0)
var<uniform> matrix: mat4x4f;

var<private> v_color: vec4f = vec4f(1.0, 0.0, 0.0, 1.0);

@vertex
fn vs_main(@location(0) v_position: vec4f) -> VertexOut {
    var out: VertexOut;

    out.position = matrix * v_position;
    out.color = v_color;

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4f {
    return in.color;
}
