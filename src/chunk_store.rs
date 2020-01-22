
use {
    crate::{
        block::Block,
        chunk::{self, Array, Chunk, Coords},
        chunk_source::ChunkStore,
    },
};

pub struct Null {
}

impl Null {
    pub fn new() -> Null {
        Null { }
    }
}

impl ChunkStore for Null {
    fn load(&mut self, _: Coords) -> Option<Chunk> {
        None
    }

    fn store(&mut self, _: Coords, _: &Chunk) {
    }
}

