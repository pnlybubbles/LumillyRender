extern crate rand;

use vector::Vector;
use ray::Ray;
use constant::*;

#[derive(Debug, Copy, Clone, Default)]
pub struct Camera {
  pub position: Vector,
  pub direction: Vector,
  pub height: usize,
  pub width: usize,
  pub screen_height: f64,
  pub screen_width: f64,
  pub focus_distance: f64,
  pub lens_radius: f64,
  pub sensor_sensitivity: f64,
  pub forward: Vector,
  pub up: Vector,
  pub right: Vector,
  direction_distance: f64,
  sensor_pixel_area: f64,
}

impl Camera {
  pub fn new(position: Vector, direction: Vector, height: usize, width: usize, screen_height: f64, screen_width: f64, focus_distance: f64, lens_radius: f64, sensor_sensitivity: f64) -> Camera {
    let direction_distance = direction.len();
    // カメラの向きを基準とした正規直交基底をつくる
    let forward = direction.norm();
    let right = forward.cross(if forward.y.abs() < 1.0 - EPS { Vector::new(0.0, 1.0, 0.0) } else { Vector::new(1.0, 0.0, 0.0) });
    let up = right.cross(forward);
    let sensor_pixel_area = (screen_height / height as f64) * (screen_width / width as f64);
    Camera {
      position: position,
      height: height,
      width: width,
      screen_height: screen_height,
      screen_width: screen_width,
      direction: direction,
      focus_distance: focus_distance,
      lens_radius: lens_radius,
      sensor_sensitivity: sensor_sensitivity * direction_distance * direction_distance / (sensor_pixel_area * PI * lens_radius * lens_radius),
      forward: forward,
      right: right,
      up: up,
      direction_distance: direction_distance,
      sensor_pixel_area: sensor_pixel_area,
    }
  }

  fn get_sensor_point(self, top: usize, left: usize) -> Vector {
    // イメージセンサー1ピクセル内の点を取得
    let x = ((left as f64 + rand::random::<f64>() - 0.5) / self.width as f64 - 0.5) * self.screen_width;
    let y = ((top as f64 + rand::random::<f64>() - 0.5) / self.height as f64 - 0.5) * self.screen_height;
    return (self.right * x + self.up * y + self.forward * (-1.0 * self.direction_distance)) + self.position;
  }

  fn get_lens_point(self) -> Vector {
    // 円形レンズ内に一様分布
    let r1 = 2.0 * PI * rand::random::<f64>();
    let r2 = rand::random::<f64>().sqrt() * self.lens_radius;
    return self.right * (r1.cos() * r2) + self.up * (r1.sin() * r2) + self.position;
  }

  pub fn sample(self, top: usize, left: usize) -> CameraSample {
    return CameraSample::new(self, self.get_sensor_point(top, left), self.get_lens_point());
  }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct CameraSample {
  pub ray: Ray,
  camera: Camera,
  sensor_position: Vector,
  lens_position: Vector,
}

impl CameraSample {
  fn new(camera: Camera, sensor_position: Vector, lens_position: Vector) -> CameraSample {
    return CameraSample {
      ray: CameraSample::get_ray(camera, sensor_position, lens_position),
      camera: camera,
      sensor_position: sensor_position,
      lens_position: lens_position,
    }
  }

  fn get_ray(camera: Camera, sensor_position: Vector, lens_position: Vector) -> Ray {
    let sensor_direction = sensor_position - camera.position;
    // センサーの位置はスクリーンの位置の逆側
    let screen_direction = sensor_direction * (-1.0);
    let screen_distance = screen_direction.dot(camera.forward);
    // 焦点が合う距離にあるオブジェクトプレーンの中心までの相対位置
    let object_plane_direction = screen_direction * (camera.focus_distance / screen_distance);
    return Ray {
      origin: lens_position,
      direction: (object_plane_direction - (lens_position - camera.position)).norm(),
    };
  }

  pub fn sensor_flux(self, incomming_radiance: Vector) -> Vector {
    let sensor_lens_direction = self.lens_position - self.sensor_position;
    let sensor_lens_direction_cos = sensor_lens_direction.norm().dot(self.camera.forward);
    // ジオメトリターム
    let g_term = (sensor_lens_direction_cos * sensor_lens_direction_cos) / sensor_lens_direction.dot(sensor_lens_direction);
    // レンズ上でのサンプルの確率密度関数 (1 / π * r^2)
    let lens_pdf = 1.0 / (PI * self.camera.lens_radius * self.camera.lens_radius);
    // イメージセンサー1ピクセル内でのサンプルの確率密度関数 (1 / w * h)
    let sensor_pdf = 1.0 / self.camera.sensor_pixel_area;
    // L * W * G / (P_lens * P_sensor)
    return incomming_radiance * (self.camera.sensor_sensitivity * g_term / lens_pdf / sensor_pdf);
  }
}
