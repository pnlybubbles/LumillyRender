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
    if in_.dot(n) < 0.0 { return Vector3::zero() }
    // ハーフベクトル
    let h = (in_ + out_).normalize();
    // Torrance-Sparrow model
    let f = self.fresnel_schlick(in_, h);
    let g = self.gaf_smith(out_, in_, n);
    // let g = out_.dot(n) * in_.dot(n);
    let d = self.ndf(h, n);
    self.reflectance * f * g * d / (4.0 * in_.dot(n) * out_.dot(n))
  }

  fn sample(&self, out_: Vector3, n: Vector3) -> Sample<Vector3> {
    // 法線方向を基準にした正規直交基底を生成
    let w = n;
    let (u, v) = w.orthonormal_basis();
    // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
    // (brdfの分布にしたがって重点的にサンプル)
    let r1 = 2.0 * PI * rand::random::<f32>();
    let r2 = rand::random::<f32>();
    let a2 = self.alpha() * self.alpha();
    let t_cos = ((1.0 - r2) / ((a2 - 1.0) * r2 + 1.0)).sqrt();
    let t_sin = (1.0 - t_cos * t_cos).sqrt();
    // ハーフベクトルをサンプリング
    let h = u * r1.cos() * t_sin + v * r1.sin() * t_sin + w * t_cos;
    // 入射ベクトル
    let in_ = h * (2.0 * out_.dot(h)) - out_;
    // 確率密度関数
    let pdf = self.ndf(h, n) * h.dot(n) / (4.0 * out_.dot(h));
    Sample {
      value: in_,
      pdf: pdf,
    }
  }
}
