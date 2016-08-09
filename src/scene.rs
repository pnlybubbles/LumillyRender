extern crate rand;
extern crate image;

use vector::Vector;
use ray::Ray;
use objects::Objects;
use constant::*;

#[derive(Debug)]
pub enum Background {
  Ibl(Vec<image::Rgb<f32>>, usize, usize),
  Color(Vector),
}

#[derive(Debug)]
pub struct Scene {
  pub objects: Objects,
  pub depth: usize,
  pub depth_limit: usize,
  pub background: Background,
}

impl Scene {
  pub fn new(objects: Objects, depth: usize, depth_limit: usize, background: Background) -> Scene {
    return Scene {
      objects: objects,
      depth: depth,
      depth_limit: depth_limit,
      background: background,
    };
  }

  pub fn radiance(&self, ray: Ray, depth: usize, no_emission: bool) -> Vector {
    // すべてのオブジェクトと当たり判定を行う
    let i = self.objects.get_intersect(ray);
    // 当たらなかった場合は背景色を返す
    if !i.is_intersect {
      match self.background {
        Background::Ibl(ref hdr_image, height, offset) => {
          let theta = (ray.direction.y).acos();
          let phi_pr = (ray.direction.z / ray.direction.x).atan();
          let phi = if ray.direction.x < 0.0 { phi_pr + PI } else { phi_pr } + PI / 2.0;
          let x = (height as f64 * phi / PI).round() as usize + offset;
          let y = (height as f64 * theta / PI).round() as usize;
          let index = y * height * 2 + if x > height * 2 { x % (height * 2) } else { x };
          let color = hdr_image[index];
          return Vector{x: color.data[0] as f64, y: color.data[1] as f64, z: color.data[2] as f64};
        },
        Background::Color(color) => return color,
      }
    }
    // 放射
    let l_e = if no_emission { Vector::new(0.0, 0.0, 0.0) } else { i.material.emission };
    // 再帰抑制用のロシアンルーレットの確率を決定する
    // 鏡面反射、屈折の時は必ず次のパスをトレースする
    // 拡散反射の時は各色の反射率のうち最大のものを使う
    let mut continue_rr_prob = if i.material.reflection > 0.0 || i.material.refraction > 0.0 { 1.0 } else {
      i.material.color.x.max(i.material.color.y).max(i.material.color.z)
    };
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
    // 拡散反射、鏡面反射、屈折のそれぞれの割合からどの処理を行うかをロシアンルーレットで決定する
    let brdf_type_rr = rand::random::<f64>();
    let mut brdf_type_rr_prob = i.material.refraction;
    let mut brdf_type = 2;
    if brdf_type_rr < i.material.diffuse {
      brdf_type_rr_prob = i.material.diffuse;
      brdf_type = 0;
    } else if brdf_type_rr < i.material.diffuse + i.material.reflection {
      brdf_type_rr_prob = i.material.reflection;
      brdf_type = 1;
    }
    // 拡散反射
    if brdf_type == 0 {
      if self.objects.emission_area_total != 0.0 {
        // 光源が存在するとき、光源方向へサンプリング
        let mut direct_light_radiance = Vector::new(0.0, 0.0, 0.0);
        // 光源面から1点をサンプル(一様分布)
        let emmisive_position = self.objects.get_emission_point();
        let i_e = emmisive_position - i.position;
        let d_e = i_e.norm();
        let test_ray = Ray {
          origin: i.position + i.normal * 0.01,
          direction: d_e,
        };
        // 可視関数用のテストレイを光源面上のサンプル点に飛ばす
        let test_i = self.objects.get_intersect(test_ray);
        // 光源と交差しなかった場合、寄与は無し
        if test_i.is_intersect && test_i.material.emission.dot(test_i.material.emission) > 0.0 {
          // ジオメトリターム
          let g_term = (d_e.dot(i.normal) * (d_e * (-1.0)).dot(test_i.normal)) / i_e.dot(i_e);
          // 光源面積測度での確率密度関数(一様分布)
          let direct_light_pdf = 1.0 / self.objects.emission_area_total;
          // BRDFは半球全体に一様に散乱するDiffuse面を考えると σ / π
          let brdf = i.material.color * (1.0 / PI);
          // 積分範囲は光源面上
          direct_light_radiance = brdf * test_i.material.emission * (g_term / direct_light_pdf);
        }
      }
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
      let u = if w.x.abs() > EPS { Vector::new(0.0, 1.0, 0.0) } else { Vector::new(1.0, 0.0, 0.0) }.cross(w);
      let v = w.cross(u);

      // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
      // (半球面状で一様にサンプル)
      // let d = &(&u.smul(r1.cos() * r2s) + &v.smul(r1.sin() * r2s)) + &w.smul(r2);
      // (cosにしたがって重点的にサンプル)
      let d = u * (r1.cos() * r2s) + v * (r1.sin() * r2s) + w * (1.0 - r2).sqrt();
      // cos項を計算
      // let dn = d.dot(&i.normal);
      // 新しいレイを作る
      let new_ray = Ray {direction: d, origin: i.position};
      // BRDFは半球全体に一様に散乱するDiffuse面を考えると σ / π
      // let brdf = i.material.color.smul(1.0 / PI);
      // 確率密度関数
      // (半球面状で一様にサンプル) 1 / 2π
      // let pdf = 1.0 / (2.0 * PI);
      // (cosにしたがって重点的にサンプル) cosθ / π
      // let pdf = dn / PI;
      // レンダリング方程式にしたがって放射輝度を計算する
      // ロシアンルーレットを用いた評価で期待値を満たすために確率で割る (BRDFタイプ用と再帰抑制用)
      // L_e + BRDF * L_i * cosθ / (PDF * RR_prob)
      // return &l_e + &(&brdf * &radiance(new_ray, depth + 1).smul(dn / (pdf * continue_rr_prob * brdf_type_rr_prob)));
      // return &l_e + &(&i.material.color * &radiance(new_ray, depth + 1).smul(2.0 * dn / (continue_rr_prob * brdf_type_rr_prob)));
      // 積分範囲は光源以外なので光源に当たった場合の寄与は無し
      return l_e + (direct_light_radiance + (i.material.color * self.radiance(new_ray, depth + 1, true))) * (1.0 / (continue_rr_prob * brdf_type_rr_prob));
    } else if brdf_type == 1 { // 鏡面
      let new_ray = Ray{direction: ray.direction - i.normal * (2.0 * ray.direction.dot(i.normal)), origin: i.position};
      return l_e + (i.material.color * self.radiance(new_ray, depth + 1, false) * (1.0 / (continue_rr_prob * brdf_type_rr_prob)));
    } else if brdf_type == 2 { // 屈折面
      // cosθ
      let mut dn = ray.direction.dot(i.normal);
      let mut n = i.normal;
      // 真空屈折率
      let eta_v = 1.0;
      // 物体屈折率
      let eta = 1.5;
      // n1 / n2
      let nn;
      // n2 / n1
      let nni;
      // 入射方向の判別
      if dn < 0.0 {
        // 表から物体へ
        nn = eta_v / eta;
        nni = eta / eta_v;
      } else {
        // 裏から物体外へ
        nn = eta / eta_v;
        nni = eta_v / eta;
        dn = -dn;
        n = n * (-1.0);
      }
      // 鏡面反射レイ
      let reflection_ray = Ray {direction: ray.direction - i.normal * (2.0 * ray.direction.dot(i.normal)), origin: i.position};
      // 判別式(全半射)
      let det = 1.0 - nn * nn * (1.0 - dn * dn);
      if det < 0.0 {
        // 全反射
        return i.material.color * self.radiance(reflection_ray, depth + 1, false) * (1.0 / (continue_rr_prob * brdf_type_rr_prob));
      }
      // 屈折レイ
      let refraction_ray = Ray {direction: ray.direction * (nn) - n * (nn * dn + det.sqrt()), origin: i.position};
      // 垂直入射での反射量
      // n1 - n2
      let nnn;
      // cos1 or cos2
      let c;
      if dn < 0.0 {
        // 表から物体へ
        nnn = eta_v - eta;
        c = -dn;
      } else {
        // 裏から物体外へ
        nnn = eta - eta_v;
        c = refraction_ray.direction.dot(i.normal);
      }
      // n1 + n2
      let nnp = eta + eta_v;
      let f_0 = (nnn * nnn) / (nnp * nnp);
      // Fresnelの式(Schlickの近似)より
      let reflection_radiance_ratio = f_0 + (1.0 - f_0) * (1.0 - c).powi(5);
      let refraction_radiance_ratio = (1.0 - reflection_radiance_ratio) * nni * nni;
      // ロシアンルーレットで反射と屈折のどちらかの寄与を取る
      // 最初の数回は両方追跡する
      if depth > 2 {
        // どちらか一方
        let refraction_rr_prob = 0.25 + 0.5 * reflection_radiance_ratio;
        if rand::random::<f64>() < refraction_rr_prob {
          // 反射
          return i.material.color * self.radiance(reflection_ray, depth + 1, false) * (reflection_radiance_ratio / (continue_rr_prob * brdf_type_rr_prob * refraction_rr_prob));
        } else {
          // 屈折
          return i.material.color * self.radiance(refraction_ray, depth + 1, false) * (refraction_radiance_ratio / (continue_rr_prob * brdf_type_rr_prob * (1.0 - refraction_rr_prob)));
        }
      } else {
        // 両方
        return (i.material.color * (self.radiance(reflection_ray, depth + 1, false) * (reflection_radiance_ratio) + self.radiance(refraction_ray, depth + 1, false) * (refraction_radiance_ratio))) * (1.0 / (continue_rr_prob * brdf_type_rr_prob));
      }
    }
    return Vector::new(0.0, 0.0, 0.0);
  }
}
