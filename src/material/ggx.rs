extern crate rand;

use super::traits::Material;
use math::vector::*;
use sample::Sample;
use constant::*;
use util::OrthonormalBasis;

pub struct GGXMaterial {
  // 反射率
  pub reflectance: Vector3,
  // 屈折率
  pub ior: f32,
  // ラフネス
  pub roughness: f32,
}

impl GGXMaterial {
  fn alpha(&self) -> f32 {
    self.roughness * self.roughness
  }

  fn gaf_smith(&self, out_: Vector3, in_: Vector3, n: Vector3) -> f32 {
    self.g_ggx(in_, n) * self.g_ggx(out_, n)
  }

  fn g_ggx(&self, v: Vector3, n: Vector3) -> f32 {
    let a2 = self.alpha() * self.alpha();
    let cos = v.dot(n);
    let tan = 1.0 / (cos * cos) - 1.0;
    2.0 / (1.0 + (1.0 + a2 * tan * tan).sqrt())
  }

  fn ndf(&self, m: Vector3, n: Vector3) -> f32 {
    let a2 = self.alpha() * self.alpha();
    let mdn = m.dot(n);
    let x = (a2 - 1.0) * mdn * mdn + 1.0;
    a2 / (PI * x * x)
  }

  fn fresnel_schlick(&self, in_: Vector3, m: Vector3) -> f32 {
    let nnn = 1.0 - self.ior;
    let nnp = 1.0 + self.ior;
    let f_0 = (nnn * nnn) / (nnp * nnp);
    let c = in_.dot(m);
    f_0 + (1.0 - f_0) * (1.0 - c).powi(5)
  }
}

impl Material for GGXMaterial {
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
    self.reflectance.x.max(self.reflectance.y).max(
      self.reflectance.z,
    )
  }

  fn brdf(&self, out_: Vector3, in_: Vector3, n: Vector3) -> Vector3 {
    let on = self.orienting_normal(out_, n);
    if in_.dot(on) <= 0.0 { return Vector3::zero() }
    debug_assert!(out_.dot(on) > 0.0, "o.n  = {}", out_.dot(on));
    // ハーフベクトル
    let h = (in_ + out_).normalize();
    // Torrance-Sparrow model
    let f = self.fresnel_schlick(in_, h);
    debug_assert!(f >= 0.0 && f <= 1.0 && f.is_finite(), "f: {}", f);
    let g = self.gaf_smith(out_, in_, on);
    debug_assert!(g >= 0.0 && g <= 1.0 && g.is_finite(), "g: {}", g);
    let d = self.ndf(h, on);
    debug_assert!(d >= 0.0 && d.is_finite() , "d: {}", d);
    self.reflectance * f * g * d / (4.0 * in_.dot(on) * out_.dot(on))
  }

  fn sample(&self, out_: Vector3, n: Vector3) -> Sample<Vector3> {
    let on = self.orienting_normal(out_, n);
    // 法線方向を基準にした正規直交基底を生成
    let w = on;
    let (u, v) = w.orthonormal_basis();
    // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
    // (brdfの分布にしたがって重点的にサンプル)
    let r1 = 2.0 * PI * rand::random::<f32>();
    let r2 = rand::random::<f32>();
    let tan = self.alpha() * (r2 / (1.0 - r2)).sqrt();
    let x = 1.0 + tan * tan;
    let cos = 1.0 / x.sqrt();
    let sin = tan / x.sqrt();
    // ハーフベクトルをサンプリング
    let h = u * r1.cos() * sin + v * r1.sin() * sin + w * cos;
    // 入射ベクトル
    let o_h = out_.dot(h);
    let in_ = h * (2.0 * o_h) - out_;
    // ヤコビアン
    let jacobian = 1.0 / (4.0 * o_h);
    // 確率密度関数
    let pdf = self.ndf(h, on) * h.dot(on) * jacobian;
    Sample {
      value: in_,
      pdf: pdf,
    }
  }
}
