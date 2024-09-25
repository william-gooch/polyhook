struct VertexOut {
    @location(0) color: vec4f,
    @builtin(position) position: vec4f,
};

@group(0) @binding(0)
var<uniform> matrix: mat4x4f;

var<private> v_positions: array<vec3f, 3> = array<vec3f, 3>(
    vec3f(0.0, 1.0, 0.0),
    vec3f(1.0, -1.0, 0.0),
    vec3f(-1.0, -1.0, 0.0),
);

var<private> v_colors: array<vec4f, 3> = array<vec4f, 3>(
    vec4f(1.0, 0.0, 0.0, 1.0),
    vec4f(0.0, 1.0, 0.0, 1.0),
    vec4f(0.0, 0.0, 1.0, 1.0),
);

@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> VertexOut {
    var out: VertexOut;

    out.position = matrix * vec4f(v_positions[v_idx], 1.0);
    out.color = v_colors[v_idx];

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4f {
    return in.color;
}
