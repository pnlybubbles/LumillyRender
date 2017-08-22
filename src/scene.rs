extern crate rand;

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
      Some(i) => self.diffuse(i, depth),
    }
  }

  fn diffuse(&self, i: Intersection, depth: usize) -> Vector3<f64> {
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
    // (半球面状で一様にサンプル)
    // let r1 = 2.0 * PI * rand::random::<f64>();
    // let r2 = rand::random::<f64>();
    // let r2s = (1.0 - r2 * r2).sqrt();
    // (cosにしたがって重点的にサンプル)
    let r1: f64 = 2.0 * PI * rand::random::<f64>();
    let r2: f64 = rand::random::<f64>();
    let r2s: f64 = r2.sqrt();

    // 反射点での法線方向を基準にした正規直交基底を生成
    let w = i.normal;
    let u = if w.x.abs() > EPS { Vector3::new(0.0, 1.0, 0.0) } else { Vector3::new(1.0, 0.0, 0.0) }.cross(w);
    let v = w.cross(u);

    // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
    // (半球面状で一様にサンプル)
    // let d = &(&u.smul(r1.cos() * r2s) + &v.smul(r1.sin() * r2s)) + &w.smul(r2);
    // (cosにしたがって重点的にサンプル)
    let d = u * (r1.cos() * r2s) + v * (r1.sin() * r2s) + w * (1.0 - r2).sqrt();
    // cos項を計算
    // let dn = d.dot(i.normal);
    // 新しいレイを作る
    let new_ray = Ray { direction: d, origin: i.position };
    // BRDFは半球全体に一様に散乱するDiffuse面を考えると σ / π
    // let brdf = i.material.diffuse.smul(1.0 / PI);
    // 確率密度関数
    // (半球面状で一様にサンプル) 1 / 2π
    // let pdf = 1.0 / (2.0 * PI);
    // (cosにしたがって重点的にサンプル) cosθ / π
    // let pdf = dn / PI;
    // レンダリング方程式にしたがって放射輝度を計算する
    // ロシアンルーレットを用いた評価で期待値を満たすために確率で割る (BRDFタイプ用と再帰抑制用)
    // L_e + BRDF * L_i * cosθ / (PDF * RR_prob)
    // return &l_e + &(&brdf * &radiance(new_ray, depth + 1).smul(dn / (pdf * continue_rr_prob * brdf_type_rr_prob)));
    return l_e + (i.material.diffuse * self.radiance(&new_ray, depth + 1) / continue_rr_prob);
  }
}
