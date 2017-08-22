extern crate rand;

use constant::*;
use vector::{Vector, VectorFloat};
use vector2::Vector2;
use vector3::Vector3;
use ray::Ray;
use sample::Sample;

#[derive(Debug, Default, Clone)]
pub struct Camera {
  // カメラの方向を基準とした正規直交基底
  pub forward: Vector3<f64>,
  pub right: Vector3<f64>,
  pub up: Vector3<f64>,
  // センサーの中心座標(m)
  pub position: Vector3<f64>,
  // センサーの解像度
  pub resolution: Vector2<usize>,
  // センサーの物理的な大きさ(m)
  pub sensor_size: Vector2<f64>,
  // 入射口の中心座標(m)
  pub aperture_position: Vector3<f64>,
  // 入射口の半径(m)
  pub aperture_radius: f64,
  // 入射口とセンサー間の距離(m)
  pub aperture_sensor_distance: f64,
  // センサー1画素の面積(m^2)
  pub sensor_pixel_area: f64,
  // センサー感度(m^2)
  pub sensor_sensitivity: f64,
}

impl Camera {
  pub fn new(
    position: Vector3<f64>,
    aperture_position: Vector3<f64>,
    sensor_size: Vector2<f64>,
    resolution: Vector2<usize>,
    aperture_radius: f64,
  ) -> Camera {
    // レンズの方向(m)
    let direction = aperture_position - position;
    // 入射口とセンサー間の距離(m)
    let aperture_sensor_distance = direction.len();
    // カメラの入射の方向を基準(forward)に正規直交基底
    let forward = direction.norm();
    let right = forward.cross(if forward.y.abs() < 1.0 - EPS {
      Vector3::new(0.0, 1.0, 0.0)
    } else {
      Vector3::new(1.0, 0.0, 0.0)
    });
    let up = right.cross(forward);
    // 1画素の面積 = センサーの面積 / センサーの画素数
    let sensor_pixel_area = (sensor_size.x * sensor_size.y) as f64 / (resolution.x * resolution.y) as f64;
    // センサー感度はpdfを打ち消すように設定(m^2 m^-2 m^-2)
    let sensor_sensitivity = aperture_sensor_distance * aperture_sensor_distance / (sensor_pixel_area * PI * aperture_radius * aperture_radius);
    Camera {
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

  pub fn sample(&self, x: usize, y: usize) -> Sample<Ray> {
    let sensor_sample = self.sample_sensor(x, y);
    let aperture_sample = self.sample_aperture();
    let ray = Ray {
      origin: aperture_sample.value,
      direction: (aperture_sample.value - sensor_sample.value).norm(),
    };
    Sample {
      value: ray,
      pdf: sensor_sample.pdf * aperture_sample.pdf,
    }
  }

  fn sample_sensor(&self, left: usize, top: usize) -> Sample<Vector3<f64>> {
    // イメージセンサー1画素内の点の座標を取得(一様分布)
    // 原点はセンサーの中心
    // 画素内の1点を一様分布でサンプリング(0~1の乱数)
    let u = rand::random::<f64>();
    let v = rand::random::<f64>();
    // センサー中心を基準とした平面座標でのサンプリング点の座標(m)
    let px = (((left as f64 + u) / self.resolution.x as f64) - 0.5) * self.sensor_size.x;
    let py = (((top as f64 + v) / self.resolution.y as f64) - 0.5) * self.sensor_size.y;
    // 空間でのサンプリング点の座標(m)
    let point = self.position - self.right * px + self.up * py;
    // 画素内の1点を一様分布でサンプリングした時の確率密度(m^-2)
    let pdf = 1.0 / self.sensor_pixel_area;
    Sample {
      value: point,
      pdf: pdf,
    }
  }

  fn sample_aperture(&self) -> Sample<Vector3<f64>> {
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
}
