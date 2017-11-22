extern crate ordered_float;

use aabb::AABB;
use shape::Shape;
use vector::Vector;
use ray::Ray;
use intersection::Intersection;
use std::sync::Arc;
use self::ordered_float::OrderedFloat;

pub struct Node {
  aabb: AABB,
  children: Vec<Arc<Shape + Send + Sync>>
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
  id: usize,
  object: Box<Shape + Send + Sync>,
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
  tree: Arc<Shape + Send + Sync>,
}

impl Shape for BVH {
  fn aabb(&self) -> AABB {
    self.tree.aabb()
  }

  fn intersect(&self, ray: &Ray) -> Option<Intersection> {
    self.tree.intersect(&ray)
  }
}

type Axis = Vec<Vec<Arc<Leaf>>>;

impl BVH {
  pub fn new(objects: Vec<Box<Shape + Send + Sync>>) -> BVH {
    // 実体
    let instance = objects.into_iter().enumerate().map( |(i, s)|
      Arc::new(Leaf {
        aabb: s.aabb(),
        id: i,
        object: s,
      })
    ).collect::<Vec<_>>();
    // 各軸に射影したAABBをソート
    let mut axis = (0..3).map( |_|
      instance.iter().cloned().collect::<Vec<_>>()
    ).collect::<Vec<_>>();
    for (i, a) in axis.iter_mut().enumerate() {
      a.sort_unstable_by_key( |v| OrderedFloat(v.aabb.center.to_array()[i]) )
    }
    BVH {
      tree: Self::construct(axis),
    }
  }

  fn construct(axis: Axis) -> Arc<Shape + Send + Sync> {
    // セットアップ
    let len = axis[0].len();
    let partition_count = 2;
    let size = len / partition_count;
    // 要素が1つのときは葉
    if len == 1 {
      return axis[0][0].clone();
    }
    // 全体のAABBを作成
    let aabb = AABB::merge(&axis[0].iter().map( |v| &v.aabb ).collect());
    // 最大のAABBの辺を基準にして分割する
    // 最大のAABBの辺のインデックスを取得
    let max_side_index = aabb
      .side()
      .to_array()
      .to_vec()
      .iter()
      .enumerate()
      .max_by_key( |&(_, v)| OrderedFloat(*v) )
      .map( |(i, _)| i )
      .unwrap_or(0);
    // 基準の軸を分割
    let partitioned_max_axis = (0..partition_count).map( |p| {
      let start = size * p;
      let end = if p == partition_count - 1 { len } else { size * (p + 1) };
      axis[max_side_index][start..end].iter().cloned().collect::<Vec<_>>()
    }).collect::<Vec<_>>();
    // 基準以外の軸も分割
    let partitioned_axis = (0..partition_count).map( |p| {
      (0..3).map( |i| {
        if i == max_side_index {
          partitioned_max_axis[p].iter().cloned().collect()
        } else {
          axis[i].iter().filter( |v|
            partitioned_max_axis[p].iter().find( |u| u.id == v.id).is_some()
          ).cloned().collect()
        }
      }).collect()
    }).collect::<Vec<_>>();
    // 再帰的に子要素を生成
    let children = partitioned_axis.into_iter().map( |a|
      Self::construct(a)
    ).collect();
    Arc::new(Node {
      aabb: aabb,
      children: children,
    })
  }
}
