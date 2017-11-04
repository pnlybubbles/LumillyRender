extern crate rand;

use constant::*;
use vector::*;
use ray::Ray;
use sample::Sample;

pub trait Camera {
  fn sample(&self, x: usize, y: usize) -> (Sample<Ray>, f64);
  fn sensor_sensitivity(&self) -> f64;
  fn info(&self) -> CameraInfo;
}

#[derive(Debug)]
pub struct PinholeCamera {
  // カメラの方向を基準とした正規直交基底
  pub forward: Vector,
  pub right: Vector,
  pub up: Vector,
  // センサーの中心座標(m)
  pub position: Vector,
  // センサーの解像度
  pub resolution: [usize; 2],
  // センサーの物理的な大きさ(m)
  pub sensor_size: [f64; 2],
  // 入射口の中心座標(m)
  pub aperture_position: Vector,
  // 入射口の半径(m)
  pub aperture_radius: f64,
  // 入射口とセンサー間の距離(m)
  pub aperture_sensor_distance: f64,
  // センサー1画素の面積(m^2)
  pub sensor_pixel_area: f64,
  // センサー感度(m^2)
  pub sensor_sensitivity: f64,
}

impl PinholeCamera {
  pub fn new(
    position: Vector,
    aperture_position: Vector,
    sensor_size: [f64; 2],
    resolution: [usize; 2],
    aperture_radius: f64,
  ) -> PinholeCamera {
    // レンズの方向(m)
    let direction = aperture_position - position;
    // 入射口とセンサー間の距離(m)
    let aperture_sensor_distance = direction.norm();
    // カメラの入射の方向を基準(forward)に正規直交基底
    let forward = direction.normalize();
    let right = forward.cross(if forward.y.abs() < 1.0 - EPS {
      Vector::new(0.0, 1.0, 0.0)
    } else {
      Vector::new(1.0, 0.0, 0.0)
    }).normalize();
    let up = right.cross(forward);
    // 1画素の面積 = センサーの面積 / センサーの画素数
    let sensor_pixel_area = (sensor_size[0] * sensor_size[1]) as f64 / (resolution[0] * resolution[1]) as f64;
    // センサー感度はpdfを打ち消すように設定(m^2 m^-2 m^-2)
    let sensor_sensitivity = aperture_sensor_distance * aperture_sensor_distance / (sensor_pixel_area * PI * aperture_radius * aperture_radius);
    PinholeCamera {
      forward: forward,
      right: right,
      up: up,
      position: position,
      resolution: resolution,
      sensor_size: sensor_size,
      aperture_position: aperture_position,
      aperture_radius: aperture_radius,
      aperture_sensor_distance: aperture_sensor_distance,
      sensor_pixel_area: sensor_pixel_area,
      sensor_sensitivity: sensor_sensitivity,
    }
  }

  fn sample_sensor(&self, left: usize, top: usize) -> Sample<Vector> {
    // イメージセンサー1画素内の点の座標を取得(一様分布)
    // 原点はセンサーの中心
    // 画素内の1点を一様分布でサンプリング(0~1の乱数)
    let u = rand::random::<f64>();
    let v = rand::random::<f64>();
    // センサー中心を基準とした平面座標でのサンプリング点の座標(m)
    let px = (((left as f64 + u) / self.resolution[0] as f64) - 0.5) * self.sensor_size[0];
    let py = (((top as f64 + v) / self.resolution[1] as f64) - 0.5) * self.sensor_size[1];
    // 空間でのサンプリング点の座標(m)
    let point = self.position - self.right * px + self.up * py;
    // 画素内の1点を一様分布でサンプリングした時の確率密度(m^-2)
    let pdf = 1.0 / self.sensor_pixel_area;
    Sample {
      value: point,
      pdf: pdf,
    }
  }

  fn sample_aperture(&self) -> Sample<Vector> {
    // 光が入射してくる入射口内の点の座標を取得(一様分布)
    let u = 2.0 * PI * rand::random::<f64>();
    let v = rand::random::<f64>().sqrt() * self.aperture_radius;
    // 入射口の中心を基準とした平面極座標でのサンプリング点の座標(m)
    let px = u.cos() * v;
    let py = u.sin() * v;
    // 空間でのサンプリング点の座標(m)
    let point = self.aperture_position + self.right * px + self.up * py;
    // 入射口内の1点を一様分布でサンプリングした時の確率密度(m^-2)
    let pdf = 1.0 / (PI * self.aperture_radius * self.aperture_radius);
    Sample {
      value: point,
      pdf: pdf,
    }
  }

  fn geometry_term(&self, direction: Vector) -> f64 {
    // cos項
    let cos_term = direction.dot(self.forward);
    // センサー面と開口部それぞれのサンプリング点同士の距離
    let d = self.aperture_sensor_distance / cos_term;
    // ジオメトリ項(m^-2)
    cos_term * cos_term / (d * d)
  }
}

impl Camera for PinholeCamera {
  fn sample(&self, x: usize, y: usize) -> (Sample<Ray>, f64) {
    let sensor_sample = self.sample_sensor(x, y);
    let aperture_sample = self.sample_aperture();
    let ray = Ray {
      origin: aperture_sample.value,
      direction: (aperture_sample.value - sensor_sample.value).normalize(),
    };
    let direction_to_sensor = ray.direction;
    (
      Sample {
        value: ray,
        pdf: sensor_sample.pdf * aperture_sample.pdf,
      },
      self.geometry_term(direction_to_sensor),
    )
  }

  fn sensor_sensitivity(&self) -> f64 {
    self.sensor_sensitivity
  }

  fn info(&self) -> CameraInfo {
    unimplemented!();
  }
}

#[derive(Debug)]
pub struct LensCamera {
  // カメラの方向を基準とした正規直交基底
  pub forward: Vector,
  pub right: Vector,
  pub up: Vector,
  // センサーの中心座標(m)
  pub position: Vector,
  // センサーの解像度
  pub resolution: [usize; 2],
  // センサーの物理的な大きさ(m)
  pub sensor_size: [f64; 2],
  // 入射口の中心座標(m)
  pub aperture_position: Vector,
  // 入射口の半径(m)
  pub aperture_radius: f64,
  // 入射口とセンサー間の距離(m)
  pub aperture_sensor_distance: f64,
  // センサー1画素の面積(m^2)
  pub sensor_pixel_area: f64,
  // センサー感度(m^2)
  pub sensor_sensitivity: f64,
  // 焦点の合う場所の入射口からの距離
  pub focus_distance: f64,
}

impl LensCamera {
  pub fn new(
    position: Vector,
    aperture_position: Vector,
    sensor_size: [f64; 2],
    resolution: [usize; 2],
    aperture_radius: f64,
    focus_distance: f64,
  ) -> LensCamera {
    // レンズの方向(m)
    let direction = aperture_position - position;
    // 入射口とセンサー間の距離(m)
    let aperture_sensor_distance = direction.norm();
    // カメラの入射の方向を基準(forward)に正規直交基底
    let forward = direction.normalize();
    let right = forward.cross(if forward.y.abs() < 1.0 - EPS {
      Vector::new(0.0, 1.0, 0.0)
    } else {
      Vector::new(1.0, 0.0, 0.0)
    }).normalize();
    let up = right.cross(forward);
    // 1画素の面積 = センサーの面積 / センサーの画素数
    let sensor_pixel_area = (sensor_size[0] * sensor_size[1]) as f64 / (resolution[0] * resolution[1]) as f64;
    // センサー感度はpdfを打ち消すように設定(m^2 m^-2 m^-[1]
    let sensor_sensitivity = aperture_sensor_distance * aperture_sensor_distance / (sensor_pixel_area * PI * aperture_radius * aperture_radius);
    LensCamera {
      forward: forward,
      right: right,
      up: up,
      position: position,
      resolution: resolution,
      sensor_size: sensor_size,
      aperture_position: aperture_position,
      aperture_radius: aperture_radius,
      aperture_sensor_distance: aperture_sensor_distance,
      sensor_pixel_area: sensor_pixel_area,
      sensor_sensitivity: sensor_sensitivity,
      focus_distance: focus_distance,
    }
  }

  fn sample_sensor(&self, left: usize, top: usize) -> Sample<Vector> {
    // イメージセンサー1画素内の点の座標を取得(一様分布)
    // 原点はセンサーの中心
    // 画素内の1点を一様分布でサンプリング(0~1の乱数)
    let u = rand::random::<f64>();
    let v = rand::random::<f64>();
    // センサー中心を基準とした平面座標でのサンプリング点の座標(m)
    let px = (((left as f64 + u) / self.resolution[0] as f64) - 0.5) * self.sensor_size[0];
    let py = (((top as f64 + v) / self.resolution[1] as f64) - 0.5) * self.sensor_size[1];
    // 空間でのサンプリング点の座標(m)
    let point = self.position - self.right * px + self.up * py;
    // 画素内の1点を一様分布でサンプリングした時の確率密度(m^-2)
    let pdf = 1.0 / self.sensor_pixel_area;
    Sample {
      value: point,
      pdf: pdf,
    }
  }

  fn sample_aperture(&self) -> Sample<Vector> {
    // 光が入射してくる入射口内の点の座標を取得(一様分布)
    let u = 2.0 * PI * rand::random::<f64>();
    let v = rand::random::<f64>().sqrt() * self.aperture_radius;
    // 入射口の中心を基準とした平面極座標でのサンプリング点の座標(m)
    let px = u.cos() * v;
    let py = u.sin() * v;
    // 空間でのサンプリング点の座標(m)
    let point = self.aperture_position + self.right * px + self.up * py;
    // 入射口内の1点を一様分布でサンプリングした時の確率密度(m^-2)
    let pdf = 1.0 / (PI * self.aperture_radius * self.aperture_radius);
    Sample {
      value: point,
      pdf: pdf,
    }
  }

  fn geometry_term(&self, direction: Vector) -> f64 {
    // cos項
    let cos_term = direction.dot(self.forward);
    // センサー面と開口部それぞれのサンプリング点同士の距離
    let d = self.aperture_sensor_distance / cos_term;
    // ジオメトリ項(m^-2)
    cos_term * cos_term / (d * d)
  }
}

impl Camera for LensCamera {
  fn sample(&self, x: usize, y: usize) -> (Sample<Ray>, f64) {
    let sensor_sample = self.sample_sensor(x, y);
    let aperture_sample = self.sample_aperture();
    // センサー上の点から開口部中心
    let sensor_center = self.aperture_position - sensor_sample.value;
    // 開口部中心からオブジェクトプレーン上
    let object_plane = sensor_center * (self.focus_distance / sensor_center.dot(self.forward));
    let ray = Ray {
      origin: aperture_sample.value,
      direction: (self.aperture_position + object_plane - aperture_sample.value).normalize(),
    };
    (
      Sample {
        value: ray,
        pdf: sensor_sample.pdf * aperture_sample.pdf,
      },
      self.geometry_term((aperture_sample.value - sensor_sample.value).normalize()),
    )
  }

  fn sensor_sensitivity(&self) -> f64 {
    self.sensor_sensitivity
  }

  fn info(&self) -> CameraInfo {
    // 焦点距離
    let focal_length = 1.0 / (1.0 / self.aperture_sensor_distance + 1.0 / self.focus_distance);
    // FoV
    let sensor_diagonal = (self.sensor_size[0].powi(2) + self.sensor_size[1].powi(2)).sqrt();
    let fov = 2.0 * ((sensor_diagonal / 2.0) / (2.0 * focal_length)).atan() * 180.0 / PI;
    // F値
    let f_number = focal_length / self.aperture_radius;
    CameraInfo {
      focal_length: focal_length,
      sensor_diagonal: sensor_diagonal,
      fov: fov,
      f_number: f_number,
    }
  }
}

#[derive(Debug)]
pub struct CameraInfo {
  pub focal_length: f64,
  pub sensor_diagonal: f64,
  pub fov: f64,
  pub f_number: f64,
}
