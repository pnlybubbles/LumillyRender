use ray::Ray;
use vector::*;
use vector3::Vector3;

pub trait Sky {
  fn radiance(&self, &Ray) -> Vector3<f64>;
}

pub struct UniformSky {
  pub emission: Vector3<f64>,
}

impl Sky for UniformSky {
  fn radiance(&self, _: &Ray) -> Vector3<f64> {
    self.emission
  }
}

pub struct SimpleSky {
  pub meridian: Vector3<f64>,
  pub horizon: Vector3<f64>
}

impl Sky for SimpleSky {
  fn radiance(&self, ray: &Ray) -> Vector3<f64> {
    let weight = ray.direction.dot(Vector3::new(0.0, 1.0, 0.0)).abs();
    self.meridian * weight + self.horizon * (1.0 - weight)
  }
}

