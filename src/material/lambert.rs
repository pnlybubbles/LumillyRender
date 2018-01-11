use super::traits::Material;
use math::vector::*;
use sample::Sample;
use constant::*;
use util::{Sampler, OrthonormalBasis};

pub struct LambertianMaterial {
  pub emission: Vector3,
  // 拡散反射率
  pub albedo: Vector3,
}

impl Material for LambertianMaterial {
  fn orienting_normal(&self, out_: Vector3, normal: Vector3) -> Vector3 {
    // 物体の内外を考慮した法線方向から拡散反射面としての法線方向を求める
    if normal.dot(out_) < 0.0 {
      normal * -1.0
    } else {
     normal 
    }
  }

  fn emission(&self) -> Vector3 {
    self.emission
  }

  fn weight(&self) -> f32 {
    // 拡散反射の時は各色の反射率のうち最大のものを使う
    self.albedo.x.max(self.albedo.y).max(self.albedo.z)
  }

  fn brdf(&self, _out_: Vector3, _in_: Vector3, _n_: Vector3) -> Vector3 {
    // BRDFは半球全体に一様に散乱するDiffuse面を考えると ρ / π
    self.albedo / PI
  }

  fn sample(&self, _out_: Vector3, n: Vector3) -> Sample<Vector3> {
    // 反射点での法線方向を基準にした正規直交基底を生成
    let w = n;
    let (u, v) = w.orthonormal_basis();
    // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
    // (cosにしたがって重点的にサンプル)
    let sample = Sampler::hemisphere_cos_importance();
    let in_ = u * sample.x + v * sample.y + w * sample.z;
    // cos項
    let cos_term = in_.dot(n);
    // 確率密度関数
    // (cosにしたがって重点的にサンプル) cosθ / π
    let pdf = cos_term / PI;
    Sample {
      value: in_,
      pdf: pdf,
    }
  }
}
