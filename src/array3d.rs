
use {
    std::ops::{Index, IndexMut},
    crate::math::{V3, V3usize, SpaceIter},
};

pub trait Dims : Clone + 'static {
    fn dims(&self) -> V3usize;

    fn volume(&self) -> usize {
        self.dims().iter().product()
    }

    fn flat_index(&self, ijk: V3usize) -> usize {
        let dims = self.dims();
        let strides = V3usize::new(dims.x * dims.y, dims.y, 1);
        ijk.dot(&strides)
    }

    fn as_dynamic(&self) -> DynamicDims {
        DynamicDims(self.dims())
    }
}

pub trait OwnHold<T> {
    type OwnType: HoldMut<Element = T>;
}

#[derive(Clone, Copy)]
pub struct DynamicDims(V3usize);

impl Dims for DynamicDims {
    fn dims(&self) -> V3usize {
        self.0
    }
}

impl<T> OwnHold<T> for DynamicDims {
    type OwnType = Vec<T>;
}


#[derive(Clone)]
pub struct SliceDims {
    whole:  DynamicDims,
    offset: V3usize,
    dims:   V3usize,
}

impl Dims for SliceDims {
    fn dims(&self) -> V3usize {
        self.dims
    }

    fn flat_index(&self, ijk: V3usize) -> usize {
        // TODO check ijk against self.dims
        self.whole.flat_index(self.offset + ijk)
    }
}


pub trait StaticDims: Default + Clone + 'static {
    const X: usize;
    const Y: usize;
    const Z: usize;
    const N: usize = Self::X * Self::Y * Self::Z;
}

impl<D> Dims for D where D: StaticDims {
    fn dims(&self) -> V3usize {
        V3usize::new(D::X, D::Y, D::Z)
    }
}

impl<T> Hold for Vec<T> {
    type Element = T;
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T> HoldMut for Vec<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<D, T> OwnHold<T> for D where D: StaticDims {
    //type OwnType = [T; D::N]; // TODO need const generics
    type OwnType = Vec<T>;
}

pub trait Hold {
    type Element;
    fn as_ref(&self) -> &[Self::Element];
}

pub trait HoldMut : Hold {
    fn as_mut(&mut self) -> &mut [Self::Element];
}

impl<'a, T> Hold for &'a [T] {
    type Element = T;
    fn as_ref(&self) -> &[T] { self }
}



impl<'a, T> Hold for &'a mut [T] {
    type Element = T;
    fn as_ref(&self) -> &[T] { self }
}

impl<'a, T> HoldMut for &'a mut [T] {
    fn as_mut(&mut self) -> &mut [T] { self }
}





pub struct Array<D, H>
      where D: Dims,
{
    hold: H,
    dims: D,
}

// TODO this is incoherent until we have specialization:
//      https://github.com/rust-lang/rust/issues/31844
//impl<Hi, Ho, Di, Do> From<Array<Di, Hi>> for Array<Do, Ho>
//    where Do: From<Di> + Dims,
//          Di: Dims,
//          Ho: From<Hi> + Hold,
//          Hi: Hold,
//{
//    fn from(array: Array<Di, Hi>) -> Array<Do, Ho> {
//        Array {
//            hold: array.hold.into(),
//            dims: array.dims.into(),
//        }
//    }
//}
//

impl<D, H> std::fmt::Debug for Array<D, H>
    where D: Dims,
          H: Hold,
          H::Element: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "<array3d {:?}>", self.hold.as_ref())
    }
}

impl<D, H> Clone for Array<D, H> where D: Dims + Clone, H: Clone {
    fn clone(&self) -> Self {
        Array {
            hold: self.hold.clone(),
            dims: self.dims.clone()
        }
    }
}

impl<Dl, Dr, Hl, Hr> PartialEq<Array<Dr, Hr>> for Array<Dl, Hl>
    where Dl: Dims, Dr: Dims,
          Hl: Hold, Hr: Hold,
          Hl::Element: PartialEq<Hr::Element>,
{
    fn eq(&self, rhs: &Array<Dr, Hr>) -> bool {
           self.dims()        == rhs.dims()
        && self.hold.as_ref() == rhs.hold.as_ref()
    }
}



//pub type ArrayOwned<T, D: OwnHold<T>> = Array<D, D::OwnType>;
pub type ArrayOwned<T, D> = Array<D, <D as OwnHold<T>>::OwnType>;

pub type ArraySlice   <'a, T> = Array<SliceDims, &'a     [T]>;
pub type ArraySliceMut<'a, T> = Array<SliceDims, &'a mut [T]>;




//impl<D, T> Array<D, D::OwnType> where D: Dims + OwnHold<T> {
impl<D, H> Array<D, H> where D: Dims, H: Hold {
    pub fn get<'a, 'b> (&'a self, ijk: V3usize) -> &'b H::Element where 'a: 'b {
        &self.hold.as_ref()[self.dims.flat_index(ijk)]
    }
}

impl<D, H> Index<V3usize> for Array<D, H> where D: Dims, H: Hold {
    type Output = H::Element;
    fn index(&self, ijk: V3usize) -> &H::Element {
        self.get(ijk)
    }
}




impl<D, H> Array<D, H> where D: Dims, H: HoldMut {
    pub fn get_mut<'b, 'a: 'b> (&'a mut self, ijk: V3usize) -> &'b mut H::Element {
        &mut self.hold.as_mut()[self.dims.flat_index(ijk)]
    }
}

impl<D, H> IndexMut<V3usize> for Array<D, H> where D: Dims, H: HoldMut {
    fn index_mut(&mut self, ijk: V3usize) -> &mut H::Element {
        self.get_mut(ijk)
    }
}




impl<D, H> Array<D, H> where D: Dims, H: Hold {
    pub fn slice(&self, offset: V3usize, dims: V3usize)
        -> ArraySlice<'_, H::Element>
    {
        // TODO cloning dims is wrong for slice-of-a-slice? maybe?
        let dims = SliceDims { whole: self.dims.as_dynamic(), dims, offset };
        ArraySlice { dims, hold: self.hold.as_ref() }
    }

    pub fn whole_slice(&self) -> ArraySlice<'_, H::Element> {
        self.slice(V3::zeros(), self.dims())
    }
}

impl<D, H> Array<D, H> where D: Dims, H: HoldMut {
    pub fn slice_mut(&mut self, offset: V3usize, dims: V3usize)
        -> ArraySliceMut<'_, H::Element>
    {
        let dims = SliceDims { whole: self.dims.as_dynamic(), dims, offset };
        ArraySliceMut { dims, hold: self.hold.as_mut() }
    }
}

impl<D, H> Array<D, H> where D: Dims {
    pub fn dims(&self) -> V3usize {
        self.dims.dims()
    }

    pub fn volume(&self) -> usize {
        self.dims.volume()
    }

    pub fn indices(&self) -> SpaceIter<usize> {
        SpaceIter::new(V3::zeros(), self.dims())
    }
}

pub struct IndexedIterMut<'a, T, D> {
    indices: SpaceIter<usize>,
    cursor:  usize,
    dims:    D,
    slice:   &'a mut [T],
}

impl<'a, T, D> Iterator for IndexedIterMut<'a, T, D> where D: Dims {
    type Item = (V3usize, &'a mut T);
    fn next(&mut self) -> Option<Self::Item> {
        let ijk = self.indices.next()?;
        let index = self.dims.flat_index(ijk);

        let slice = std::mem::take(&mut self.slice);
        let (_, tail) = slice.split_at_mut(index - self.cursor);
        let (elem, rest) = tail.split_first_mut()?;

        self.slice = rest;
        self.cursor = index + 1;
        Some((ijk, elem))
    }
}

impl<'a, T, D> IndexedIterMut<'a, T, D> where D: Dims {
    fn new(indices: SpaceIter<usize>, dims: D, slice: &'a mut [T]) -> Self {
        Self { indices, cursor: 0, dims, slice }
    }
}



impl<D, H> Array<D, H> where D: Dims, H: Hold {
    pub fn indexed_iter(&self) -> impl Iterator<Item = (V3usize, &H::Element)> {
        self.indices()
            .map(move |ijk| (ijk, &self[ijk]))
    }

    pub fn iter(&self) -> impl Iterator<Item = &H::Element> {
        self.indexed_iter()
            .map(|(_, elem)| elem)
    }
}

impl<D, H> Array<D, H> where D: Dims, H: HoldMut {
    pub fn indexed_iter_mut<'b, 'a: 'b> (&'a mut self)
        -> IndexedIterMut<'a, H::Element, D>
    {
        IndexedIterMut::new(self.indices(), self.dims.clone(), self.hold.as_mut())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut H::Element> {
        self.indexed_iter_mut()
            .map(|(_, e)| e)
    }
}


impl<D, H> Array<D, H>
    where D:          Dims,
          H:          HoldMut,
          H::Element: Copy
{
    pub fn copy_from<Ds, Hs> (&mut self, src: &Array<Ds, Hs>)
        where Ds: Dims,
              Hs: Hold<Element = H::Element>,
    {
        assert!(self.dims() == src.dims());
        for (dst, src) in self.iter_mut().zip(src.iter().copied()) {
            *dst = src;
        }
    }
}

impl<T> Array<DynamicDims, Vec<T>> {
    pub fn generate_with_dims(dims: V3usize, func: impl FnMut(V3usize) -> T) -> Self {
        let hold: Vec<T> = SpaceIter::new(V3usize::zeros(), dims)
            .map(func)
            .collect();
        Array { hold, dims: DynamicDims(dims) }
    }

    pub fn new_with_dims(dims: V3usize, elements: Vec<T>) -> Self {
        let dims = DynamicDims(dims);
        assert!(elements.len() == dims.volume());
        Array { hold: elements, dims }
    }
}

impl<D, T> Array<D, Vec<T>> where D: StaticDims {
    pub fn generate(func: impl FnMut(V3usize) -> T) -> Self {
        let dims = D::default();
        let hold: Vec<T> = SpaceIter::new(V3usize::zeros(), dims.dims())
            .map(func)
            .collect();
        Array { hold, dims }
    }

    pub fn new(elements: Vec<T>) -> Self {
        let dims = D::default();
        assert!(elements.len() == dims.volume());
        Array { hold: elements, dims }
    }
}

impl<D, T> Array<D, Vec<T>> where D: StaticDims, T: Copy {
    pub fn new_filled(with: T) -> Self {
        let dims = D::default();
        let elements: Vec<T> = std::iter::repeat(with)
            .take(dims.volume())
            .collect();
        Array::new(elements)
    }
}

impl<D, T> Default for Array<D, Vec<T>> where D: StaticDims, T: Copy + Default {
    fn default() -> Self {
        Array::new_filled(T::default())
    }
}

impl<D, T> std::iter::FromIterator<T> for Array<D, Vec<T>> where D: StaticDims {
    fn from_iter<I> (iter: I) -> Self where I: IntoIterator<Item = T> {
        Array::new(Vec::from_iter(iter))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Default)]
    struct Dims;

    impl StaticDims for Dims {
        const X: usize = 3;
        const Y: usize = 3;
        const Z: usize = 3;
    }

    #[test]
    fn slice() {
        let array: ArrayOwned<_, Dims> = Array::generate(|ijk| ijk != V3::new(1, 1, 1));

        let x0 = array.slice(V3::new(0, 0, 0), V3::new(1, 3, 3));
        assert_eq!(x0.dims(), V3::new(1, 3, 3));
        assert_eq!(x0.volume(), 9);
        assert!(x0.iter().all(|b| *b));

        let z1 = array.slice(V3::new(0, 0, 1), V3::new(3, 3, 1));
        assert_eq!(z1.iter().filter(|b| **b).count(), 8);
    }

    #[test]
    fn copy_from() {
        type Array = ArrayOwned<i32, Dims>;

        let array = Array::new((0 .. 27).collect());
        let mut dest = Array::new_filled(99);

        dest.slice_mut(V3::zeros(), dest.dims())
            .copy_from(&array.slice(V3::zeros(), array.dims()));

        assert_eq!(array, dest);
    }
}

