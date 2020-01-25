
pub mod iter;
pub use iter::*;

pub mod linear;
pub use linear::*;

pub mod planar;
pub use planar::Plane;

pub mod box3;
pub use box3::Box3;

pub use nalgebra as na;

pub trait Intersect<With> {
    type Intersection;
    fn intersect(&self, other: &With) -> Option<Self::Intersection>;
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct OrdFloat<T> (pub T) where T: PartialOrd + PartialEq;

impl<T> Eq for OrdFloat<T> where T: PartialOrd + PartialEq { }

impl<T> OrdFloat<T> where T: PartialOrd + PartialEq {
    fn unwrap(self) -> T { self.0 }
}

impl<T> std::cmp::Ord for OrdFloat<T> where T: PartialOrd + PartialEq {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&rhs.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

pub type V2<T = f32> = na::Vector2<T>;
pub type V3<T = f32> = na::Vector3<T>;
pub type V4<T = f32> = na::Vector4<T>;

pub type P3<T = f32> = na::Point3<T>;

pub type V2u8 = V2<u8>;
pub type V3u8 = V3<u8>;
pub type V4u8 = V4<u8>;

pub type V3i8 = V3<i8>;

pub type P3u8 = P3<u8>;

pub type V2i32 = V2<i32>;
pub type V3i32 = V3<i32>;

pub type P3i32 = P3<i32>;

pub type V3usize = V3<usize>;
pub type P3usize = P3<usize>;

pub type M4<T = f32> = na::Matrix4<T>;
pub type Motion<T = f32> = na::Isometry3<T>;
pub type Perspective<T = f32> = na::Perspective3<T>;
pub type Translation<T = f32> = na::Translation3<T>;

pub type Quaternion<T = f32> = na::Quaternion<T>;
pub type Versor<T = f32> = na::UnitQuaternion<T>;
pub type Complex<T = f32> = na::Complex<T>;

pub const PI: f32 = std::f32::consts::PI;

