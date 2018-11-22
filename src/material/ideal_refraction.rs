use super::traits::Material;
use math::vector::*;
use sample::Sample;
use util::{BoundaryResponse};

pub struct IdealRefractionMaterial {
  // スペキュラー反射率
  pub reflectance: Vector3,
  pub absorbtance: f32,
  // 屈折率
  pub ior: f32,
}
// ディラックのデルタ関数
const DELTA_FUNC: f32 = 1.0;

impl Material for IdealRefractionMaterial {
  fn orienting_normal(&self, out_: Vector3, normal: Vector3) -> Vector3 {
    // 物体の内外を考慮した法線方向から拡散反射面としての法線方向を求める
    if normal.dot(out_) < 0.0 {
      normal * -1.0
    } else {
      normal
    }
  }

  fn emission(&self) -> Vector3 {
    Vector3::zero()
  }

  fn weight(&self) -> f32 {
    // 反射率のうち最大のものをつかう
    self
      .reflectance
      .x
      .max(self.reflectance.y)
      .max(self.reflectance.z)
  }

  fn brdf(&self, out_: Vector3, in_: Vector3, n: Vector3, _pos: Vector3) -> Vector3 {
    // out_: 入射
    // in_: 出射(透過/反射)
    // n: surface normal
    let on = self.orienting_normal(out_, n);
    let (from_ior, to_ior) = self.ior_pair(out_, n);
    let from_per_to_ior = from_ior / to_ior;
    match out_.refract(on, from_per_to_ior) {
      Some(r) => {
        // 屈折
        // 反射率
        let fr = Self::fresnel(from_ior, to_ior, out_, r, on);
        if in_.dot(on) > 0.0 {
          // 反射
          self.reflectance * DELTA_FUNC / in_.dot(n) * fr
        } else {
          // 透過率
          let ft = (1.0 - fr) * (to_ior / from_ior).powi(2);
          // ロシアンルーレットで反射と屈折のどちらかの寄与を取る
          self.reflectance * DELTA_FUNC / in_.dot(n) * ft
        }
      },
      None => {
        // 全反射
        self.reflectance * DELTA_FUNC / in_.dot(n)
      }
    }
  }

  fn sample(&self, out_: Vector3, n: Vector3) -> Sample<Vector3> {
    // out_: 入射
    // n: surface normal
    let (from_ior, to_ior) = self.ior_pair(out_, n);
    let from_per_to_ior = from_ior / to_ior;
    let on = self.orienting_normal(out_, n);
    // 鏡面反射レイ
    match out_.refract(on, from_per_to_ior) {
      Some(r) => {
        // 反射率
        let fr = Self::fresnel(from_ior, to_ior, out_, r, on);
        // ロシアンルーレットで反射と屈折のどちらかの寄与を取る
        let rr_prob = fr;
        if rand::random::<f32>() < rr_prob {
          // 反射
          Sample {
            value: out_.reflect(on),
            pdf: DELTA_FUNC * rr_prob
          }
        } else {
          // 透過
          Sample {
            value: r,
            pdf: DELTA_FUNC * (1.0 - rr_prob)
          }
        }
      },
      None => {
        // 全半射
        let r = out_.reflect(on);
        Sample {
          value: r,
          pdf: DELTA_FUNC
        }
      }
    }
  }

  fn coef(&self, out_: Vector3, n: Vector3, fly_distance: f32) -> Vector3 {
    if out_.dot(n) < 0.0 {
      let v = -(Vector3::new(1.0, 1.0, 1.0) - self.reflectance) * self.absorbtance * fly_distance;
      Vector3::new(v.x.exp(), v.y.exp(), v.z.exp())
    } else {
      Vector3::new(1.0, 1.0, 1.0)
    }
  }
}

impl IdealRefractionMaterial {
  fn ior_pair(&self, out_: Vector3, n: Vector3) -> (f32, f32) {
    // n: surface normal
    // 真空屈折率
    let ior_v = 1.0;
    // 物体屈折率
    let ior = self.ior;
    // 入射媒質屈折率 / 出射媒質屈折率
    let from_ior;
    let to_ior;
    if out_.dot(n) > 0.0 {
      // 表から物体へ
      from_ior = ior_v;
      to_ior = ior;
    } else {
      // 裏から物体外へ
      from_ior = ior;
      to_ior = ior_v;
    }
    (from_ior, to_ior)
  }

  fn fresnel(from_ior: f32, to_ior: f32, out_: Vector3, in_: Vector3, on: Vector3) -> f32 {
    // out_: 入射
    // in_: 透過
    // on: orienting normal
    // Fresnelの式
    let cos1 = out_.dot(on);
    let cos2 = in_.dot(-on);
    let n1 = from_ior;
    let n2 = to_ior;
    let rs = ((n1 * cos1 - n2 * cos2) / (n1 * cos1 + n2 * cos2)).powi(2);
    let rp = ((n1 * cos2 - n2 * cos1) / (n1 * cos2 + n2 * cos1)).powi(2);
    (rs + rp) / 2.0
    // Fresnelの式(Schlickの近似)より
    // 反射率
    // let cos = if from_ior < to_ior {
    //   out_.dot(on)
    // } else {
    //   in_.dot(-on)
    // };
    // let f_0 = (from_ior - to_ior).powi(2) / (from_ior + to_ior).powi(2);
    // f_0 + (1.0 - f_0) * (1.0 - cos).powi(5)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use constant::*;

  #[test]
  fn ior_pair_into_test() {
    let ior = 1.5;
    let mat = IdealRefractionMaterial {
      reflectance: Vector3::new(1.0, 1.0, 1.0),
      absorbtance: 0.0,
      ior: ior,
    };
    let n = Vector3::new(0.0, 0.0, 1.0);
    let out_ = Vector3::new(1.0, 0.0, 1.0).normalize();
    let (from_ior, to_ior) = mat.ior_pair(out_, n);
    assert!(from_ior == 1.0);
    assert!(to_ior == ior);
  }

  #[test]
  fn ior_pair_outgoing_test() {
    let ior = 1.5;
    let mat = IdealRefractionMaterial {
      reflectance: Vector3::new(1.0, 1.0, 1.0),
      absorbtance: 0.0,
      ior: ior,
    };
    let n = Vector3::new(0.0, 0.0, -1.0);
    let out_ = Vector3::new(1.0, 0.0, 1.0).normalize();
    let (from_ior, to_ior) = mat.ior_pair(out_, n);
    assert!(from_ior == ior);
    assert!(to_ior == 1.0);
  }

  #[test]
  fn brdf_reflecting_test() {
    let mat = IdealRefractionMaterial {
      reflectance: Vector3::new(1.0, 1.0, 1.0),
      absorbtance: 0.0,
      ior: INF,
    };
    let n = Vector3::new(0.0, 0.0, -1.0);
    let out_ = Vector3::new(1.0, 0.0, 1.0).normalize();
    let on = mat.orienting_normal(out_, n);
    let in_ = mat.sample(out_, n).value;
    let expect = out_.reflect(on);
    assert!((expect - in_).norm() < EPS);
    let brdf = mat.brdf(out_, in_, n, Vector3::zero());
    let expect_brdf = Vector3::new(1.0, 1.0, 1.0) / in_.dot(n);
    assert!((expect_brdf - brdf).norm() < EPS);
  }

  // #[test]
  // fn brdf_refracting_test() {
  //   let mat = IdealRefractionMaterial {
  //     reflectance: Vector3::new(1.0, 1.0, 1.0),
  //     ior: 1.5,
  //   };
  //   let n = Vector3::new(0.0, 0.0, 1.0);
  //   let out_ = Vector3::new(1.0, 0.0, 1.0).normalize();
  //   let on = mat.orienting_normal(out_, n);
  //   let (from_ior, to_ior) = mat.ior_pair(out_, n);
  //   let transmit = out_.refract(on, from_ior / to_ior).unwrap();
  //   let reflect = out_.reflect(on);
  //   let brdf_t = mat.brdf(out_, transmit, n);
  //   let brdf_r = mat.brdf(out_, reflect, n);
  //   // assert!(brdf_t.dot(Vector3::new(1.0, 1.0, 1.0)) > 0.0);
  //   // assert!(brdf_r.dot(Vector3::new(1.0, 1.0, 1.0)) > 0.0);
  //   let cos_term_r = reflect.dot(n);
  //   let cos_term_t = transmit.dot(n);
  //   println!("{}", transmit);
  //   assert!(brdf_r.x * cos_term_r > 0.0, "{} {}", brdf_r.x, cos_term_r);
  //   assert!(brdf_t.x * cos_term_t > 0.0, "{} {}", brdf_t.x, cos_term_t);
  //   assert!(brdf_r.x * cos_term_r < 1.0, "{} {}", brdf_r.x, cos_term_r);
  //   assert!(brdf_t.x * cos_term_t < 1.0, "{} {}", brdf_t.x, cos_term_t);
  // }

  // #[test]
  // fn brdf_refracting_outgoing_test() {
  //   let mat = IdealRefractionMaterial {
  //     reflectance: Vector3::new(1.0, 1.0, 1.0),
  //     ior: 1.5,
  //   };
  //   let n = Vector3::new(0.0, 0.0, -1.0);
  //   let out_ = Vector3::new(0.5, 0.0, 1.0).normalize();
  //   let on = mat.orienting_normal(out_, n);
  //   let (from_ior, to_ior) = mat.ior_pair(out_, n);
  //   let transmit = out_.refract(on, from_ior / to_ior).unwrap();
  //   let reflect = out_.reflect(on);
  //   let brdf_t = mat.brdf(out_, transmit, n);
  //   let brdf_r = mat.brdf(out_, reflect, n);
  //   assert!(brdf_t.dot(Vector3::new(1.0, 1.0, 1.0)) > 0.0);
  //   assert!(brdf_r.dot(Vector3::new(1.0, 1.0, 1.0)) > 0.0);
  //   let cos_term = reflect.dot(n).abs();
  //   assert!(brdf_r.x * cos_term < 1.0);
  //   assert!(brdf_t.x * cos_term < 1.0);
  // }

  #[test]
  fn fresnel_45_test() {
    let from_ior = 1.0;
    let to_ior = 1.5;
    let on = Vector3::new(0.0, 0.0, 1.0);
    let out_ = Vector3::new(1.0, 0.0, 1.0).normalize();
    let in_ = out_.refract(on, from_ior / to_ior).unwrap();
    let fr = IdealRefractionMaterial::fresnel(from_ior, to_ior, out_, in_, on);
    assert!(fr <= 1.0 && fr > 0.0);
  }

  #[test]
  fn fresnel_test() {
    let from_ior = 1.0;
    let to_ior = 1.5;
    let on = Vector3::new(0.0, 0.0, 1.0);
    for i in 0..100 {
      let t = i as f32 / 100.0 * PI / 2.0;
      let out_ = Vector3::new(t.sin(), 0.0, t.cos()).normalize();
      let in_ = out_.refract(on, from_ior / to_ior).unwrap();
      let fr = IdealRefractionMaterial::fresnel(from_ior, to_ior, out_, in_, on);
      assert!(fr <= 1.0 && fr > 0.0, "{}", fr);
    }
  }

  #[test]
  fn fresnel_outgoing_test() {
    let from_ior = 1.5;
    let to_ior = 1.0;
    let on = Vector3::new(0.0, 0.0, 1.0);
    for i in 0..100 {
      let t = i as f32 / 100.0 * PI / 2.0;
      let out_ = Vector3::new(t.sin(), 0.0, t.cos());
      out_.refract(on, from_ior / to_ior).map( |in_| {
        let fr = IdealRefractionMaterial::fresnel(from_ior, to_ior, out_, in_, on);
        assert!(fr <= 1.0 && fr > 0.0, "{} {} {}", fr, in_, on);
      });
    }
  }

  #[test]
  fn sample_test() {
    let mat = IdealRefractionMaterial {
      reflectance: Vector3::new(1.0, 1.0, 1.0),
      absorbtance: 0.0,
      ior: 1.5,
    };
    let n = Vector3::new(0.0, 0.0, -1.0);
    let out_ = Vector3::new(1.0, 0.0, 1.0).normalize();
    let in_ = mat.sample(out_, n).value;
    assert!((in_.norm() - 1.0).abs() < EPS);
  }
}
