extern crate rand;

use vector::*;
use sky::Sky;
use ray::Ray;
use objects::Objects;
use intersection::Intersection;

pub struct Scene {
  pub objects: Objects,
  pub depth: usize,
  pub depth_limit: usize,
  pub sky: Box<Sky + Send + Sync>,
}

impl Scene {
  pub fn radiance(&self, ray: &Ray, depth: usize) -> Vector {
    // すべてのオブジェクトと当たり判定を行う
    let maybe_intersect = self.objects.get_intersect(&ray);
    // 当たらなかった場合は背景色を返す
    match maybe_intersect {
      None => self.sky.radiance(&ray),
      Some(i) => self.intersect_radiance(i, &ray, depth),
    }
  }

  pub fn radiance_nee(&self, ray: &Ray, depth: usize, no_emission: bool) -> Vector {
    // すべてのオブジェクトと当たり判定を行う
    let maybe_intersect = self.objects.get_intersect(&ray);
    // 当たらなかった場合は背景色を返す
    match maybe_intersect {
      None => self.sky.radiance(&ray),
      Some(i) => self.intersect_radiance_nee(i, &ray, depth, no_emission),
    }
  }

  pub fn normal(&self, ray: &Ray) -> Vector {
    let maybe_intersect = self.objects.get_intersect(&ray);
    match maybe_intersect {
      None => Vector::zero(),
      Some(i) => i.normal / 2.0 + Vector::new(0.5, 0.5, 0.5),
    }
  }

  pub fn shade(&self, ray: &Ray, light: Vector) -> Vector {
    let maybe_intersect = self.objects.get_intersect(&ray);
    match maybe_intersect {
      None => Vector::zero(),
      Some(i) => (i.normal.dot(light) / 2.0 + 0.5) * i.material.color() + i.material.emission(),
    }
  }

  pub fn depth(&self, ray: &Ray) -> f64 {
    let maybe_intersect = self.objects.get_intersect(&ray);
    match maybe_intersect {
      None => 0.0,
      Some(i) => i.distance,
    }
  }

  fn intersect_radiance(&self, i: Intersection, ray: &Ray, depth: usize) -> Vector {
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
    let (sample_ray, brdf, cos_term) = i.material.sample(&ray, &i);
    // ロシアンルーレットを用いた評価で期待値を満たすために確率で割る (再帰抑制用)
    // L_e + BRDF * L_i * cosθ / (PDF * RR_prob)
    let l_i = self.radiance(&sample_ray.value, depth + 1);
    let pdf = sample_ray.pdf / continue_rr_prob;
    return l_e + (brdf * l_i * cos_term / pdf);
    // return i.normal / 2.0 + Vector::new(0.5, 0.5, 0.5);
  }

  fn intersect_radiance_nee(&self, i: Intersection, ray: &Ray, depth: usize, no_emission: bool) -> Vector {
    // 放射
    let l_e = if no_emission {
      Vector::zero()
    } else {
      i.material.emission()
    };
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
    let (sample_ray, brdf, cos_term) = i.material.sample(&ray, &i);
    // ロシアンルーレットを用いた評価で期待値を満たすために確率で割る (再帰抑制用)
    // L_e + BRDF * L_i * cosθ / (PDF * RR_prob)
    let l_i = self.radiance(&sample_ray.value, depth + 1);
    let pdf = sample_ray.pdf / continue_rr_prob;
    return l_e + (brdf * l_i * cos_term / pdf);
    // return i.normal / 2.0 + Vector::new(0.5, 0.5, 0.5);
  }
}
