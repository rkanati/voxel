
use super::*;

#[derive(Clone, Copy, Debug)]
pub struct Plane(V3);

impl Plane {
    pub fn new(normal_to_centre: V3) -> Plane {
        Plane(normal_to_centre)
    }

    pub fn from_point_and_normal(point: P3, normal: V3) -> Plane {
        let distance = normal.dot(&point.coords);
        let normal_to_centre = normal * distance;
        Self::new(normal_to_centre)
    }
}

pub struct Intersection {
    pub lambda: f32,
    pub point:  P3,
}

impl<L> Intersect<L> for Plane where L: Linear {
    type Intersection = Intersection;
    fn intersect(&self, other: &L) -> Option<Intersection> {
        let line = other.whole_line();
        let normal = self.0;
        let denom = line.stride().dot(&normal);
        if denom.abs() < 0.000001 {
            return None;
        }

        let lambda = (P3::from(normal) - line.source()).dot(&normal);
        if !other.parameter_on(lambda) {
            return None;
        }

        let point = line.at(lambda);
        Some(Intersection { lambda, point })
    }
}

