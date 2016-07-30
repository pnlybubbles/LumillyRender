pub fn clamp(x: f64) -> f64 {
  if x < 0.0 {
    return 0.0;
  }
  if x > 1.0 {
    return 1.0;
  }
  return x
}

pub fn to_int(x: f64) -> u8 {
  return (clamp(x).powf(1.0 / 2.2) * 255.0) as u8
}
