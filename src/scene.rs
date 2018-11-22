extern crate rand;

use math::vector::*;
use sky::Sky;
use ray::Ray;
use objects::Objects;
use shape::Shape;
use intersection::Intersection;
use constant::*;

pub struct Scene<'a> {
  pub objects: Objects<'a>,
  pub depth: usize,
  pub depth_limit: usize,
  pub sky: Box<dyn Sky + Send + Sync>,
  pub no_direct_emitter: bool,
}

impl<'a> Scene<'a> {
  pub fn radiance(&self, ray: &Ray) -> Vector3 {
    self.radiance_recursive(ray, 0)
  }

  fn radiance_recursive(&self, ray: &Ray, depth: usize) -> Vector3 {
    // すべてのオブジェクトと当たり判定を行う
    let maybe_intersect = self.objects.intersect(&ray);
    // 当たらなかった場合は背景色を返す
    match maybe_intersect {
      None => self.sky.radiance(&ray),
      Some(i) => self.intersect_radiance(&i, &ray, depth),
    }
  }

  pub fn radiance_nee(&self, ray: &Ray) -> Vector3 {
    self.radiance_nee_recursive(ray, 0, false)
  }

  fn radiance_nee_recursive(&self, ray: &Ray, depth: usize, no_emission: bool) -> Vector3 {
    // すべてのオブジェクトと当たり判定を行う
    let maybe_intersect = self.objects.intersect(&ray);
    // 当たらなかった場合は背景色を返す
    match maybe_intersect {
      None => self.sky.radiance(&ray),
      Some(i) => self.intersect_radiance_nee(&i, &ray, depth, no_emission),
    }
  }

  pub fn normal(&self, ray: &Ray) -> Vector3 {
    let maybe_intersect = self.objects.intersect(&ray);
    match maybe_intersect {
      None => Vector3::zero(),
      Some(i) => i.normal / 2.0 + Vector3::new(0.5, 0.5, 0.5),
    }
  }

  pub fn depth(&self, ray: &Ray) -> f32 {
    let maybe_intersect = self.objects.intersect(&ray);
    match maybe_intersect {
      None => 0.0,
      Some(i) => i.distance,
    }
  }

  fn russian_roulette(&self, init: f32, depth: usize) -> f32 {
    // 再帰抑制用のロシアンルーレットの確率を決定する
    let mut continue_rr_prob = init;
    // スタックオーバーフロー対策のために反射回数の限界値を超えたら極端に確率を下げる
    if depth > self.depth_limit {
      continue_rr_prob *= (0.5f32).powi((depth - self.depth_limit) as i32);
    }
    // 最初の数回の反射では必ず次のパスをトレースするようにする
    if depth <= self.depth && continue_rr_prob > 0.0 {
      continue_rr_prob = 1.0;
    }
    continue_rr_prob
  }

  fn material_interaction_radiance<F>(&self, i: &Intersection, ray: &Ray, f: F) -> Vector3
    where F: Fn(Ray) -> Vector3
  {
    let out_ = -ray.direction;
    // BRDFに応じたサンプリング
    let sample = i.material.sample(out_, i.normal);
    let in_ = sample.value;
    let pdf = sample.pdf;
    // BRDF
    let brdf = i.material.brdf(out_, in_, i.normal);
    // コサイン項
    let cos = in_.dot(i.normal);
    // assert!(brdf.x * cos < 1.0 && brdf.x * cos > 0.0, "{} {} {}", brdf.x * cos, brdf.x, cos);
    // 放射輝度の圧縮で透過の場合は1を超えてもおかしくない
    let new_ray = Ray {
      direction: in_,
      origin: i.position,
    };
    // 再帰的にレイを追跡
    let l_i = f(new_ray);
    // レンダリング方程式にしたがって放射輝度を計算する
    brdf * l_i * cos / pdf
  }

  fn direct_light_radiance(&self, i: &Intersection, ray: &Ray) -> Vector3 {
    if i.material.emission().sqr_norm() > 0.0 || !self.objects.has_emission() {
      // 交差したマテリアルが放射を持っているとき、NEE対象の光源が存在しないとき
      return Vector3::zero()
    }
    // 光源上から1点をサンプリング (確率密度は面積測度)
    let direct_sample = self.objects.sample_emission();
    // 交差した座標と光源上の1点のパスを接続
    let direct_path = direct_sample.value - i.position;
    // 可視関数のテストレイを生成
    let direct_ray = Ray {
      origin: i.position,
      direction: direct_path.normalize(),
    };
    // 面に対して可視であるかテスト
    let point_in = direct_ray.direction;
    let point_out = -ray.direction;
    let point_normal = i.material.orienting_normal(point_out, i.normal);
    if point_in.dot(point_normal) <= 0.0 {
      // レイの入射方向とは逆の方向にレイを接続した場合は遮蔽
      return Vector3::zero()
    }
    // 直接光のみのサンプリングなので可視の場合のみ寄与
    match self.objects.intersect(&direct_ray) {
      Some(direct_i) => {
        if (direct_i.distance - direct_path.norm()).abs() > EPS {
          // 接続したパスとテストレイの距離が違う場合は遮蔽
          return Vector3::zero()
        }
        let light_out = -direct_ray.direction;
        let light_normal = direct_i.normal;
        let light_cos = light_out.dot(light_normal);
        if light_cos <= 0.0 {
          // 光源の裏面は寄与なし
          return Vector3::zero()
        }
        // ジオメトリターム (測度の変換)
        let point_cos = point_in.dot(point_normal);
        let g_term = point_cos * light_cos / direct_path.sqr_norm();
        // BRDF
        let brdf = i.material.brdf(point_out, point_in, point_normal);
        let l_i = direct_i.material.emission();
        let pdf = direct_sample.pdf;
        brdf * l_i * g_term / pdf
      },
      None => Vector3::zero(),
    }
  }

  fn intersect_radiance(&self, i: &Intersection, ray: &Ray, depth: usize) -> Vector3 {
    // 放射
    let l_e = if !(self.no_direct_emitter && depth == 0) && (-ray.direction).dot(i.normal) > 0.0 {
      i.material.emission()
    } else {
      Vector3::zero()
    };
    // ロシアンルーレットで再帰を抑制
    let continue_rr_prob = self.russian_roulette(i.material.weight(), depth);
    if continue_rr_prob != 1.0 && rand::random::<f32>() >= continue_rr_prob {
      return l_e;
    }
    // マテリアルに応じたサンプリングによる寄与
    let material_radiance = self.material_interaction_radiance(&i, &ray, |new_ray| {
      self.radiance_recursive(&new_ray, depth + 1)
    });
    // ロシアンルーレットを用いた評価で期待値を満たすために確率で割る (再帰抑制用)
    return l_e + material_radiance / continue_rr_prob;
  }

  fn intersect_radiance_nee(&self, i: &Intersection, ray: &Ray, depth: usize, no_emission: bool) -> Vector3 {
    // 放射
    let l_e = if !(self.no_direct_emitter && depth == 0) && !no_emission && (-ray.direction).dot(i.normal) > 0.0 {
      i.material.emission()
    } else {
      Vector3::zero()
    };
    // ロシアンルーレットで再帰を抑制
    let continue_rr_prob = self.russian_roulette(i.material.weight(), depth);
    if continue_rr_prob != 1.0 && rand::random::<f32>() >= continue_rr_prob {
      return l_e;
    }
    // 直接光のサンプリングによる寄与
    let direct_light_radiance = self.direct_light_radiance(&i, &ray);
    // マテリアルに応じたサンプリングによる寄与
    let material_radiance = self.material_interaction_radiance(&i, &ray, |new_ray| {
      self.radiance_nee_recursive(&new_ray, depth + 1, true)
    });
    // ロシアンルーレットを用いた評価で期待値を満たすために確率で割る (再帰抑制用)
    return l_e + (direct_light_radiance + material_radiance) / continue_rr_prob;
  }
}
