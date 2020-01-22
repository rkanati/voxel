
use super::{V3, na};

pub trait Indexish : na::Scalar {
    fn increment(&mut self);
}

impl Indexish for u8 {
    fn increment(&mut self) { *self += 1; }
}

impl Indexish for i32 {
    fn increment(&mut self) { *self += 1; }
}

impl Indexish for usize {
    fn increment(&mut self) { *self += 1; }
}

pub struct SpaceIter<T> where T: Indexish {
    orig: V3<T>,
    ends: V3<T>,
    cur:  V3<T>,
}

impl<T> SpaceIter<T> where T: Indexish {
    pub fn new (orig: V3<T>, ends: V3<T>) -> SpaceIter<T>  {
        SpaceIter { orig, ends, cur: orig }
    }
}

impl<T> Iterator for SpaceIter<T> where T: Indexish {
    type Item = V3<T>;

    fn next(&mut self) -> Option<V3<T>> {
        if self.cur.x == self.ends.x {
            return None;
        }

        let value = self.cur;
        self.cur.z.increment();

        if self.cur.z == self.ends.z {
            self.cur.z = self.orig.z;
            self.cur.y.increment();
        }

        if self.cur.y == self.ends.y {
            self.cur.y = self.orig.y;
            self.cur.x.increment();
        }

        Some(value)
    }
}

