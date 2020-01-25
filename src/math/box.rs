
use super::*;

pub struct Box3 {
    mins: P3,
    maxs: P3,
}

pub struct Intersection {
    pub lambda: f32,
    pub point:  P3,
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

        // sigh
        let tmin = tx1.min(tx2).min(ty1).min(ty2);
        let tmax = tx1.max(tx2).max(ty1).max(ty2);

        // TODO 
        if tmax <= tmin || !other.parameter_on(tmin) {
            return None;
        }

        let lambda = tmin;
        let point = other.at(tmin);

        Some(Intersection { lambda, point })
    }
}

