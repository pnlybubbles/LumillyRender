extern crate ordered_float;

use aabb::AABB;
use shape::*;
use ray::Ray;
use intersection::Intersection;
use self::ordered_float::OrderedFloat;

#[derive(Clone)]
struct Leaf {
  aabb: AABB,
  index: usize,
}

trait Branch {
  fn may_intersect(&self, &Ray, &mut Vec<usize>);
  fn aabb(&self) -> &AABB;
}

impl Branch for Leaf {
  fn may_intersect(&self, ray: &Ray, candidate: &mut Vec<usize>) {
    if self.aabb.is_intersect(ray) {
      candidate.push(self.index)
    }
  }

  fn aabb(&self) -> &AABB {
    &self.aabb
  }
}

struct Node {
  aabb: AABB,
  left: Box<Branch + Send + Sync>,
  right: Box<Branch + Send + Sync>,
}

impl Branch for Node {
  fn may_intersect(&self, ray: &Ray, mut candidate: &mut Vec<usize>) {
    if self.aabb.is_intersect(ray) {
      self.left.may_intersect(ray, &mut candidate);
      self.right.may_intersect(ray, &mut candidate);
    }
  }

  fn aabb(&self) -> &AABB {
    &self.aabb
  }
}

pub struct BVH<'a> {
  list: &'a Vec<Box<SurfaceShape + Send + Sync>>,
  root: Box<Branch + Send + Sync>,
}

impl<'a> BVH<'a> {
  pub fn new(list: &Vec<Box<SurfaceShape + Send + Sync>>) -> BVH {
    let mut leaf = list.iter().enumerate().map( |(i, v)| Leaf {
      aabb: v.aabb().clone(),
      index: i,
    }).collect::<Vec<_>>();
    let root = Self::construct(&mut leaf);
    BVH {
      list: list,
      root: root, 
    }
  }

  fn construct(list: &mut [Leaf]) -> Box<Branch + Send + Sync> {
    // TODO
    let t_aabb = 1.0;
    let t_tri = 2.0;
    // セットアップ
    let n = list.len();
    // 要素が1つのときは葉
    if n == 1 {
      return box list[0].clone();
    }
    // 全体のAABB
    let mut aabb = AABB::empty();
    // SAHに基づいた最良の分割軸とインデックスを取得
    let (partition_axis, partition_index, _) = (0..3).map( |axis| {
      // 基準の軸でソート
      list.sort_unstable_by_key( |v| {
        OrderedFloat(v.aabb.center[axis])
      });
      // S1のAABBの表面積
      let mut s1_aabb = list[0].aabb.clone();
      let mut s1_a = Vec::with_capacity(n - 1);
      for leaf in list[0..n].iter() {
        s1_aabb = s1_aabb.merge_with(&leaf.aabb);
        s1_a.push(s1_aabb.surface_area());
      }
      // S2のAABBの表面積
      let mut s2_aabb = list[n - 1].aabb.clone();
      let mut s2_a = Vec::with_capacity(n - 1);
      for leaf in list[1..].iter().rev() {
        s2_aabb = s2_aabb.merge_with(&leaf.aabb);
        s2_a.push(s2_aabb.surface_area());
      }
      // 全体SのAABBの表面積
      aabb = s1_aabb.merge_with(&list[n - 1].aabb);
      let s_a = aabb.surface_area();
      // SAHのスコアを評価
      (0..n - 1).map( |i| {
        // ポリゴン数
        let s1_n = (i + 1) as f32;
        let s2_n = (n - i - 1) as f32;
        // Surface Area Heuristics
        // T = 2 * T_aabb + (A(S1) * N(S1) + A(S2) * N(S2)) * T_tri / A(S)
        OrderedFloat(2.0 * t_aabb + (s1_a[i] * s1_n + s2_a[n - i - 2] * s2_n) * t_tri / s_a)
      }).enumerate().min_by_key( |&(_, t)| t ).unwrap()
    }).enumerate().min_by_key( |&(_, (_, t))| t ).map( |(a, (i, t))| (a, i + 1, t) ).unwrap();
    // 基準の軸でソート
    list.sort_unstable_by_key( |v| {
      OrderedFloat(v.aabb.center[partition_axis])
    });
    // 再帰的に子要素を生成
    debug_assert!(partition_index != 0 && partition_index != n);
    let left = Self::construct(&mut list[0..partition_index]);
    let right = Self::construct(&mut list[partition_index..]);
    box Node {
      aabb: aabb,
      left: left,
      right: right,
    }
  }
}

impl<'a> Shape for BVH<'a> {
  fn intersect(&self, ray: &Ray) -> Option<Intersection> {
    let mut candidate = Vec::new();
    self.root.may_intersect(ray, &mut candidate);
    candidate.iter().flat_map( |&i| {
      self.list[i].intersect(&ray)
    }).min_by(
      |a, b| {
        a.distance.partial_cmp(&b.distance).unwrap()
      },
    )
  }

  fn aabb(&self) -> &AABB {
    self.root.aabb()
  }
}
