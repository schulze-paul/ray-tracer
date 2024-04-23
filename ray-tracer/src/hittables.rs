use std::cmp::Ordering;

use rand::Rng;

use crate::{HitRecord, Material, Ray, Vec3, HitType};
use crate::dot;



#[derive(Debug, Clone)]
pub enum Hittable <'a>{
    HittableList(HittableListStruct<'a>),
    Sphere(SphereStruct<'a>),
    BoundingBox(BoundingBoxStruct),
    BHVNode(BVHNodeStruct<'a>),
    XYRectangle(XYRectangleStruct<'a>),
    XZRectangle(XZRectangleStruct<'a>),
    YZRectangle(YZRectangleStruct<'a>),
    Cuboid(CuboidStruct<'a>),

}

#[derive(Debug, Clone)]
pub struct SphereStruct <'a>{
    pub center: Vec3, 
    pub radius: f64, 
    pub material: &'a Material,
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingBoxStruct {
    pub min_corner: Vec3, 
    pub max_corner: Vec3,
}

pub trait Hit<'a> {
    fn hit(&'a self, ray: &'a Ray, range: [f64;2]) -> HitType;
}

pub trait MaterialTrait {
    fn material(&self) -> Option<&Material>;
}

pub trait BoundingVolumeTrait {
    fn bounding_volume(&self) -> Option<BoundingBoxStruct>;
}

impl <'a>Hit<'a> for Hittable<'a> {
    fn hit(&'a self, ray: &'a Ray, range: [f64;2]) -> HitType {
        match self {
            Hittable::Sphere(s) =>       s.hit(ray, range),
            Hittable::HittableList(l) => l.hit(ray, range),
            Hittable::BoundingBox(b) =>  b.hit(ray, range),
            Hittable::BHVNode(n) =>      n.hit(ray, range),
            Hittable::XYRectangle(r) =>  r.hit(ray, range),
            Hittable::XZRectangle(r) =>  r.hit(ray, range),
            Hittable::YZRectangle(r) =>  r.hit(ray, range),
            Hittable::Cuboid(c) =>       c.hit(ray, range),
        }
    }
}

impl <'a>MaterialTrait for Hittable<'_> {
    fn material(&self) -> Option<&Material> {
        match self {
            Hittable::Sphere(s) =>       Some(&s.material),
            Hittable::HittableList(_) => None,
            Hittable::BoundingBox(_) =>  None,
            Hittable::BHVNode(_) =>      None,
            Hittable::XYRectangle(r) =>  Some(&r.material),
            Hittable::XZRectangle(r) =>  Some(&r.material),
            Hittable::YZRectangle(r) =>  Some(&r.material),
            Hittable::Cuboid(c)      =>  Some(&c.material),
        }
    }
}

impl BoundingVolumeTrait for Hittable<'_> {
    fn bounding_volume(&self) -> Option<BoundingBoxStruct> {
        match self {
            Hittable::Sphere(s) =>       Some(s.bounding_volume()),
            Hittable::HittableList(l) => l.bounding_volume(),
            Hittable::BoundingBox(b) =>  Some(b.bounding_volume()),
            Hittable::BHVNode(n) =>      Some(n.bounding_volume()),
            Hittable::XYRectangle(r) =>  Some(r.bounding_volume()),
            Hittable::XZRectangle(r) =>  Some(r.bounding_volume()),
            Hittable::YZRectangle(r) =>  Some(r.bounding_volume()),
            Hittable::Cuboid(c) =>       Some(c.bounding_volume()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HittableListStruct <'a>{
    pub list: Vec<&'a Hittable<'a>>
}

impl <'a>HittableListStruct<'a> {
    pub fn new() -> HittableListStruct<'a> {
        return HittableListStruct{
            list: Vec::new() 
        }
    }
    pub fn from(list: Vec<&'a Hittable<'a>>) -> HittableListStruct<'a> {
        return HittableListStruct{list}
    }
    pub fn push(mut self, hittable: &'a Hittable<'a>) -> HittableListStruct<'a> {
        self.list.push(hittable);
        return self;
    }
    fn hit(&'a self, ray: &'a Ray, range: [f64;2]) -> HitType {
        let mut closest_hit_record = HitType::None;
        for hittable in &self.list {
            let hit_record = hittable.hit(ray, range);
            match hit_record {
                HitType::Hit(h) => {
                    match closest_hit_record {
                        HitType::Hit(ref c) => {
                            if h.t_hit < c.t_hit {
                                closest_hit_record = HitType::Hit(h) 
                            }
                        }
                        _ => closest_hit_record = HitType::Hit(h),
                    }
                }
                _ => continue,
            }
        }
        return closest_hit_record;
    }
    fn bounding_volume(&self) -> Option<BoundingBoxStruct> {
        if self.list.len() == 0 {
            return None;
        }
        let mut bbox: BoundingBoxStruct = self.list[0].bounding_volume()?;

        for object in &self.list {
            bbox = BoundingBoxStruct::surrounding(
                bbox, 
                object.bounding_volume()?
            )
        }
        return Some(bbox);
    }
}

impl <'a>SphereStruct<'_> {
    pub fn new(center: Vec3, radius: f64, material: &Material) -> SphereStruct{
        SphereStruct{center, radius, material}
    }
    pub fn hit(&'a self, ray: &'a Ray, range: [f64;2]) -> HitType {
        let oc = ray.origin - self.center;              // origin to center
        let a = dot(ray.direction, ray.direction);      // direction squared
        let b = 2.0 * dot(oc, ray.direction);           // 2 * alignment of center direction and ray direction
        let c = dot(oc,oc) - self.radius * self.radius; // center distance squared - radius squared
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return HitType::None; // no hit
        }
        // ray in direction of sphere
        let mut hit_at_t = (-b - discriminant.sqrt()) / (2.0 * a);
        if !(hit_at_t < range[1] && hit_at_t > range[0]) {
            // not in range, try other hit
            hit_at_t = (-b + discriminant.sqrt()) / (2.0 * a);
            if !(hit_at_t < range[1] && hit_at_t > range[0])
            {
                // not in range, no hit
                return HitType::None;
            }
        }
        let normal = self.get_normal(ray.at(hit_at_t));
        let rec = HitRecord::new(hit_at_t ,ray, normal)
            .with_material(self.material);
        return HitType::Hit(rec);

    }
    fn get_normal(&self, point_on_surface: Vec3) -> Vec3 {
        (point_on_surface - self.center) / self.radius
    }
    pub fn bounding_volume(&self) -> BoundingBoxStruct {
        BoundingBoxStruct::new(
               self.center - self.radius*Vec3::ones(),
               self.center + self.radius*Vec3::ones()
        )
    }
}

impl BoundingBoxStruct {
    pub fn new(corner_a: Vec3, corner_b: Vec3) -> BoundingBoxStruct {
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
        BoundingBoxStruct{min_corner, max_corner}
    }
    pub fn surrounding(box_a: BoundingBoxStruct, box_b: BoundingBoxStruct) -> BoundingBoxStruct{
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
        BoundingBoxStruct{min_corner, max_corner}
    }
    pub fn bounding_volume(self) -> BoundingBoxStruct {
        self
    }


    pub fn hit(self, ray: &Ray, range: [f64;2]) -> HitType {
        for dim in 0..3 {
            let inv_d = 1.0/ray.direction[dim];
            let mut t0 = (self.min_corner[dim] - ray.origin[dim]) * inv_d;
            let mut t1 = (self.max_corner[dim] - ray.origin[dim]) * inv_d;
            if inv_d.is_sign_negative() {
                std::mem::swap(&mut t0, &mut t1);

            }
            let t_min = 
                if t0 > range[0] {t0} else {range[0]};
            let t_max = 
                if t1 < range[0] {t1} else {range[0]};
            if t_max <= t_min{
                return HitType::BoundingHit
            }
        }
        return HitType::None
    }
}

#[derive(Debug, Clone)]
pub enum BVHNodeType<'a>{
    BVHNode(Box<BVHNodeStruct<'a>>),
    Hittable(&'a Hittable<'a>)
}

impl <'a>BVHNodeType<'_> {
    fn hit(&'a self, ray: &'a Ray, range: [f64;2]) -> HitType {
        match self {
            BVHNodeType::BVHNode(n) =>  n.hit(ray, range),
            BVHNodeType::Hittable(h) => h.hit(ray, range),
        }
    }
    pub fn bounding_volume(&self) -> BoundingBoxStruct {
        match self {
            BVHNodeType::BVHNode(n) =>  n.bounding_volume(),
            BVHNodeType::Hittable(h) => h.bounding_volume()
                .expect("BVHNode hittable has no bounging volume"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BVHNodeStruct<'a> {
    left:  BVHNodeType<'a>,
    right: BVHNodeType<'a>,
}

impl <'a>BVHNodeStruct<'_> {
    pub fn new(objects: &mut HittableListStruct<'a>, start: usize, end: usize) -> BVHNodeStruct<'a> {
        let mut rng = rand::thread_rng();
        let axis: usize = rng.gen_range(0..3);
        let object_span = end - start;
        if object_span == 1 {
            return BVHNodeStruct{
                left: BVHNodeType::Hittable(objects.list[start]),
                right: BVHNodeType::Hittable(objects.list[start]),
            }
        }
        if object_span == 2 {
            if Self::is_closer(objects.list[start], objects.list[start+1], axis).is_lt() {
                return BVHNodeStruct{
                    left: BVHNodeType::Hittable(objects.list[start]),
                    right: BVHNodeType::Hittable(objects.list[start+1]),
                }
            } else {
                return BVHNodeStruct{
                    left: BVHNodeType::Hittable(objects.list[start+1]),
                    right: BVHNodeType::Hittable(objects.list[start]),
                }
            }
        }

        objects.list[start..end].sort_by(|a, b| Self::is_closer(a, b, axis));
        let mid = start + object_span / 2;
        let left =  BVHNodeType::BVHNode( Box::new(BVHNodeStruct::new(objects, start, mid)));
        let right = BVHNodeType::BVHNode( Box::new(BVHNodeStruct::new(objects, mid, end)));
        return BVHNodeStruct {left, right};
    }

    pub fn is_closer(obj_a: &Hittable, obj_b: &Hittable, axis: usize) -> Ordering {
        match obj_a.bounding_volume().zip(obj_b.bounding_volume()) {
            None => panic!("No bounding box in bvhnode init"),
            Some((a, b)) => {
                return a.min_corner[axis].partial_cmp(&b.min_corner[axis]).expect("no ordering found");
            }
        }

    }
    fn hit(&'a self, ray: &'a Ray, range: [f64;2]) -> HitType {
        match self.bounding_volume().hit(ray, range) {
            HitType::None => return HitType::None,
            _ => {
                let left_hit =  self.left.hit(ray, range);
                let right_hit = self.right.hit(ray, range);
                match left_hit {
                    HitType::None => {
                        match right_hit {
                            HitType::Hit(h) =>      return HitType::Hit(h),
                            HitType::BoundingHit => return HitType::BoundingHit,
                            HitType::None =>        return HitType::None,
                        }
                    },
                    HitType::BoundingHit => {
                        match right_hit {
                            HitType::Hit(h) => return HitType::Hit(h),
                            _ =>               return HitType::BoundingHit,
                        }
                    },
                    HitType::Hit(lh) => {
                        match right_hit {
                            HitType::Hit(rh) => {
                                if lh.t_hit < rh.t_hit {
                                    return HitType::Hit(lh)
                                } else {
                                    return HitType::Hit(rh)
                                }
                            },
                            _ => return HitType::Hit(lh),
                        }
                    },
                }
            }
        }


    }
    pub fn bounding_volume(&self) -> BoundingBoxStruct {
        return BoundingBoxStruct::surrounding(
            self.left.bounding_volume(),
            self.right.bounding_volume(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct XYRectangleStruct<'a> {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    k:  f64,
    material: &'a Material
}

impl <'a>XYRectangleStruct<'_> {
    pub fn new(x0: f64, x1: f64, y0: f64, y1: f64, k: f64, material: &'a Material) -> XYRectangleStruct<'a> {
        let x_min = f64::min(x0, x1);
        let y_min = f64::min(y0, y1);
        let x_max = f64::max(x0, x1);
        let y_max = f64::max(y0, y1);
        XYRectangleStruct{x0: x_min, x1: x_max, y0: y_min, y1: y_max, k, material}
    }
    fn hit(&'a self, ray: &'a Ray, range: [f64;2]) -> HitType {
        let t = (self.k - ray.origin.z()) / ray.direction.z();
        if t < range[0] || t > range[1] {
            return HitType::None
        }
        let x = ray.origin.x() + t * ray.direction.x();
        let y = ray.origin.y() + t * ray.direction.y();
        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return HitType::None;
        }
        let mut normal = Vec3::z_hat();
        if dot(ray.direction, normal) > 0.0 {
            normal = -normal;
        }
        return HitType::Hit(HitRecord::new(t, ray, normal));
    }
    fn bounding_volume(&self) -> BoundingBoxStruct {
        return BoundingBoxStruct::new(
            Vec3::new(self.x0, self.y0, self.k-0.0001),
            Vec3::new(self.x1, self.y1, self.k+0.0001),
        )
    }
}

#[derive(Debug, Clone)]
pub struct XZRectangleStruct<'a> {
    x0: f64,
    x1: f64,
    z0: f64,
    z1: f64,
    k:  f64,
    material: &'a Material
}

impl <'a>XZRectangleStruct<'_> {
    pub fn new(x0: f64, x1: f64, z0: f64, z1: f64, k: f64, material: &'a Material) -> XZRectangleStruct<'a> {
        let x_min = f64::min(x0, x1);
        let z_min = f64::min(z0, z1);
        let x_max = f64::max(x0, x1);
        let z_max = f64::max(z0, z1);
        XZRectangleStruct{x0: x_min, x1: x_max, z0: z_min, z1: z_max, k, material}
    }
    fn hit(&'a self, ray: &'a Ray, range: [f64;2]) -> HitType {
        let t = (self.k - ray.origin.y()) / ray.direction.y();
        if t < range[0] || t > range[1] {
            return HitType::None
        }
        let x = ray.origin.x() + t * ray.direction.x();
        let z = ray.origin.z() + t * ray.direction.z();
        if x < self.x0 || x > self.x1 || z < self.z0 || z > self.z1 {
            return HitType::None;
        }
        let mut normal = Vec3::y_hat();
        if dot(ray.direction, normal) > 0.0 {
            normal = -normal;
        }
        return HitType::Hit(HitRecord::new(t, ray, normal));
    }
    fn bounding_volume(&self) -> BoundingBoxStruct {
        return BoundingBoxStruct::new(
            Vec3::new(self.x0, self.z0, self.k-0.0001),
            Vec3::new(self.x1, self.z1, self.k+0.0001),
        )
    }
}


#[derive(Debug, Clone)]
pub struct YZRectangleStruct<'a> {
    y0: f64,
    y1: f64,
    z0: f64,
    z1: f64,
    k:  f64,
    material: &'a Material
}

impl <'a>YZRectangleStruct<'_> {
    pub fn new(y0: f64, y1: f64, z0: f64, z1: f64, k: f64, material: &'a Material) -> YZRectangleStruct<'a> {
        let y_min = f64::min(y0, y1);
        let z_min = f64::min(z0, z1);
        let y_max = f64::max(y0, y1);
        let z_max = f64::max(z0, z1);
        YZRectangleStruct{y0: y_min, y1: y_max, z0: z_min, z1: z_max, k, material}
    }
    fn hit(&'a self, ray: &'a Ray, range: [f64;2]) -> HitType {
        let t = (self.k - ray.origin.x()) / ray.direction.x();
        if t < range[0] || t > range[1] {
            return HitType::None
        }
        let y = ray.origin.y() + t * ray.direction.y();
        let z = ray.origin.z() + t * ray.direction.z();
        if y < self.y0 || y > self.y1 || z < self.z0 || z > self.z1 {
            return HitType::None;
        }
        let mut normal = Vec3::x_hat();
        if dot(ray.direction, normal) > 0.0 {
            normal = -normal;
        }
        return HitType::Hit(HitRecord::new(t, ray, normal));
    }
    fn bounding_volume(&self) -> BoundingBoxStruct {
        return BoundingBoxStruct::new(
            Vec3::new(self.y0, self.z0, self.k-0.0001),
            Vec3::new(self.y1, self.z1, self.k+0.0001),
        )
    }
}


#[derive(Debug, Clone)]
pub struct CuboidStruct<'a> {
    sides: Vec<Hittable<'a>>,
    v_max: Vec3,
    v_min: Vec3,
    material: &'a Material,
}

impl <'a>CuboidStruct<'_> {
    pub fn new(p0: Vec3, p1: Vec3, material: &'a Material) -> CuboidStruct<'a> {

        let mut v_min = Vec3::zero();
        let mut v_max = Vec3::zero();
        v_min[0] = f64::min(p0.x(), p1.x());
        v_min[1] = f64::min(p0.y(), p1.y());
        v_min[2] = f64::min(p0.z(), p1.z());
        v_max[0] = f64::max(p0.x(), p1.x());
        v_max[1] = f64::max(p0.y(), p1.y());
        v_max[2] = f64::max(p0.z(), p1.z());

        let mut sides = Vec::new();
        sides.push(Hittable::XYRectangle(XYRectangleStruct::new(v_min.x(), v_max.x(), v_min.y(), v_max.y(), v_min.z(), material)));
        sides.push(Hittable::XYRectangle(XYRectangleStruct::new(v_min.x(), v_max.x(), v_min.y(), v_max.y(), v_max.z(), material)));
        sides.push(Hittable::XZRectangle(XZRectangleStruct::new(v_min.x(), v_max.x(), v_min.z(), v_max.z(), v_min.y(), material)));
        sides.push(Hittable::XZRectangle(XZRectangleStruct::new(v_min.x(), v_max.x(), v_min.z(), v_max.z(), v_max.y(), material)));
        sides.push(Hittable::YZRectangle(YZRectangleStruct::new(v_min.y(), v_max.y(), v_min.z(), v_max.z(), v_min.x(), material)));
        sides.push(Hittable::YZRectangle(YZRectangleStruct::new(v_min.y(), v_max.y(), v_min.z(), v_max.z(), v_max.x(), material)));
        return CuboidStruct{sides, v_min, v_max, material};
    }
    fn hit(&'a self, ray: &'a Ray, range: [f64;2]) -> HitType {
        let mut closest_hit_record = HitType::None;
        for hittable in &self.sides {
            let hit_record = hittable.hit(ray, range);
            match hit_record {
                HitType::Hit(h) => {
                    match closest_hit_record {
                        HitType::Hit(ref c) => {
                            if h.t_hit < c.t_hit {
                                closest_hit_record = HitType::Hit(h) 
                            }
                        }
                        _ => closest_hit_record = HitType::Hit(h),
                    }
                }
                _ => continue,
            }
        }
        return closest_hit_record;
    }
    fn bounding_volume(&self) -> BoundingBoxStruct {
        return BoundingBoxStruct::new(self.v_min, self.v_max);
    }
}

