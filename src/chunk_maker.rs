
use {
    crate::{
        block::Block,
        chunk::{self, Array, Chunk, Coords},
        chunk_source::ChunkMaker,
    },
};

pub struct Test {
}

impl Test {
    pub fn new() -> Test {
        Test { }
    }
}

impl ChunkMaker for Test {
    fn make(&self, coords: Coords) -> Chunk {
        Array::generate(|rel| {
            let p = coords.coords * chunk::DIM + rel.map(|x| x as i32);
            //let solid = p.map(|x| x % 2 == 0).iter().all(|b| *b);
            //let solid = (p.x + p.y + p.z) % 12 == 0;
            let solid = p.z <= 0 && p.map(|x| x as f32).norm().trunc() as i32 % 8 == 0;
            if solid { Block::Solid } else { Block::Empty }
        }).into()
    }
}

