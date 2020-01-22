
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
    fn make(&self, coords: Coords) -> Chunk;
}

pub struct Source<S, M> {
    cache: Cache<Coords, Chunk>,
    store: S,
    maker: M,
}

impl<S, M> Source<S, M> where S: ChunkStore, M: ChunkMaker {
    pub fn new(store: S, maker: M) -> Self {
        Source {
            cache: Cache::new(),
            store,
            maker
        }
    }

    pub fn load(&mut self, coords: Coords) -> Chunk {
        if let Some(chunk) = self.cache.acquire(&coords) {
            return chunk;
        }

        let chunk = self.store.load(coords)
            .unwrap_or_else(|| self.maker.make(coords));

        //self.cache.insert(coords, chunk);
        //self.cache.acquire(coords)
        self.cache.insert_and_acquire(coords);
        chunk
    }

    pub fn store(&mut self, coords: Coords, chunk: Chunk) {
        self.cache.release(coords, chunk);
        //let chunk = self.cache.remove(coords);
        //self.store.store(coords, chunk);
    }

    pub fn sync(&mut self) {
        unimplemented!();
        //for (coords, chunk) in self.cache.iter_all().unwrap() {
        //    if chunk.touched() { // TODO think
        //        self.store.store(*coords, chunk)
        //    }
        //}
    }

    pub fn flush(&mut self) {
        unimplemented!();
    }
}

