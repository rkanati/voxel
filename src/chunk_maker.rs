
use {
    crate::{
        block::Block,
        chunk::{self, Array, Chunk, Coords as ChunkCoords},
        chunk_source::ChunkMaker,
        halton::*,
        math::*,
    },
    noise::{self, NoiseFn, Seedable},
};

type HeightNoise = noise::Fbm;
type ForestNoise = noise::Perlin;

pub struct Test {
    height_noise: HeightNoise,
    forest_noise: ForestNoise,

    //tree_buf: Vec<V2u8>,
}

impl Test {
    pub fn new(seed: u64) -> Test {
        let seed = seed as u32;
        Test {
            height_noise: HeightNoise::new().set_seed(seed),
            forest_noise: ForestNoise::new().set_seed(seed),

            //tree_buf: Vec::new(),
        }
    }
}

trait NoiseFnExt : NoiseFn<[f64; 2]> {
    fn at(&self, pos: V2) -> f32 {
        let pos = [pos.x as f64, pos.y as f64];
        self.get(pos) as f32
    }
}

impl<N> NoiseFnExt for N
    where N: NoiseFn<[f64; 2]>
{ }


impl ChunkMaker for Test {
    fn make(&self, chunk_coords: ChunkCoords) -> Vec<(ChunkCoords, Chunk)> {
        const DIM: usize = chunk::DIM as usize;

        let chunk_xyz = chunk_coords.unwrap();
        let chunk_xy  = chunk_xyz.xy();

        const FLOOR:   i32 = -4;
        const CEILING: i32 =  4;

        if !(FLOOR .. CEILING).contains(&chunk_xyz.z) {
            return vec![(chunk_coords, Array::new_filled(Block::Empty).into())];
        }

        let uns = |x| (x + 1.) * 0.5;

        const TREE_MIN: i32 =  4;
        const TREE_MAX: i32 = 10;

        let get_tree_height = {
            let n_trees = {
                let coords = chunk_xy.map(|x| x as f32 * 0.1);
                const SCALE: f32 = 3.;
                let raw = (uns(self.forest_noise.at(coords)) - 1. + (1. / SCALE)) * SCALE;
                const MAX: f32 = 30.;
                (raw * MAX).max(0.).min(MAX).trunc() as usize
            };
            //dbg!(n_trees);

            let adj = |h: &mut Halton, min, max| {
                let raw = min as f32 + h.next() * (max - min) as f32;
                (raw as i32).min(max).max(min)
            };

            let mut heights = [[0i32; DIM]; DIM];

            //std::iter::from_fn({
            //        let mut xs = Halton::new_seed(2, chunk_xy.x as u32);
            //        let mut ys = Halton::new_seed(3, chunk_xy.y as u32);
            //        let adj = |h: &mut Halton| adj(h, 0, chunk::DIM-1);
            //        move || Some((adj(&mut xs), adj(&mut ys)))
            //    })
            //    .take(n_trees)
            //    .for_each({
            //        let mut hs = Halton::new_seed(5, chunk_xy.x as u32);
            //        move |(x, y)| heights[y as usize][x as usize] =
            //            adj(&mut hs, TREE_MIN, TREE_MAX)
            //    });

            let mut xs = Halton::new_seed(2, chunk_xy.x as u32);
            let mut ys = Halton::new_seed(3, chunk_xy.y as u32);
            let mut hs = Halton::new_seed(5, chunk_xy.x as u32);
            for _ in 0 .. n_trees {
                let adj_xy = |h: &mut Halton| adj(h, 0, chunk::DIM-1) as usize;
                let x = adj_xy(&mut xs);
                let y = adj_xy(&mut ys);
                let h = adj(&mut hs, TREE_MIN, TREE_MAX);
                heights[y][x] = h;
            }

            //if n_trees > 0 {
            //    dbg!(&heights);
            //}

            move |xy: V2i32| heights[xy.y as usize][xy.x as usize]
        };

        let get_ground_height = {
            let chunk_mins = chunk_xy.map(|x| (x * chunk::DIM) as f32);
            let mut heights = [[0i32; DIM]; DIM];
            for y in 0 .. DIM {
                for x in 0 .. DIM {
                    let p = chunk_mins + V2::new(x as f32, y as f32);
                    let value = uns(self.height_noise.at(p * 0.005));
                    const MIN: f32 = -24.;
                    const MAX: f32 =  24.;
                    heights[y][x] = (MIN + value * (MAX - MIN)).trunc() as i32;
                }
            }

            move |xy: V2i32| heights[xy.y as usize][xy.x as usize]
        };

        (FLOOR .. CEILING)
            .map(|chunk_z| {
                let chunk = Array::generate(|rel| {
                    let rel = rel.map(|x| x as i32);
                    let ground_height = get_ground_height(rel.xy());
                    let block_height = chunk_z * chunk::DIM + rel.z;
                    let altitude = block_height - ground_height;

                    use Block::*;
                    if      altitude < -4       { Stone }
                    else if altitude <  0       { Soil  }
                    else if altitude <  1       { Grass }
                    else if altitude > TREE_MAX { Empty }
                    else {
                        let tree_height = get_tree_height(rel.xy());
                        if altitude > tree_height { Empty     }
                        else                      { TreeTrunk }
                    }
                }).into();

                let coords = ChunkCoords::new(chunk_xy.push(chunk_z).into());
                (coords, chunk)
            })
            .collect()
    }
}

