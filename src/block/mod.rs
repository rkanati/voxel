
use {
    crate::array3d,
};

pub type Slice   <'a> = array3d::ArraySlice   <'a, Block>;
pub type SliceMut<'a> = array3d::ArraySliceMut<'a, Block>;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Block {
    Empty,
    Stone,
    Soil,
    Grass,
    TreeTrunk,
}

impl Block {
    pub fn is_empty(&self) -> bool {
        *self == Block::Empty
    }

    pub fn is_nonempty(&self) -> bool {
        !self.is_empty()
    }
}

