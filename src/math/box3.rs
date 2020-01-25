
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

        let pmins = (self.mins - p).component_mul(&rs);
        let pmaxs = (self.maxs - p).component_mul(&rs);

        let tmins = pmins.zip_map(&pmaxs, |a, b| a.min(b));
        let tmaxs = pmins.zip_map(&pmaxs, |a, b| a.max(b));

        let tmin = tmins.max();
        let tmax = tmaxs.min();

        // TODO check correctness
        let normal = match tmin {
            t if t == pmins.x => -V3::x(),
            t if t == pmaxs.x =>  V3::x(),
            t if t == pmins.y => -V3::y(),
            t if t == pmaxs.y =>  V3::y(),
            t if t == pmins.z => -V3::z(),
            t if t == pmaxs.z =>  V3::z(),
            _ => unreachable!()
        };

        // TODO deal with ntersections when source is inside box
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
            .map(|ixn| assert_eq!(ixn.normal, V3::new(-1., 0., 0.)))
            .unwrap();

        // through the middle
        b.intersect(&Segment::new(P3::new(0., 0., 2.), V3::new(0., 0., -3.)))
            .map(|ixn| assert_eq!(ixn.normal, V3::new(0., 0., 1.)))
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
            .map(|ixn| assert_eq!(ixn.normal, V3::new(-1., 0., 0.)))
            .unwrap();
    }

    #[test]
    fn dilate() {
        // TODO
    }
}
