use std::cmp::Ordering;
use rand::Rng;

use crate::{hit_record::HitRecord, Hit, HittableList, Interval, Ray, Vec3}; 

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min_corner: Vec3, 
    pub max_corner: Vec3,
}
impl BoundingBox {
    pub fn new(corner_a: Vec3, corner_b: Vec3) -> BoundingBox {
        let min_corner = Vec3::new(
            f64::min(corner_a.x(), corner_b.x()),
            f64::min(corner_a.y(), corner_b.y()),
            f64::min(corner_a.z(), corner_b.z()),
        );
        let max_corner = Vec3::new(
            f64::max(corner_a.x(), corner_b.x()),
            f64::max(corner_a.y(), corner_b.y()),
            f64::max(corner_a.z(), corner_b.z()),
        );
        BoundingBox{min_corner, max_corner}
    }
    pub fn surrounding(box_a: BoundingBox, box_b: BoundingBox) -> BoundingBox{
        let min_corner = Vec3::new(
            f64::min(box_a.min_corner.x(), box_b.min_corner.x()),
            f64::min(box_a.min_corner.y(), box_b.min_corner.y()),
            f64::min(box_a.min_corner.z(), box_b.min_corner.z()),
        );
        let max_corner = Vec3::new(
            f64::max(box_a.max_corner.x(), box_b.max_corner.x()),
            f64::max(box_a.max_corner.y(), box_b.max_corner.y()),
            f64::max(box_a.max_corner.z(), box_b.max_corner.z()),
        );
        BoundingBox{min_corner, max_corner}
    }
    fn hit(&self, ray: &Ray, range: Interval) -> bool {
        for dim in 0..3 {
            let inv_d = 1.0/ray.direction[dim];
            let mut t0 = (self.min_corner[dim] - ray.origin[dim]) * inv_d;
            let mut t1 = (self.max_corner[dim] - ray.origin[dim]) * inv_d;
            if inv_d.is_sign_negative() {
                std::mem::swap(&mut t0, &mut t1);

            }
            let t_min = f64::min(t0, range.min);
            let t_max = f64::max(t1, range.max);
            if t_max <= t_min{
                return true; 
            }
        }
        return false;
    }
}


#[derive(Clone)]
pub enum BVHNodeType<'a>{
    BVHNode(Box<BVHNode<'a>>),
    Hittable(&'a dyn Hit)
}

impl <'a>BVHNodeType<'a> {
    fn hit(&self, ray: &Ray, range: Interval) -> Option<HitRecord> {
        match self {
            BVHNodeType::BVHNode(n) =>  n.hit(ray, range),
            BVHNodeType::Hittable(h) => {
                if h.bounding_volume().hit(ray, range) {
                    h.hit(ray, range)
                } else {
                    None
                }
            }
        }
    }
    pub fn bounding_volume(&self) -> BoundingBox {
        match self {
            BVHNodeType::BVHNode(n) =>  n.bounding_volume(),
            BVHNodeType::Hittable(h) => h.bounding_volume(),
        }
    }
}

#[derive(Clone)]
pub struct BVHNode<'a> {
    left:  BVHNodeType<'a>,
    right: BVHNodeType<'a>,
}

impl <'a>BVHNode<'_> {
    pub fn new(objects: &mut HittableList<'a>, start: usize, end: usize) -> BVHNode<'a> {
        let mut rng = rand::thread_rng();
        let axis: usize = rng.gen_range(0..3);
        let object_span = end - start;
        if object_span == 1 {
            return BVHNode{
                left: BVHNodeType::Hittable(objects.list[start]),
                right: BVHNodeType::Hittable(objects.list[start]),
            }
        }
        if object_span == 2 {
            if Self::is_closer(objects.list[start], objects.list[start+1], axis).is_lt() {
                return BVHNode{
                    left: BVHNodeType::Hittable(objects.list[start]),
                    right: BVHNodeType::Hittable(objects.list[start+1]),
                }
            } else {
                return BVHNode{
                    left: BVHNodeType::Hittable(objects.list[start+1]),
                    right: BVHNodeType::Hittable(objects.list[start]),
                }
            }
        }

        objects.list[start..end].sort_by(|a, b| Self::is_closer(*a, *b, axis));
        let mid = start + object_span / 2;
        let left =  BVHNodeType::BVHNode( Box::new(BVHNode::new(objects, start, mid)));
        let right = BVHNodeType::BVHNode( Box::new(BVHNode::new(objects, mid, end)));
        return BVHNode {left, right};
    }

    pub fn is_closer(obj_a: &'a dyn Hit, obj_b: &dyn Hit, axis: usize) -> Ordering {
        return obj_a.bounding_volume().min_corner[axis].partial_cmp(&obj_b.bounding_volume().min_corner[axis]).expect("no ordering found");
    }
}
impl<'a> Hit for BVHNode<'a> {
    fn hit(&self, ray: &Ray, range: Interval) -> Option<HitRecord> {
        if !self.bounding_volume()
            .hit(ray, range) {
            return None;
        }
        let left_hit =  self.left.hit(ray, range);
        let right_hit = self.right.hit(ray, range);
        return match (left_hit, right_hit) {
            (None, None) =>         None,
            (Some(_), None) =>      left_hit,
            (None, Some(_)) =>      right_hit,
            (Some(lh), Some(rh)) => {
                if lh.is_closer_than(rh) {
                    left_hit
                } else {
                    right_hit
                }
            },
            
        }
    }
    fn bounding_volume(&self) -> BoundingBox {
        return BoundingBox::surrounding(
            self.left.bounding_volume(),
            self.right.bounding_volume(),
        )
    }
    fn pdf_value(&self, origin: Vec3, direction: Vec3) -> f64 {
        return 0.0;
    }
    fn random_to_surface(&self, origin: Vec3) -> Option<Vec3> {
        return None;
    }
}

