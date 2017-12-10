extern crate image;

use ray::Ray;
use math::vector::*;
use constant::*;
use std::fs::File;
use std::io::BufReader;

pub trait Sky {
  fn radiance(&self, &Ray) -> Vector3;
}

pub struct UniformSky {
  pub emission: Vector3,
}

impl Sky for UniformSky {
  fn radiance(&self, _: &Ray) -> Vector3 {
    self.emission
  }
}

pub struct SimpleSky {
  pub meridian: Vector3,
  pub horizon: Vector3,
}

impl Sky for SimpleSky {
  fn radiance(&self, ray: &Ray) -> Vector3 {
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
  fn radiance(&self, ray: &Ray) -> Vector3 {
    let theta = (ray.direction.y).acos();
    let phi_pr = (ray.direction.z / ray.direction.x).atan();
    let phi = if ray.direction.x < 0.0 {
      phi_pr + PI
    } else {
      phi_pr
    } + PI / 2.0;
    let x = (self.height as f32 * phi / PI).round() as usize + self.longitude_offset;
    let y = (self.height as f32 * theta / PI).round() as usize;
    let index = y * self.height * 2 +
      if x > self.height * 2 {
        x % (self.height * 2)
      } else {
        x
      };
    let color = self.hdr_image[index];
    return Vector3::new(
      color.data[0] as f32,
      color.data[1] as f32,
      color.data[2] as f32,
    );
  }
}
