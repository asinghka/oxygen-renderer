@vertex
fn vertex_shader(
    @builtin(vertex_index) in_vertex_index: u32
) -> @builtin(position) vec4<f32> {
      var positions = array<vec2f, 3>(
          vec2f(-0.5, -0.5),
          vec2f( 0.5, -0.5),
          vec2f( 0.0,  0.5),
      );

      return vec4<f32>(positions[in_vertex_index], 1.0, 1.0);
}

@fragment
fn fragment_shader() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
