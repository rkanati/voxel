
use super::*;

#[derive(Clone, Copy)]
pub struct Box3 {
    mins: P3,
    maxs: P3,
}

impl Box3 {
    pub fn new(a: P3, b: P3) -> Box3 {
        let mins = a.coords.zip_map(&b.coords, |a, b| a.min(b));
        let maxs = a.coords.zip_map(&b.coords, |a, b| a.max(b));
        Self::new_unchecked(mins.into(), maxs.into())
    }

    pub fn new_unchecked(mins: P3, maxs: P3) -> Box3 {
        Box3 { mins, maxs }
    }

    pub fn with_dims(corner: P3, dims: V3) -> Box3 {
        Self::new(corner, corner + dims)
    }

    pub fn with_dims_unchecked(mins: P3, dims: V3) -> Box3 {
        Self::new_unchecked(mins, mins + dims)
    }

    #[must_use]
    pub fn dilate(&self, with: &Box3) -> Box3 {
        Self::new_unchecked(self.mins - with.maxs.coords, self.maxs - with.mins.coords)
    }

    #[must_use]
    pub fn at(&self, point: P3) -> Box3 {
        Self::new_unchecked(self.mins + point.coords, self.maxs + point.coords)
    }

    pub fn contains(&self, point: P3) -> bool {
        (self.mins.x ..= self.maxs.x).contains(&point.x) &&
        (self.mins.y ..= self.maxs.y).contains(&point.y) &&
        (self.mins.z ..= self.maxs.z).contains(&point.z)
    }
}

#[derive(Clone, Copy)]
pub struct Intersection {
    pub lambda: f32,
    pub point:  P3,
    pub normal: V3,
}

impl<L> Intersect<L> for Box3 where L: Linear {
    type Intersection = Intersection;

    fn intersect(&self, other: &L) -> Option<Intersection> {
        let p = other.source();
        let rs = other.stride().map(|x| 1. / x);

        let tx1 = (self.mins.x - p.x) * rs.x;
        let tx2 = (self.maxs.x - p.x) * rs.x;
        let ty1 = (self.mins.y - p.y) * rs.y;
        let ty2 = (self.maxs.y - p.y) * rs.y;
        let tz1 = (self.mins.z - p.z) * rs.z;
        let tz2 = (self.maxs.z - p.z) * rs.z;

        // sigh
        let tmin = tx1.min(tx2).max(ty1.min(ty2)).max(tz1.min(tz2));
        let tmax = tx1.max(tx2).min(ty1.max(ty2)).min(tz1.max(tz2));

        // TODO check correctness
        let normal = match tmin {
            t if t == tx1 => -V3::x(),
            t if t == tx2 =>  V3::x(),
            t if t == ty1 => -V3::y(),
            t if t == ty2 =>  V3::y(),
            t if t == tz1 => -V3::z(),
            t if t == tz2 =>  V3::z(),
            _ => unreachable!()
        };

        // TODO deal with intersections when source is inside box
        if tmax <= tmin || !other.parameter_on(tmin) {
            return None;
        }

        let lambda = tmin;
        let point = other.at(tmin);

        Some(Intersection { lambda, point, normal })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersect() {
        let b = Box3::with_dims(P3::new(-1., -1., -1.), V3::repeat(2.));

        // through the middle
        b.intersect(&Segment::new(P3::new(-2., 0., 0.), V3::new(2., 0., 0.)))
            .unwrap();

        // through an edge
        b.intersect(&Segment::new(P3::new(-2., -2., 0.), V3::new(2., 2., 0.)))
            .unwrap();

        // through a corner
        b.intersect(&Segment::new(P3::new(-2., -2., -2.), V3::new(2., 2., 2.)))
            .unwrap();

        // stops short
        b.intersect(&Segment::new(P3::new(-2., 0., 0.), V3::new(0.5, 0., 0.)))
            .map(|_| assert!(false));

        // points away
        b.intersect(&Segment::new(P3::new(-2., 0., 0.), V3::new(-3., 0., 0.)))
            .map(|_| assert!(false));

        // straight past
        b.intersect(&Segment::new(P3::new(-2., -2., 0.), V3::new(5., 0., 0.)))
            .map(|_| assert!(false));

        // diagonally past an edge
        b.intersect(&Segment::new(P3::new(-2.1, 0., 0.), V3::new(2., 2., 0.)))
            .map(|_| assert!(false));

        // grazing an edge
        b.intersect(&Segment::new(P3::new(-1.8, 0., 0.), V3::new(2., 2., 0.)))
            .unwrap();
    }

    #[test]
    fn dilate() {
        // TODO
    }
}
