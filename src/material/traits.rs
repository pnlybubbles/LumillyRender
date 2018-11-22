use math::vector::Vector3;
use sample::Sample;

// (ω_o) 出射ベクトル(視線ベクトル)
// (ω_i) 入射ベクトル(光源ベクトル)

pub trait Material {
  // 物体自体の放射成分
  fn emission(&self) -> Vector3;
  // 出射ベクトル, 物体法線ベクトル -> 法線ベクトル
  fn orienting_normal(&self, Vector3, Vector3) -> Vector3;
  // 出射ベクトル, 入射ベクトル, 法線ベクトル -> BRDF
  fn brdf(&self, Vector3, Vector3, Vector3) -> Vector3;
  // 出射ベクトル, 法線ベクトル -> 入射ベクトル, 確率密度
  fn sample(&self, Vector3, Vector3) -> Sample<Vector3>;
  // 再帰継続用ロシアンルーレットの重み
  fn weight(&self) -> f32;
  // 輝度に乗算する係数
  // 出射ベクトル, 法線ベクトル, 飛行距離
  fn coef(&self, Vector3, Vector3, f32) -> Vector3 {
    Vector3::new(1.0, 1.0, 1.0)
  }
}
