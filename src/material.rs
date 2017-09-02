extern crate rand;

use vector::*;
use vector3::Vector3;
use ray::Ray;
use sample::Sample;
use constant::*;
use intersection::Intersection;
use util::*;

pub trait Material {
  // 物体自体の放射成分
  fn emission(&self) -> Vector3<f64>;
  // 入射ベクトル, 出射ベクトル, 法線ベクトル
  fn brdf(&self, Vector3<f64>, Vector3<f64>, Vector3<f64>) -> Vector3<f64>;
  // -> サンプルしたレイ, brdfの値, cos項
  fn sample(&self, &Ray, &Intersection) -> (Sample<Ray>, Vector3<f64>, f64);
  // 再帰継続用ロシアンルーレットの重み
  fn rr_weight(&self) -> f64;
}

pub struct LambertianMaterial {
  pub emission: Vector3<f64>,
  // 拡散反射率
  pub albedo: Vector3<f64>,
}

impl LambertianMaterial {
  pub fn orienting_normal(&self, in_: Vector3<f64>, normal: Vector3<f64>) -> Vector3<f64> {
    // 物体の内外を考慮した法線方向から拡散反射面としての法線方向を求める
    if normal.dot(in_) > 0.0 {
      normal * -1.0
    } else {
      normal
    }
  }
}

impl Material for LambertianMaterial {
  fn emission(&self) -> Vector3<f64> {
    self.emission
  }

  fn rr_weight(&self) -> f64 {
    // 拡散反射の時は各色の反射率のうち最大のものを使う
    self.albedo.x.max(self.albedo.y).max(self.albedo.z)
  }

  fn brdf(&self, _: Vector3<f64>, _: Vector3<f64>, _: Vector3<f64>) -> Vector3<f64> {
    // BRDFは半球全体に一様に散乱するDiffuse面を考えると ρ / π
    self.albedo / PI
  }

  fn sample(&self, in_ray: &Ray, i: &Intersection) -> (Sample<Ray>, Vector3<f64>, f64) {
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
    let new_ray = Ray { direction: d, origin: i.position };
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
      cos_term
    )
  }
}

pub struct IdealRefractionMaterial {
  pub emission: Vector3<f64>,
  // スペキュラー反射率
  pub albedo: Vector3<f64>,
  // 屈折率
  pub ior: f64,
}

// ディラックのデルタ関数
const DELTA_FUNC: f64 = 1.0;

impl Material for IdealRefractionMaterial {
  fn emission(&self) -> Vector3<f64> {
    self.emission
  }

  fn rr_weight(&self) -> f64 {
    // 滑らかな界面の時は各色のスペキュラー反射率のうち最大のものを使う
    self.albedo.x.max(self.albedo.y).max(self.albedo.z)
  }

  fn brdf(&self, _: Vector3<f64>, out_: Vector3<f64>, normal: Vector3<f64>) -> Vector3<f64> {
    self.albedo * DELTA_FUNC / out_.dot(normal)
  }

  fn sample(&self, in_ray: &Ray, i: &Intersection) -> (Sample<Ray>, Vector3<f64>, f64) {
    // cosθ
    let mut dn = in_ray.direction.dot(i.normal);
    let mut n = i.normal;
    // 真空屈折率
    let eta_v = 1.0;
    // 物体屈折率
    let eta = self.ior;
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
    let reflection_ray = Ray {direction: in_ray.direction - i.normal * (2.0 * in_ray.direction.dot(i.normal)), origin: i.position};
    // 判別式(全半射)
    let det = 1.0 - nn * nn * (1.0 - dn * dn);
    if det < 0.0 {
      // 全反射
      let brdf = self.brdf(in_ray.direction, reflection_ray.direction, n);
      let cos_term = reflection_ray.direction.dot(n);
      return (
        Sample {
          value: reflection_ray,
          pdf: DELTA_FUNC,
        },
        brdf,
        cos_term,
      );
    }
    // 屈折レイ
    let refraction_ray = Ray {direction: in_ray.direction * (nn) - n * (nn * dn + det.sqrt()), origin: i.position};
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
    // 反射率
    let fr = f_0 + (1.0 - f_0) * (1.0 - c).powi(5);
    // 透過率
    let ft = (1.0 - fr) * nni * nni;
    // ロシアンルーレットで反射と屈折のどちらかの寄与を取る
    let rr_prob = fr;
    if rand::random::<f64>() < rr_prob {
      // 反射
      let brdf = self.brdf(in_ray.direction, reflection_ray.direction, n) * fr;
      let cos_term = reflection_ray.direction.dot(n);
      (
        Sample {
          value: reflection_ray,
          pdf: DELTA_FUNC * rr_prob,
        },
        brdf,
        cos_term,
      )
    } else {
      // 屈折
      let brdf = self.brdf(in_ray.direction, refraction_ray.direction, n) * ft;
      let cos_term = refraction_ray.direction.dot(n);
      (
        Sample {
          value: refraction_ray,
          pdf: DELTA_FUNC * (1.0 - rr_prob),
        },
        brdf,
        cos_term,
      )
    }
  }
}

pub struct CookTorranceMaterial {
  // 鏡面反射率
  pub reflectance: Vector3<f64>,
  // 吸収係数
  pub absorptance: Vector3<f64>,
  // 屈折率
  pub ior: f64,
  // ラフネス
  pub roughness: f64,
}

impl CookTorranceMaterial {
  pub fn orienting_normal(&self, in_: Vector3<f64>, normal: Vector3<f64>) -> Vector3<f64> {
    // 物体の内外を考慮した法線方向から拡散反射面としての法線方向を求める
    if normal.dot(in_) > 0.0 {
      normal * -1.0
    } else {
      normal
    }
  }

  fn alpha(&self) -> f64 {
    self.roughness * self.roughness
  }

  // マイクロファセット分布関数 (Microfacet Distribution Functions)
  fn ndf(&self, h: Vector3<f64>, n: Vector3<f64>) -> f64 {
    // GGX
    let n_dot_h = n.dot(h);
    let a = self.alpha();
    let a2 = a * a;
    let b = (a2 - 1.0) * n_dot_h * n_dot_h + 1.0;
    a2 / (PI * b * b)
  }

  // 幾何減衰項 (Masking-Shadowing Fucntion)
  fn geometry(&self, i: Vector3<f64>, o: Vector3<f64>, h: Vector3<f64>, n: Vector3<f64>) -> f64 {
    // Height-Correlated Masking and Shadowing (Smith Joint Masking-Shadowing Function)
    self.g1(i, h, n) * self.g1(o, h, n)
  }

  #[allow(unused_variables)]
  fn g1(&self, x: Vector3<f64>, h: Vector3<f64>, n: Vector3<f64>) -> f64 {
    let a = self.alpha();
    let a2 = a * a;
    let x_dot_n = x.dot(n);
    2.0 * x.dot(n) / (x.dot(n) * (a2 + (1.0 - a2) * x_dot_n * x_dot_n).sqrt())
  }

  // フレネル項
  fn fresnel(&self, i: Vector3<f64>, h: Vector3<f64>) -> f64 {
    // 垂直入射での反射量
    // 真空屈折率
    let eta_v = 1.0;
    // 物体屈折率
    let eta = self.ior;
    // n1 - n2
    let nnn = eta_v - eta;
    // n1 + n2
    let nnp = eta + eta_v;
    let f_0 = (nnn * nnn) / (nnp * nnp);
    // Fresnelの式(Schlickの近似)より
    // 反射率
    f_0 + (1.0 - f_0) * (1.0 - i.dot(h)).powi(5)
  }
}

impl Material for CookTorranceMaterial {
  fn emission(&self) -> Vector3<f64> {
    Vector3::new(0.0, 0.0, 0.0)
  }

  fn rr_weight(&self) -> f64 {
    // 反射率のうち最大のものをつかう
    self.reflectance.x.max(self.reflectance.y).max(self.reflectance.z)
  }

  fn brdf(&self, i: Vector3<f64>, o: Vector3<f64>, n: Vector3<f64>) -> Vector3<f64> {
    let h = (o + i).norm();
    // Torrance-Sparrow model (PBRT p.546)
    // fr = FGD / 4(i.n)(o.n)
    let fr = self.fresnel(i, h) * self.geometry(i, o, h, n) * self.ndf(h, n) / (4.0 * i.dot(n) * o.dot(n));
    self.reflectance * fr
  }

  fn sample(&self, in_ray: &Ray, i: &Intersection) -> (Sample<Ray>, Vector3<f64>, f64) {
    let normal = self.orienting_normal(in_ray.direction, i.normal);
    // 反射点での法線方向を基準にした正規直交基底を生成
    let w = normal;
    let (u, v) = normal.orthonormal_basis();
    // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
    // (cosにしたがって重点的にサンプル)
    let sample = Sampler::hemisphere_cos_importance();
    let d = u * sample.x + v * sample.y + w * sample.z;
    // 新しいレイを作る
    let new_ray = Ray { direction: d, origin: i.position };
    // cos項
    let cos_term = d.dot(normal);
    // 確率密度関数
    // (cosにしたがって重点的にサンプル) cosθ / π
    let pdf = cos_term / PI;
    // in = view, out = light
    let brdf = i.material.brdf(-in_ray.direction, new_ray.direction, normal);
    (
      Sample {
        value: new_ray,
        pdf: pdf,
      },
      brdf,
      cos_term
    )
  }
}

pub struct PhongMaterial {
  // 反射率
  pub reflectance: Vector3<f64>,
  // ラフネス
  pub roughness: f64,
}

impl PhongMaterial {
  pub fn orienting_normal(&self, in_: Vector3<f64>, normal: Vector3<f64>) -> Vector3<f64> {
    // 物体の内外を考慮した法線方向から拡散反射面としての法線方向を求める
    if normal.dot(in_) > 0.0 {
      normal * -1.0
    } else {
      normal
    }
  }

  pub fn alpha(&self) -> f64 {
    self.roughness
  }
}

impl Material for PhongMaterial {
  fn emission(&self) -> Vector3<f64> {
    Vector3::new(0.0, 0.0, 0.0)
  }

  fn rr_weight(&self) -> f64 {
    // 反射率のうち最大のものをつかう
    self.reflectance.x.max(self.reflectance.y).max(self.reflectance.z)
  }

  fn brdf(&self, v_i: Vector3<f64>, l: Vector3<f64>, n: Vector3<f64>) -> Vector3<f64> {
    let v = - v_i;
    let r = v.reflect(n);
    let cos = r.dot(l);
    let a = self.alpha();
    // modified phong
    self.reflectance * ((a + 2.0) / (2.0 * PI) * cos.powf(a))
  }

  fn sample(&self, in_ray: &Ray, i: &Intersection) -> (Sample<Ray>, Vector3<f64>, f64) {
    let normal = self.orienting_normal(in_ray.direction, i.normal);
    let a = self.alpha();
    // 反射点での法線方向を基準にした正規直交基底を生成
    let w = normal;
    let (u, v) = normal.orthonormal_basis();
    // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
    // (brdfの分布にしたがって重点的にサンプル)
    let r1 = 2.0 * PI * rand::random::<f64>();
    let r2 = rand::random::<f64>();
    let t = r2.powf(2.0 / (a + 1.0));
    let ts = (1.0 - t).sqrt();

    let d = u * r1.cos() * ts + v * r1.sin() * ts + w * t.sqrt();
    // 新しいレイを作る
    let new_ray = Ray { direction: d, origin: i.position };
    // cos項
    let cos_term = d.dot(normal);
    // 確率密度関数
    // (brdfの分布にしたがって重点的にサンプル)
    let v = - in_ray.direction;
    let r = v.reflect(normal);
    let cos = r.dot(new_ray.direction);
    let pdf = (a + 2.0) / (2.0 * PI) * cos.powf(a);
    let brdf = i.material.brdf(in_ray.direction, new_ray.direction, normal);
    (
      Sample {
        value: new_ray,
        pdf: pdf,
      },
      brdf,
      cos_term
    )
  }
}
