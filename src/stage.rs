
use {
    std::mem,
    crate::{
        array3d,
        math::*,
    },
};

struct Element<T> {
    coords:  P3i32,
    content: Option<T>
}

pub struct Stage<T> {
//  dim:      i32,
//  elements: Box<[Element<T>]>,
//  centre:   P3i32,
    array:  array3d::ArrayOwned<Element<T>, array3d::DynamicDims>,
    center: P3i32,
}

//type Slice   <'a, T> = array3d::ArraySlice   <'a, Element<T>>;
//type SliceMut<'a, T> = array3d::ArraySliceMut<'a, Element<T>>;

impl<T> Stage<T> {
    pub fn dims(&self) -> V3usize {
        self.array.dims()
    }

    pub fn new(radius: i32, center: P3i32) -> Stage<T> {
        let offset = V3::repeat(radius);
        let dims = V3::repeat(radius as usize * 2);
        let array = array3d::Array::generate_with_dims(
            dims,
            |ijk| Element {
                coords: center - offset + ijk.map(|x| x as i32),
                content: None
            }
        );

        Stage { array, center }
    }

    pub fn relative_mins(&self) -> V3i32 {
        self.dims()
            .map(|x| 1 - (x as i32 / 2))
    }

    pub fn relative_maxs(&self) -> V3i32 {
        self.dims()
            .map(|x| 1 + (x as i32 / 2))
    }

    pub fn relative_to_absolute(&self, rel: V3i32) -> P3i32 {
        self.center + rel
    }

    pub fn absolute_to_relative(&self, abs: P3i32) -> V3i32 {
        abs - self.center
    }

    pub fn relative_coords_iter(&self) -> SpaceIter<i32> {
        SpaceIter::new(self.relative_mins(), self.relative_maxs())
    }

    pub fn absolute_coords_iter(&self) -> impl Iterator<Item = P3i32> {
        let center = self.center;
        self.relative_coords_iter()
            .map(move |rel| center + rel)
    }

    fn rel_to_ijk(&self, rel: V3i32) -> Option<V3usize> {
        let mins = self.relative_mins();
        if rel.x < mins.x || rel.y < mins.y || rel.z < mins.z { return None; }
        let maxs = self.relative_maxs();
        if rel.x >= maxs.x || rel.y >= maxs.y || rel.z >= maxs.z { return None; }
        let abs = self.center.coords + rel;
        let ijk = abs.zip_map(
            &self.dims(),
            |x, dim| x.rem_euclid(dim as i32) as usize
        );
        Some(ijk)
    }

    fn abs_to_ijk(&self, abs: P3i32) -> Option<V3usize> {
        self.rel_to_ijk(abs - self.center)
    }

    fn elem_abs(&self, abs: P3i32) -> Option<&Element<T>> {
        let elem = self.array.get(self.abs_to_ijk(abs)?);
        if elem.coords == abs { Some(elem) }
        else                  { None }
    }

    fn elem_abs_mut(&mut self, abs: P3i32) -> Option<&mut Element<T>> {
        let elem = self.array.get_mut(self.abs_to_ijk(abs)?);
        if elem.coords == abs { Some(elem) }
        else                  { None }
    }

    pub fn at_absolute(&self, abs: P3i32) -> Option<&T> {
        self.elem_abs(abs)
            .and_then(|elem| elem.content.as_ref())
    }

    pub fn at_absolute_mut(&mut self, abs: P3i32) -> Option<&mut T> {
        self.elem_abs_mut(abs)
            .and_then(|elem| elem.content.as_mut())
    }

    pub fn at_relative(&self, rel: V3i32) -> Option<&T> {
        self.at_absolute(self.center + rel)
    }

    pub fn at_relative_mut(&mut self, rel: V3i32) -> Option<&mut T> {
        self.at_absolute_mut(self.center + rel)
    }

    pub fn insert_absolute(&mut self, abs: P3i32, content: T) -> Option<T> {
        let element = self.array.get_mut(self.abs_to_ijk(abs)?);
        let new_element = Element { coords: abs, content: Some(content) };
        mem::replace(element, new_element).content
    }

    pub fn insert_relative(&mut self, rel: V3i32, content: T) -> Option<T> {
        self.insert_absolute(self.center + rel, content)
    }

    pub fn relocate(&mut self, new_center: P3i32) -> Vec<StaleChunk<T>> {
        self.center = new_center;

        let mut stale = Vec::new();
        for abs in self.absolute_coords_iter() {
            let elem = self.array.get_mut(self.abs_to_ijk(abs).unwrap());
            if elem.content.is_none() {
                stale.push(StaleChunk::Missing(abs));
            }
            else if elem.coords != abs {
                let value = elem.content.take().unwrap();
                let evicted = StaleChunk::Evicted {
                    old_coords: elem.coords,
                    new_coords: abs,
                    value
                };
                stale.push(evicted);
            }
        }

        stale
    }
}

pub enum StaleChunk<T> {
    Missing(P3i32),
    Evicted {
        old_coords: P3i32,
        new_coords: P3i32,
        value:      T
    },
}

impl<T> std::ops::Index<V3i32> for Stage<T> {
    type Output = T;
    fn index(&self, ijk: V3i32) -> &T {
        self.at_relative(ijk)
            .unwrap()
    }
}

impl<T> std::ops::IndexMut<V3i32> for Stage<T> {
    fn index_mut(&mut self, ijk: V3i32) -> &mut T {
        self.at_relative_mut(ijk)
            .unwrap()
    }
}

