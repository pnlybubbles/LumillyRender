extern crate image;

use ray::Ray;
use vector::*;
use vector3::Vector3;
use constant::*;
use std::fs::File;
use std::io::BufReader;

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

pub struct IBLSky {
  hdr_image: Vec<image::Rgb<f32>>,
  height: usize,
  longitude_offset: usize,
}

impl IBLSky {
  pub fn new(path: &'static str, longitude_offset: usize) -> IBLSky {
    println!("loading hdr image...");
    let image_file = File::open(path).unwrap();
    let decoder = image::hdr::HDRDecoder::new(BufReader::new(image_file)).unwrap();
    println!("{:?}", decoder.metadata());
    let height = decoder.metadata().height as usize;
    let image = decoder.read_image_hdr().unwrap();
    IBLSky {
      hdr_image: image,
      height: height,
      longitude_offset: longitude_offset,
    }
  }
}

impl Sky for IBLSky {
  fn radiance(&self, ray: &Ray) -> Vector3<f64> {
    let interval = 0.04;
    let theta = (ray.direction.y).acos();
    let phi_pr = (ray.direction.z / ray.direction.x).atan();
    let phi = if ray.direction.x < 0.0 { phi_pr + PI } else { phi_pr } + PI / 2.0;
    let u_ = phi / (2.0 * PI);
    let v_ = theta / PI;
    if u_ < 0.0 || u_ > 1.0 || v_ < 0.0 || v_ > 1.0 {
      println!("{:?} {:?}", u_, v_);
      panic!("oh! no!");
    }
    let u = (phi / (2.0 * PI) / interval * 2.0).floor();
    let v = (theta / PI / interval).floor();
    let p = (u + v) % 2.0;
    return Vector3::new(0.1, 0.1, 0.1) + Vector3::new(0.9, 0.9, 0.9) * (1.0 - p);
  }
}

