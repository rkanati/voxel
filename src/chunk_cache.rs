
use {
    std::{
        borrow::Borrow,
        collections::HashMap,
        hash::Hash,
    },
};

enum Entry<V> {
    Cached(V),
    Borrowed
}

impl<V> Entry<V> {
    fn acquire(&mut self) -> V {
        match std::mem::replace(self, Entry::Borrowed) {
            Entry::Borrowed      => { panic!(); }
            Entry::Cached(value) => { value }
        }
    }

    fn release(&mut self, value: V) {
        if let Entry::Cached(_) = std::mem::replace(self, Entry::Cached(value)) {
            panic!();
        }
    }

    fn unwrap(self) -> V {
        match self {
            Entry::Borrowed      => { panic!(); }
            Entry::Cached(value) => { value }
        }
    }
}

pub struct Cache<K, V> {
    map: HashMap<K, Entry<V>>,
    borrows: usize,
}

impl<K, V> Cache<K, V> where K: Hash + Eq {
    pub fn new() -> Cache<K, V> {
        Cache {
            map: HashMap::new(),
            borrows: 0,
        }
    }

    pub fn acquire<Q> (&mut self, key: &Q) -> Option<V>
        where K: Borrow<Q>,
              Q: Hash + Eq
    {
        let value = self.map.get_mut(&key)?.acquire();
        self.borrows += 1;
        Some(value)
    }

    pub fn release(&mut self, key: K, value: V) {
        self.map.get_mut(&key).unwrap().release(value);
        self.borrows -= 1;
    }

    pub fn insert(&mut self, key: K, value: V) {
        let prev = self.map.insert(key, Entry::Cached(value));
        assert!(prev.is_none());
    }

    pub fn insert_and_acquire(&mut self, key: K) {
        let prev = self.map.insert(key, Entry::Borrowed);
        assert!(prev.is_none());
        self.borrows += 1;
    }

    pub fn remove<Q> (&mut self, key: &Q) -> V
        where K: Borrow<Q>,
              Q: Hash + Eq
    {
        self.map
            .remove(&key)
            .unwrap()
            .unwrap()
    }

    pub fn borrowed(&self) -> bool {
        self.borrows != 0
    }

    //pub fn iter(&self) -> impl Iterator<Item = (&K, Option<&V>)> {
    //    self.map.iter()
    //        .map(|(k, v)| (k, v.as_ref()))
    //}

    //pub fn iter_unborrowed(&self) -> impl Iterator<Item = (&K, &V)> {
    //    self.iter()
    //        .filter_map(|(k, v)| v.map(|v| (k, v)))
    //}

    //pub fn iter_all(&self) -> Option<impl Iterator<Item = (&K, &V)>> {
    //    if self.borrowed() {
    //        None
    //    }
    //    else {
    //        let iter = self.iter()
    //            .map(|(k, v)| (k, v.unwrap()));
    //        Some(iter)
    //    }
    //}
}

