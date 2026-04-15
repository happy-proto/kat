struct Theme {
  active: bool,
  accent: vec4<f32>,
};

@fragment
fn fs_main() -> @location(0) vec4<f32> {
  return vec4<f32>(0.74, 0.58, 0.98, 1.0);
}
