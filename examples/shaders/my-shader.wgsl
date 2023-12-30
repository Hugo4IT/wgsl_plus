//:include vertex.wgsl

//:const USE_TANGENTS

@vertex
fn vertex_main(in: VertexInput) -> Vertex {
    var out: Vertex;
    out.clip_position = in.position;
    return out;
}