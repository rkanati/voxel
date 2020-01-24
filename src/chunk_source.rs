
use {
    crate::{
        chunk::{Chunk, Coords},
        chunk_cache::*,
    },
};

pub trait ChunkStore {
    fn load(&mut self, coords: Coords) -> Option<Chunk>;
    fn store(&mut self, coords: Coords, chunk: &Chunk);
}

pub trait ChunkMaker {
//  fn make(&self, coords: Coords) -> Chunk;
    fn make(&self, coords: Coords) -> Vec<(Coords, Chunk)>;
}

pub struct Source<S, M> {
    cache: Cache<Coords, Chunk>,
    store: S,
    maker: M,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LoadedFrom {
    Cache,
    Store,
    Maker
}

impl<S, M> Source<S, M> where S: ChunkStore, M: ChunkMaker {
    pub fn new(store: S, maker: M) -> Self {
        Source {
            cache: Cache::new(),
            store,
            maker
        }
    }

    pub fn load(&mut self, coords: Coords) -> (Chunk, LoadedFrom) {
        if let Some(chunk) = self.cache.acquire(&coords) {
            return (chunk, LoadedFrom::Cache);
        }

        if let Some(chunk) = self.store.load(coords) {
            return (chunk, LoadedFrom::Store);
        }

        self.maker.make(coords)
            .drain(..)
            .for_each(|(coords, chunk)| {
                self.cache.insert(coords, chunk);
            });

        let chunk = self.cache.acquire(&coords).unwrap();
        (chunk, LoadedFrom::Maker)
    }

    pub fn store(&mut self, coords: Coords, chunk: Chunk) {
        self.cache.release(coords, chunk);
        //let chunk = self.cache.remove(coords);
        //self.store.store(coords, chunk);
    }

    pub fn sync(&mut self) {
        todo!();
        //for (coords, chunk) in self.cache.iter_all().unwrap() {
        //    if chunk.touched() { // TODO think
        //        self.store.store(*coords, chunk)
        //    }
        //}
    }

    pub fn flush(&mut self) {
        todo!();
    }
}

