extern crate rand;

use vector::*;
use sky::Sky;
use ray::Ray;
use objects::Objects;
use shape::Shape;
use intersection::Intersection;
use constant::*;

pub struct Scene {
  pub objects: Objects,
  pub depth: usize,
  pub depth_limit: usize,
  pub sky: Box<Sky + Send + Sync>,
}

impl Scene {
  pub fn radiance(&self, ray: &Ray, depth: usize, no_direct_emitter: bool) -> Vector {
    // すべてのオブジェクトと当たり判定を行う
    let maybe_intersect = self.objects.intersect(&ray);
    // 当たらなかった場合は背景色を返す
    match maybe_intersect {
      None => self.sky.radiance(&ray),
      Some(i) => self.intersect_radiance(i, &ray, depth, no_direct_emitter),
    }
  }
  
  pub fn radiance_nee(&self, ray: &Ray, depth: usize, no_emission: bool, no_direct_emitter: bool) -> Vector {
    // すべてのオブジェクトと当たり判定を行う
    let maybe_intersect = self.objects.intersect(&ray);
    // 当たらなかった場合は背景色を返す
    match maybe_intersect {
      None => self.sky.radiance(&ray),
      Some(i) => self.intersect_radiance_nee(i, &ray, depth, no_emission, no_direct_emitter),
    }
  }

  pub fn normal(&self, ray: &Ray) -> Vector {
    let maybe_intersect = self.objects.intersect(&ray);
    match maybe_intersect {
      None => Vector::zero(),
      Some(i) => i.normal / 2.0 + Vector::new(0.5, 0.5, 0.5),
    }
  }

  pub fn shade(&self, ray: &Ray, light: Vector) -> Vector {
    let maybe_intersect = self.objects.intersect(&ray);
    match maybe_intersect {
      None => Vector::zero(),
      Some(i) => (i.normal.dot(light) / 2.0 + 0.5) * i.material.color() + i.material.emission(),
    }
  }

  pub fn depth(&self, ray: &Ray) -> f32 {
    let maybe_intersect = self.objects.intersect(&ray);
    match maybe_intersect {
      None => 0.0,
      Some(i) => i.distance,
    }
  }

  fn intersect_radiance(&self, i: Intersection, ray: &Ray, depth: usize, no_direct_emitter: bool) -> Vector {
    // 放射
    let l_e = if !no_direct_emitter && (-ray.direction).dot(i.normal) > 0.0 {
      i.material.emission()
    } else {
      Vector::zero()
    };
    // 再帰抑制用のロシアンルーレットの確率を決定する
    let mut continue_rr_prob = i.material.rr_weight();
    // スタックオーバーフロー対策のために反射回数の限界値を超えたら極端に確率を下げる
    if depth > self.depth_limit {
      continue_rr_prob *= (0.5f32).powi((depth - self.depth_limit) as i32);
    }
    // 最初の数回の反射では必ず次のパスをトレースするようにする
    if depth <= self.depth && continue_rr_prob > 0.0 {
      continue_rr_prob = 1.0;
    }
    // ロシアンルーレットで再帰を抑制
    if continue_rr_prob != 1.0 && rand::random::<f32>() >= continue_rr_prob {
      return l_e;
    }
    // レンダリング方程式にしたがって放射輝度を計算する
    // マテリアル
    // 新しいRayのサンプリング
    let (sample_ray, brdf, cos_term) = i.material.sample(&ray, &i);
    // ロシアンルーレットを用いた評価で期待値を満たすために確率で割る (再帰抑制用)
    // L_e + BRDF * L_i * cosθ / (PDF * RR_prob)
    let l_i = self.radiance(&sample_ray.value, depth + 1, false);
    let pdf = sample_ray.pdf;
    return l_e + (brdf * l_i * cos_term / pdf) / continue_rr_prob;
    // return i.normal / 2.0 + Vector::new(0.5, 0.5, 0.5);
  }

  fn intersect_radiance_nee(&self, i: Intersection, ray: &Ray, depth: usize, no_emission: bool, no_direct_emitter: bool) -> Vector {
    // 放射
    let l_e = if !no_direct_emitter && !no_emission && (-ray.direction).dot(i.normal) > 0.0 {
      i.material.emission()
    } else {
      Vector::zero()
    };
    // 再帰抑制用のロシアンルーレットの確率を決定する
    let mut continue_rr_prob = i.material.rr_weight();
    // スタックオーバーフロー対策のために反射回数の限界値を超えたら極端に確率を下げる
    if depth > self.depth_limit {
      continue_rr_prob *= (0.5f32).powi((depth - self.depth_limit) as i32);
    }
    // 最初の数回の反射では必ず次のパスをトレースするようにする
    if depth <= self.depth && continue_rr_prob > 0.0 {
      continue_rr_prob = 1.0;
    }
    // ロシアンルーレットで再帰を抑制
    if continue_rr_prob != 1.0 && rand::random::<f32>() >= continue_rr_prob {
      return l_e;
    }
    // 直接光をサンプリング
    let direct_radiance = if i.material.emission().sqr_norm() == 0.0 && self.objects.has_emission() {
      // 光源上から1点をサンプリング (確率密度は面積測度)
      let direct_sample = self.objects.sample_emission();
      // 交差した座標と光源上の1点のパスを接続
      let direct_path = direct_sample.value - i.position;
      // 可視関数のテストレイを生成
      let direct_ray = Ray {
        origin: i.position,
        direction: direct_path.normalize(),
      };
      // 直接光のみのサンプリングなので可視の場合のみ寄与
      match self.objects.intersect(&direct_ray) {
        Some(direct_i) => {
          if (direct_i.distance - direct_path.norm()).abs() < EPS {
            let cos_emission = (-direct_ray.direction).dot(direct_i.normal);
            if cos_emission > 0.0 {
              // 光源は表面のみ寄与あり
              let cos_surface = direct_ray.direction.dot(i.normal);
              // ジオメトリターム (測度の変換)
              let g_term = cos_surface * cos_emission / direct_path.sqr_norm();
              let brdf = i.material.brdf(ray.direction, direct_ray.direction, i.normal);
              let l_i = direct_i.material.emission();
              let pdf = direct_sample.pdf;
              brdf * l_i * g_term / pdf
            } else {
              // 光源の裏面は寄与なし
              Vector::zero()
            }
          } else {
            Vector::zero()
          }
        },
        None => Vector::zero(),
      }
    } else {
      Vector::zero()
    };
    // レンダリング方程式にしたがって放射輝度を計算する
    // マテリアル
    // 新しいRayのサンプリング
    let (sample_ray, brdf, cos_term) = i.material.sample(&ray, &i);
    // ロシアンルーレットを用いた評価で期待値を満たすために確率で割る (再帰抑制用)
    // L_e + BRDF * L_i * cosθ / (PDF * RR_prob)
    let l_i = self.radiance_nee(&sample_ray.value, depth + 1, true, false);
    let pdf = sample_ray.pdf;
    return l_e + (direct_radiance + brdf * l_i * cos_term / pdf) / continue_rr_prob;
    // return i.normal / 2.0 + Vector::new(0.5, 0.5, 0.5);
  }
}
