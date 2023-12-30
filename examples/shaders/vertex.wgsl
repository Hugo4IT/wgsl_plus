struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    //:if USE_TANGENTS
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    //:end
};

struct Vertex {
    @builtin(position) clip_position: vec4<f32>,
};