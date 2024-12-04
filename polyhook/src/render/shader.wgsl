struct VertexIn {
    @location(0) position: vec4f,
    @location(1) uv: vec2f,
    @location(2) normal: vec3f,
    @location(3) tangent: vec3f,
    @location(4) bitangent: vec3f,
};

struct VertexOut {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
    @location(1) tangent_position: vec3f,
    @location(2) tangent_light_dir: vec3f,
};

struct MVP {
    model: mat4x4f,
    view: mat4x4f,
    projection: mat4x4f,
    normal: mat3x3f,
};

@group(0) @binding(0)
var<uniform> mvp: MVP;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(2)
var t_normal: texture_2d<f32>;
@group(1) @binding(3)
var s_normal: sampler;

var<private> ambient_factor: f32 = 0.5;
var<private> light_dir: vec3f = vec3f(-1, -1, -1);

@vertex
fn vs_main(v: VertexIn) -> VertexOut {
    var out: VertexOut;

    let world_normal = normalize(mvp.normal * v.normal);
    let world_tangent = normalize(mvp.normal * v.tangent);
    let world_bitangent = normalize(mvp.normal * v.bitangent);
    let tangent_matrix = transpose(mat3x3f(
        world_tangent,
        world_bitangent,
        world_normal,
    ));

    let world_position = mvp.model * v.position;

    out.position = mvp.projection * mvp.view * world_position;
    out.uv = v.uv;
    out.tangent_position = tangent_matrix * world_position.xyz;
    out.tangent_light_dir = tangent_matrix * light_dir.xyz;

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4f {
    let tex_diffuse = textureSample(t_diffuse, s_diffuse, in.uv);
    // let tex_diffuse = vec4f(1.0);
    let tex_normal = textureSample(t_normal, s_normal, in.uv);

    let tangent_normal = tex_normal.xyz * 2.0 - 1.0;
    
    let diffuse_factor = max(dot(tangent_normal, normalize(in.tangent_light_dir)), 0.0);
    return min(diffuse_factor + ambient_factor, 1.0) * tex_diffuse;
}
