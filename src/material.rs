extern crate rand;

use vector::*;
use ray::Ray;
use sample::Sample;
use constant::*;
use intersection::Intersection;
use util::*;

pub trait Material {
  fn color(&self) -> Vector;
  // 物体自体の放射成分
  fn emission(&self) -> Vector;
  // 入射ベクトル, 出射ベクトル, 法線ベクトル
  fn brdf(&self, Vector, Vector, Vector) -> Vector;
  // -> サンプルしたレイ, brdfの値, cos項
  fn sample(&self, &Ray, &Intersection) -> (Sample<Ray>, Vector, f32);
  // 再帰継続用ロシアンルーレットの重み
  fn rr_weight(&self) -> f32;
}

#[derive(Clone)]
pub struct LambertianMaterial {
  pub emission: Vector,
  // 拡散反射率
  pub albedo: Vector,
}

impl LambertianMaterial {
  pub fn orienting_normal(&self, in_: Vector, normal: Vector) -> Vector {
    // 物体の内外を考慮した法線方向から拡散反射面としての法線方向を求める
    if normal.dot(in_) > 0.0 {
      normal * -1.0
    } else {
      normal
    }
  }
}

impl Material for LambertianMaterial {
  fn color(&self) -> Vector {
    self.albedo
  }

  fn emission(&self) -> Vector {
    self.emission
  }

  fn rr_weight(&self) -> f32 {
    // 拡散反射の時は各色の反射率のうち最大のものを使う
    self.albedo.x.max(self.albedo.y).max(self.albedo.z)
  }

  fn brdf(&self, _: Vector, _: Vector, _: Vector) -> Vector {
    // BRDFは半球全体に一様に散乱するDiffuse面を考えると ρ / π
    self.albedo / PI
  }

  fn sample(&self, in_ray: &Ray, i: &Intersection) -> (Sample<Ray>, Vector, f32) {
    // 拡散反射
    let normal = self.orienting_normal(in_ray.direction, i.normal);
    // 反射点での法線方向を基準にした正規直交基底を生成
    let w = normal;
    let (u, v) = normal.orthonormal_basis();
    // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
    // (cosにしたがって重点的にサンプル)
    let sample = Sampler::hemisphere_cos_importance();
    let d = u * sample.x + v * sample.y + w * sample.z;
    // 新しいレイを作る
    let new_ray = Ray {
      direction: d,
      origin: i.position,
    };
    // cos項
    let cos_term = d.dot(normal);
    // 確率密度関数
    // (cosにしたがって重点的にサンプル) cosθ / π
    let pdf = cos_term / PI;
    let brdf = i.material.brdf(in_ray.direction, new_ray.direction, normal);
    (
      Sample {
        value: new_ray,
        pdf: pdf,
      },
      brdf,
      cos_term,
    )
  }
}

// #[derive(Clone)]
// pub struct IdealRefractionMaterial {
//   pub emission: Vector,
//   // スペキュラー反射率
//   pub albedo: Vector,
//   // 屈折率
//   pub ior: f32,
// }

// // ディラックのデルタ関数
// const DELTA_FUNC: f32 = 1.0;

// impl Material for IdealRefractionMaterial {
//   fn emission(&self) -> Vector {
//     self.emission
//   }

//   fn rr_weight(&self) -> f32 {
//     // 滑らかな界面の時は各色のスペキュラー反射率のうち最大のものを使う
//     self.albedo.x.max(self.albedo.y).max(self.albedo.z)
//   }

//   fn brdf(&self, _: Vector, out_: Vector, normal: Vector) -> Vector {
//     self.albedo * DELTA_FUNC / out_.dot(normal)
//   }

//   fn sample(&self, in_ray: &Ray, i: &Intersection) -> (Sample<Ray>, Vector, f32) {
//     // cosθ
//     let mut dn = in_ray.direction.dot(i.normal);
//     let mut n = i.normal;
//     // 真空屈折率
//     let eta_v = 1.0;
//     // 物体屈折率
//     let eta = self.ior;
//     // n1 / n2
//     let nn;
//     // n2 / n1
//     let nni;
//     // 入射方向の判別
//     if dn < 0.0 {
//       // 表から物体へ
//       nn = eta_v / eta;
//       nni = eta / eta_v;
//     } else {
//       // 裏から物体外へ
//       nn = eta / eta_v;
//       nni = eta_v / eta;
//       dn = -dn;
//       n = n * (-1.0);
//     }
//     // 鏡面反射レイ
//     let reflection_ray = Ray {
//       direction: in_ray.direction - i.normal * (2.0 * in_ray.direction.dot(i.normal)),
//       origin: i.position,
//     };
//     // 判別式(全半射)
//     let det = 1.0 - nn * nn * (1.0 - dn * dn);
//     if det < 0.0 {
//       // 全反射
//       let brdf = self.brdf(in_ray.direction, reflection_ray.direction, n);
//       let cos_term = reflection_ray.direction.dot(n);
//       return (
//         Sample {
//           value: reflection_ray,
//           pdf: DELTA_FUNC,
//         },
//         brdf,
//         cos_term,
//       );
//     }
//     // 屈折レイ
//     let refraction_ray = Ray {
//       direction: in_ray.direction * (nn) - n * (nn * dn + det.sqrt()),
//       origin: i.position,
//     };
//     // 垂直入射での反射量
//     // n1 - n2
//     let nnn;
//     // cos1 or cos2
//     let c;
//     if dn < 0.0 {
//       // 表から物体へ
//       nnn = eta_v - eta;
//       c = -dn;
//     } else {
//       // 裏から物体外へ
//       nnn = eta - eta_v;
//       c = refraction_ray.direction.dot(i.normal);
//     }
//     // n1 + n2
//     let nnp = eta + eta_v;
//     let f_0 = (nnn * nnn) / (nnp * nnp);
//     // Fresnelの式(Schlickの近似)より
//     // 反射率
//     let fr = f_0 + (1.0 - f_0) * (1.0 - c).powi(5);
//     // 透過率
//     let ft = (1.0 - fr) * nni * nni;
//     // ロシアンルーレットで反射と屈折のどちらかの寄与を取る
//     let rr_prob = fr;
//     if rand::random::<f32>() < rr_prob {
//       // 反射
//       let brdf = self.brdf(in_ray.direction, reflection_ray.direction, n) * fr;
//       let cos_term = reflection_ray.direction.dot(n);
//       (
//         Sample {
//           value: reflection_ray,
//           pdf: DELTA_FUNC * rr_prob,
//         },
//         brdf,
//         cos_term,
//       )
//     } else {
//       // 屈折
//       let brdf = self.brdf(in_ray.direction, refraction_ray.direction, n) * ft;
//       let cos_term = refraction_ray.direction.dot(n);
//       (
//         Sample {
//           value: refraction_ray,
//           pdf: DELTA_FUNC * (1.0 - rr_prob),
//         },
//         brdf,
//         cos_term,
//       )
//     }
//   }
// }

// #[derive(Clone)]
// pub struct CookTorranceMaterial {
//   // 鏡面反射率
//   pub reflectance: Vector,
//   // 吸収係数
//   pub absorptance: Vector,
//   // 屈折率
//   pub ior: f32,
//   // ラフネス
//   pub roughness: f32,
// }

// impl CookTorranceMaterial {
//   pub fn orienting_normal(&self, in_: Vector, normal: Vector) -> Vector {
//     // 物体の内外を考慮した法線方向から拡散反射面としての法線方向を求める
//     if normal.dot(in_) > 0.0 {
//       normal * -1.0
//     } else {
//       normal
//     }
//   }

//   fn alpha(&self) -> f32 {
//     self.roughness * self.roughness
//   }

//   // マイクロファセット分布関数 (Microfacet Distribution Functions)
//   fn ndf(&self, h: Vector, n: Vector) -> f32 {
//     // GGX
//     let n_dot_h = n.dot(h);
//     let a = self.alpha();
//     let a2 = a * a;
//     let b = (a2 - 1.0) * n_dot_h * n_dot_h + 1.0;
//     a2 / (PI * b * b)
//   }

//   // 幾何減衰項 (Masking-Shadowing Fucntion)
//   fn geometry(&self, i: Vector, o: Vector, h: Vector, n: Vector) -> f32 {
//     // Height-Correlated Masking and Shadowing (Smith Joint Masking-Shadowing Function)
//     self.g1(i, h, n) * self.g1(o, h, n)
//   }

//   #[allow(unused_variables)]
//   fn g1(&self, x: Vector, h: Vector, n: Vector) -> f32 {
//     let a = self.alpha();
//     let a2 = a * a;
//     let x_dot_n = x.dot(n);
//     2.0 * x.dot(n) / (x.dot(n) * (a2 + (1.0 - a2) * x_dot_n * x_dot_n).sqrt())
//   }

//   // フレネル項
//   fn fresnel(&self, i: Vector, h: Vector) -> f32 {
//     // 垂直入射での反射量
//     // 真空屈折率
//     let eta_v = 1.0;
//     // 物体屈折率
//     let eta = self.ior;
//     // n1 - n2
//     let nnn = eta_v - eta;
//     // n1 + n2
//     let nnp = eta + eta_v;
//     let f_0 = (nnn * nnn) / (nnp * nnp);
//     // Fresnelの式(Schlickの近似)より
//     // 反射率
//     f_0 + (1.0 - f_0) * (1.0 - i.dot(h)).powi(5)
//   }
// }

// impl Material for CookTorranceMaterial {
//   fn emission(&self) -> Vector {
//     Vector::zero()
//   }

//   fn rr_weight(&self) -> f32 {
//     // 反射率のうち最大のものをつかう
//     self.reflectance.x.max(self.reflectance.y).max(
//       self.reflectance.z,
//     )
//   }

//   fn brdf(&self, i: Vector, o: Vector, n: Vector) -> Vector {
//     let h = (o + i).normalize();
//     // Torrance-Sparrow model (PBRT p.546)
//     // fr = FGD / 4(i.n)(o.n)
//     let fr = self.fresnel(i, h) * self.geometry(i, o, h, n) * self.ndf(h, n) /
//       (4.0 * i.dot(n) * o.dot(n));
//     self.reflectance * fr
//   }

//   fn sample(&self, in_ray: &Ray, i: &Intersection) -> (Sample<Ray>, Vector, f32) {
//     let normal = self.orienting_normal(in_ray.direction, i.normal);
//     // 反射点での法線方向を基準にした正規直交基底を生成
//     let w = normal;
//     let (u, v) = normal.orthonormal_basis();
//     // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
//     // (cosにしたがって重点的にサンプル)
//     let sample = Sampler::hemisphere_cos_importance();
//     let d = u * sample.x + v * sample.y + w * sample.z;
//     // 新しいレイを作る
//     let new_ray = Ray {
//       direction: d,
//       origin: i.position,
//     };
//     // cos項
//     let cos_term = d.dot(normal);
//     // 確率密度関数
//     // (cosにしたがって重点的にサンプル) cosθ / π
//     let pdf = cos_term / PI;
//     // in = view, out = light
//     let brdf = i.material.brdf(
//       -in_ray.direction,
//       new_ray.direction,
//       normal,
//     );
//     (
//       Sample {
//         value: new_ray,
//         pdf: pdf,
//       },
//       brdf,
//       cos_term,
//     )
//   }
// }

// #[derive(Clone)]
// pub struct PhongMaterial {
//   // 反射率
//   pub reflectance: Vector,
//   // ラフネス
//   pub roughness: f32,
// }

// impl PhongMaterial {
//   pub fn orienting_normal(&self, in_: Vector, normal: Vector) -> Vector {
//     // 物体の内外を考慮した法線方向から拡散反射面としての法線方向を求める
//     if normal.dot(in_) > 0.0 {
//       normal * -1.0
//     } else {
//       normal
//     }
//   }

//   pub fn alpha(&self) -> f32 {
//     self.roughness
//   }
// }

// impl Material for PhongMaterial {
//   fn emission(&self) -> Vector {
//     Vector::zero()
//   }

//   fn rr_weight(&self) -> f32 {
//     // 反射率のうち最大のものをつかう
//     self.reflectance.x.max(self.reflectance.y).max(
//       self.reflectance.z,
//     )
//   }

//   fn brdf(&self, v_i: Vector, l: Vector, n: Vector) -> Vector {
//     let v = -v_i;
//     let r = v.reflect(n);
//     let cos = r.dot(l);
//     let a = self.alpha();
//     // modified phong
//     self.reflectance * ((a + 2.0) / (2.0 * PI) * cos.powf(a))
//   }

//   fn sample(&self, in_ray: &Ray, i: &Intersection) -> (Sample<Ray>, Vector, f32) {
//     let normal = self.orienting_normal(in_ray.direction, i.normal);
//     let a = self.alpha();
//     // 反射点での法線方向を基準にした正規直交基底を生成
//     let w = normal;
//     let (u, v) = normal.orthonormal_basis();
//     // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
//     // (brdfの分布にしたがって重点的にサンプル)
//     let r1 = 2.0 * PI * rand::random::<f32>();
//     let r2 = rand::random::<f32>();
//     let t = r2.powf(2.0 / (a + 1.0));
//     let ts = (1.0 - t).sqrt();

//     let d = u * r1.cos() * ts + v * r1.sin() * ts + w * t.sqrt();
//     // 新しいレイを作る
//     let new_ray = Ray {
//       direction: d,
//       origin: i.position,
//     };
//     // cos項
//     let cos_term = d.dot(normal);
//     // 確率密度関数
//     // (brdfの分布にしたがって重点的にサンプル)
//     let v = -in_ray.direction;
//     let r = v.reflect(normal);
//     let cos = r.dot(new_ray.direction);
//     let pdf = (a + 2.0) / (2.0 * PI) * cos.powf(a);
//     let brdf = i.material.brdf(in_ray.direction, new_ray.direction, normal);
//     (
//       Sample {
//         value: new_ray,
//         pdf: pdf,
//       },
//       brdf,
//       cos_term,
//     )
//   }
// }
