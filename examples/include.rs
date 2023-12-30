use wgsl_plus::WgslWorkspace;

#[rustfmt::skip]
fn main() {
    let mut workspace = WgslWorkspace::from_memory("shaders", &[
        ("my-shader.wgsl", include_str!("shaders/my-shader.wgsl")),
        ("vertex.wgsl", include_str!("shaders/vertex.wgsl")),
    ]).unwrap();

    workspace.set_global_bool("USE_TANGENTS", false);

    let shader = workspace.get_shader("my-shader.wgsl").unwrap();

    println!("USE_TANGENTS = false");
    println!("{}", shader);
    
    workspace.set_global_bool("USE_TANGENTS", true);

    let shader = workspace.get_shader("my-shader.wgsl").unwrap();
    
    println!("USE_TANGENTS = true");
    println!("{}", shader);
}
