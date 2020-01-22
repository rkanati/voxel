
use crate::{
    array3d,
    block::Block,
    math::P3i32,
};

pub type Coords = P3i32;

pub const DIM_LOG2: i32 = 4;                //   3     4     5     6
pub const DIM:      i32 = 1 << DIM_LOG2;    //   8    16    32    64
pub const DIM_MASK: i32 = DIM - 1;          //   7     f    1f    3f
pub const VOLUME:   i32 = DIM * DIM * DIM;  // 512    4k   32k  256k  (Blocks, not bytes)

#[derive(Default, Clone)]
pub struct Dims;

impl array3d::StaticDims for Dims {
    const X: usize = DIM as usize;
    const Y: usize = DIM as usize;
    const Z: usize = DIM as usize;
}

pub type Array     = array3d::ArrayOwned<Block, Dims>;
pub type Slice<'a> = array3d::ArraySlice<'a, Block>;

#[derive(Clone)]
pub struct Chunk {
    blocks: Array,
}

impl From<Array> for Chunk {
    fn from(array: Array) -> Chunk {
        Chunk::new(array)
    }
}

impl Chunk {
    pub fn new(blocks: Array) -> Chunk {
        Chunk { blocks }
    }

    pub fn is_empty(&self) -> bool {
        self.iter()
            .all(|block| block.is_empty())
    }
}

impl std::ops::Deref for Chunk {
    type Target = Array;
    fn deref(&self) -> &Array {
        &self.blocks
    }
}

impl std::ops::DerefMut for Chunk {
    fn deref_mut(&mut self) -> &mut Array {
        &mut self.blocks
    }
}

impl std::iter::FromIterator<Block> for Chunk {
    fn from_iter<T> (iter: T) -> Chunk where T: IntoIterator<Item = Block> {
        Chunk::new(Array::from_iter(iter))
    }
}

