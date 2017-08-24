extern crate rand;

use vector::*;
use vector3::Vector3;
use ray::Ray;
use objects::Objects;
use intersection::Intersection;

pub struct Scene {
  pub objects: Objects,
  pub depth: usize,
  pub depth_limit: usize,
  pub background: Vector3<f64>,
}

impl Scene {
  pub fn radiance(&self, ray: &Ray, depth: usize) -> Vector3<f64> {
    // すべてのオブジェクトと当たり判定を行う
    let maybe_intersect = self.objects.get_intersect(&ray);
    // 当たらなかった場合は背景色を返す
    match maybe_intersect {
      None => self.background,
      Some(i) => self.intersect_radiance(i, &ray, depth),
    }
  }

  pub fn normal(&self, ray: &Ray) -> Vector3<f64> {
    let maybe_intersect = self.objects.get_intersect(&ray);
    match maybe_intersect {
      None => Vector3::new(0.0, 0.0, 0.0),
      Some(i) => {
        if i.normal.dot(ray.direction) > 0.0 {
          i.normal * -1.0
        } else {
          i.normal
        }
      },
    }
  }

  pub fn depth(&self, ray: &Ray) -> f64 {
    let maybe_intersect = self.objects.get_intersect(&ray);
    match maybe_intersect {
      None => 0.0,
      Some(i) => i.distance,
    }
  }

  fn intersect_radiance(&self, i: Intersection, ray: &Ray, depth: usize) -> Vector3<f64> {
    // 放射
    let l_e = i.material.emission();
    // 再帰抑制用のロシアンルーレットの確率を決定する
    let mut continue_rr_prob = i.material.rr_weight();
    // スタックオーバーフロー対策のために反射回数の限界値を超えたら極端に確率を下げる
    if depth > self.depth_limit {
      continue_rr_prob *= (0.5f64).powi((depth - self.depth_limit) as i32);
    }
    // 最初の数回の反射では必ず次のパスをトレースするようにする
    if depth <= self.depth {
      continue_rr_prob = 1.0;
    }
    // ロシアンルーレットで再帰を抑制
    if continue_rr_prob != 1.0 && rand::random::<f64>() >= continue_rr_prob {
      return i.material.emission();
    }
    // レンダリング方程式にしたがって放射輝度を計算する
    // マテリアル
    // 新しいRayのサンプリング
    let (sample_ray, cos_term) = i.material.sample(&ray, &i);
    // ロシアンルーレットを用いた評価で期待値を満たすために確率で割る (再帰抑制用)
    // L_e + BRDF * L_i * cosθ / (PDF * RR_prob)
    let l_i = self.radiance(&sample_ray.value, depth + 1);
    let brdf = i.material.brdf(ray.direction, sample_ray.value.direction, i.normal);
    return l_e + (brdf * l_i * cos_term / sample_ray.pdf / continue_rr_prob);
  }
}
