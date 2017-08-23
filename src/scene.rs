extern crate rand;

use vector::*;
use vector3::Vector3;
use ray::Ray;
use objects::Objects;
use constant::*;
use intersection::Intersection;

#[derive(Debug, Default, Clone)]
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
      Some(i) => self.diffuse(i, &ray, depth),
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

  fn diffuse(&self, i: Intersection, ray: &Ray, depth: usize) -> Vector3<f64> {
    // 拡散反射
    // 物体の内外を考慮した法線方向から拡散反射面としての法線方向を求める
    let orienting_normal = if i.normal.dot(ray.direction) > 0.0 {
      i.normal * -1.0
    } else {
      i.normal
    };
    // 放射
    let l_e = i.material.emission;
    // 再帰抑制用のロシアンルーレットの確率を決定する
    // 拡散反射の時は各色の反射率のうち最大のものを使う
    let mut continue_rr_prob = i.material.diffuse.x.max(i.material.diffuse.y).max(i.material.diffuse.z);
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
      return i.material.emission;
    }
    // 拡散反射
    // 乱数を生成
    // (cosにしたがって重点的にサンプル)
    let r1: f64 = 2.0 * PI * rand::random::<f64>();
    let r2: f64 = rand::random::<f64>();
    let r2s: f64 = r2.sqrt();
    // 反射点での法線方向を基準にした正規直交基底を生成
    let w = i.normal;
    let u = if w.x.abs() > EPS { Vector3::new(0.0, 1.0, 0.0) } else { Vector3::new(1.0, 0.0, 0.0) }.cross(w).norm();
    let v = w.cross(u);
    // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
    // (cosにしたがって重点的にサンプル)
    let d = u * (r1.cos() * r2s) + v * (r1.sin() * r2s) + w * (1.0 - r2).sqrt();
    // 新しいレイを作る
    let new_ray = Ray { direction: d, origin: i.position };
    // BRDFは半球全体に一様に散乱するDiffuse面を考えると σ / π
    // 確率密度関数
    // (cosにしたがって重点的にサンプル) cosθ / π
    // レンダリング方程式にしたがって放射輝度を計算する
    // ロシアンルーレットを用いた評価で期待値を満たすために確率で割る (BRDFタイプ用と再帰抑制用)
    // L_e + BRDF * L_i * cosθ / (PDF * RR_prob)
    return l_e + (i.material.diffuse * self.radiance(&new_ray, depth + 1) / continue_rr_prob);
  }
}
