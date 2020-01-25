
use crate::{
    array3d,
    block::Block,
    math::*,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coords(V3i32);

impl Coords {
    pub fn origin() -> Coords {
        Coords(V3::zeros())
    }

    pub fn new(point: P3i32) -> Coords {
        Coords(point.coords)
    }

    pub fn unwrap(self) -> V3i32 {
        self.0
    }

    pub fn block_mins(&self) -> BlockCoords {
        BlockCoords::new(self.0.map(|x| x << DIM_LOG2).into())
    }

    pub fn block_at_offset(&self, offset: V3u8) -> BlockCoords {
        self.block_mins() + offset.map(|x| x as i32)
    }

    pub fn containing(point: P3) -> Coords {
        BlockCoords::containing(point).chunk()
    }
}

impl std::ops::Add<V3i32> for Coords {
    type Output = Coords;
    fn add(self, rhs: V3i32) -> Coords {
        Coords(self.0 + rhs)
    }
}

impl std::ops::Sub<V3i32> for Coords {
    type Output = Coords;
    fn sub(self, rhs: V3i32) -> Coords {
        Coords(self.0 - rhs)
    }
}

impl std::ops::Sub<Coords> for Coords {
    type Output = V3i32;
    fn sub(self, rhs: Coords) -> V3i32 {
        self.0 - rhs.0
    }
}

pub const DIM_LOG2: i32 = 4;                //   3     4     5     6
pub const DIM:      i32 = 1 << DIM_LOG2;    //   8    16    32    64
pub const DIM_MASK: i32 = DIM - 1;          //   7     f    1f    3f
pub const VOLUME:   i32 = DIM * DIM * DIM;  // 512    4k   32k  256k  (Blocks, not bytes)

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockCoords(V3i32);

impl BlockCoords {
    pub fn new(point: P3i32) -> BlockCoords {
        BlockCoords(point.coords)
    }

    pub fn unwrap(self) -> V3i32 {
        self.0
    }

    pub fn unwrap_f32(self) -> V3 {
        self.0.map(|x| x as f32)
    }

    pub fn origin() -> BlockCoords {
        BlockCoords(V3::zeros())
    }

    pub fn containing(point: P3) -> BlockCoords {
        BlockCoords(point.coords.map(|x| x.floor() as i32))
    }

    pub fn chunk(&self) -> Coords {
        Coords::new(self.0.map(|x| x >> DIM_LOG2).into())
    }

    pub fn offset(&self) -> V3u8 {
        self.0.map(|x| (x & DIM_MASK) as u8)
    }

    pub fn chunk_and_offset(&self) -> (Coords, V3u8) {
        (self.chunk(), self.offset())
    }
}

impl std::ops::Add<V3i32> for BlockCoords {
    type Output = BlockCoords;
    fn add(self, rhs: V3i32) -> BlockCoords {
        BlockCoords(self.0 + rhs)
    }
}

impl std::ops::Sub<BlockCoords> for BlockCoords {
    type Output = V3i32;
    fn sub(self, rhs: BlockCoords) -> V3i32 {
        self.0 - rhs.0
    }
}




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

impl std::ops::Index<V3u8> for Chunk {
    type Output = Block;
    fn index(&self, ijk: V3u8) -> &Block {
        self.get(ijk.map(|x| x as usize))
    }
}

impl std::ops::IndexMut<V3u8> for Chunk {
    fn index_mut(&mut self, ijk: V3u8) -> &mut Block {
        self.get_mut(ijk.map(|x| x as usize))
    }
}

impl std::iter::FromIterator<Block> for Chunk {
    fn from_iter<T> (iter: T) -> Chunk where T: IntoIterator<Item = Block> {
        Chunk::new(Array::from_iter(iter))
    }
}

