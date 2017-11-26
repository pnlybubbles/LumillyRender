extern crate ordered_float;

use aabb::AABB;
use shape::*;
use ray::Ray;
use intersection::Intersection;
use std::sync::Arc;
use self::ordered_float::OrderedFloat;

pub struct Node {
  aabb: AABB,
  children: Vec<Arc<Shape + Sync + Send>>
}

impl Shape for Node {
  fn aabb(&self) -> AABB {
    self.aabb.clone()
  }

  fn intersect(&self, ray: &Ray) -> Option<Intersection> {
    if self.aabb.is_intersect(&ray) {
      self.children.iter().flat_map(|v| v.intersect(&ray)).min_by(
        |a, b| {
          a.distance.partial_cmp(&b.distance).unwrap()
        },
      )
    } else {
      None
    }
  }
}

pub struct Leaf {
  aabb: AABB,
  object: Arc<SurfaceShape + Sync + Send>,
}

impl Shape for Leaf {
  fn aabb(&self) -> AABB {
    self.aabb.clone()
  }

  fn intersect(&self, ray: &Ray) -> Option<Intersection> {
    if self.aabb.is_intersect(&ray) {
      self.object.intersect(&ray)
    } else {
      None
    }
  }
}

pub struct BVH {
  tree: Arc<Shape + Sync + Send>,
}

impl Shape for BVH {
  fn aabb(&self) -> AABB {
    self.tree.aabb()
  }

  fn intersect(&self, ray: &Ray) -> Option<Intersection> {
    self.tree.intersect(&ray)
  }
}

impl BVH {
  pub fn new(objects: Vec<Arc<SurfaceShape + Sync + Send>>) -> BVH {
    // 実体
    let mut instance = objects.into_iter().map( |s|
      Arc::new(Leaf {
        aabb: s.aabb(),
        object: s,
      })
    ).collect::<Vec<_>>();
    BVH {
      tree: Self::construct(&mut instance, 0),
    }
  }

  fn construct(list: &mut [Arc<Leaf>], depth: usize) -> Arc<Shape + Sync + Send> {
    // セットアップ
    let len = list.len();
    let partition_count = 2usize;
    let size = (len as f64 / partition_count as f64).ceil() as usize;
    // 要素が1つのときは葉
    if len == 1 {
      return list[0].clone();
    }
    // 全体のAABBを作成
    let aabb = AABB::merge(&list.iter().map( |v| &v.aabb ).collect());
    // 最大のAABBの辺を基準にして分割する
    // 最大のAABBの辺のインデックスを取得
    let max_side_index = aabb.side().to_array().to_vec().iter()
      .enumerate().max_by_key( |&(_, v)| OrderedFloat(*v) )
      .map( |(i, _)| i ).unwrap_or(0);
    // 基準の軸でソート
    list.sort_unstable_by_key( |v| OrderedFloat(v.aabb.center[max_side_index]) );
    // 再帰的に子要素を生成
    let children = (0..partition_count).flat_map( |p| {
      let start = size * p;
      let end = if p == partition_count - 1 { len } else { size * (p + 1) };
      if start >= len {
        None
      } else {
        Some(Self::construct(&mut list[start..end], depth + 1))
      }
    }).collect();
    Arc::new(Node {
      aabb: aabb,
      children: children,
    })
  }
}
