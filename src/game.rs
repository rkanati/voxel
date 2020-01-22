
use {
    crate::{
        block::{self, Block},
        chunk::{self, Chunk},
        chunk_maker,
        chunk_source,
        chunk_store,
        math::*,
        mesher,
        shader,
        stage,
    },
    std::rc::Rc,
};

const STAGE_RADIUS: i32 = 4;

#[derive(Clone)]
struct StageChunk {
    chunk: Chunk,
    mesh:  Option<Rc<dyn mesher::Mesh>>,
}

impl StageChunk {
    fn new(chunk: Chunk) -> StageChunk {
        StageChunk { chunk, mesh: None }
    }
}

type Stage = stage::Stage<StageChunk>;

mod meshing_buffer {
    use crate::{array3d, chunk, block::Block};

    const DIM: usize = chunk::DIM as usize + 1;

    #[derive(Default, Clone)]
    pub struct Dims;

    impl array3d::StaticDims for Dims {
        const X: usize = DIM;
        const Y: usize = DIM;
        const Z: usize = DIM;
    }

    pub type Buffer = array3d::ArrayOwned<Block, Dims>;
}

use meshing_buffer::Buffer as MeshingBuffer;

fn fill_meshing_buffer(buffer: &mut MeshingBuffer, stage: &Stage, rel: V3i32)
    -> Option<()>
{
    const CDIM: usize = chunk::DIM as usize;

    let zero = V3::zeros();
    let dims = V3::repeat(CDIM);

    let c0 = &stage.at_relative(rel)?.chunk;
    let cx = &stage.at_relative(rel + V3::x())?.chunk;
    let cy = &stage.at_relative(rel + V3::y())?.chunk;
    let cz = &stage.at_relative(rel + V3::z())?.chunk;

    buffer.slice_mut(zero, dims)
        .copy_from(&c0.slice(zero, dims));

    buffer.slice_mut(V3::new(CDIM, 0, 0), V3::new(1, CDIM, CDIM))
        .copy_from(&cx.slice(zero, V3::new(1, CDIM, CDIM)));

    buffer.slice_mut(V3::new(0, CDIM, 0), V3::new(CDIM, 1, CDIM))
        .copy_from(&cy.slice(zero, V3::new(CDIM, 1, CDIM)));

    buffer.slice_mut(V3::new(0, 0, CDIM), V3::new(CDIM, CDIM, 1))
        .copy_from(&cz.slice(zero, V3::new(CDIM, CDIM, 1)));

    Some(())
}

#[cfg(test)]
#[test]
fn test_fill_meshing_buffer() {
    let chunk: Chunk = chunk::Array::new_filled(Block::Solid).into();
    let chunk = StageChunk::new(chunk);

    let mut stage = Stage::new(3, P3::origin());
    let stale = stage.relocate(P3::origin());
    for coords in stale {
        stage.insert_absolute(coords, chunk.clone());
    }

    let mut meshing_buffer = MeshingBuffer::new_filled(Block::Empty);

    let ok = fill_meshing_buffer(&mut meshing_buffer, &stage, V3::zeros());
    assert!(ok.is_some());

    let n_empty = meshing_buffer.iter()
        .filter(|block| block.is_empty())
        .count();
    assert_eq!(n_empty, chunk::DIM as usize * 3 + 1);
}

#[derive(Clone, Copy)]
pub struct Inputs {
    pub fore:  bool,
    pub left:  bool,
    pub back:  bool,
    pub right: bool,
    pub up:    bool,
    pub down:  bool,

    pub cam_delta: V2,

    pub build: bool,
    pub smash: bool,
}

impl Inputs {
    pub fn new() -> Inputs {
        Inputs {
            fore:  false,
            left:  false,
            back:  false,
            right: false,
            up:    false,
            down:  false,

            cam_delta: V2::zeros(),

            build: false,
            smash: false,
        }
    }

    pub fn take(&mut self) -> Inputs {
        let out = *self;
        self.cam_delta = V2::zeros();
        out
    }
}

type MesherFn = dyn for<'a> FnMut(block::Slice<'a>) -> Rc<dyn mesher::Mesh>;

type ChunkSource = chunk_source::Source<chunk_store::Null, chunk_maker::Test>;

pub struct Game {
    source:   ChunkSource,
    stage:    Stage,
    mesher:   Box<MesherFn>,
    mesh_buf: MeshingBuffer,
//  shader:   shader::Program,

    player_position: P3,
    pitch: f32,
    yaw:   Complex,
    facing: V3,
}

impl Game {
    pub fn new() -> Result<Game, Box<dyn std::error::Error>> {
        let source = ChunkSource::new(
            chunk_store::Null::new(),
            chunk_maker::Test::new()
        );

        let stage = Stage::new(STAGE_RADIUS, P3::origin());

        let mesher: Box<MesherFn> = {
            use mesher::Mesher;
            let mesher = mesher::Simple::new();
            let mut builder = mesher::NaiveTriangleMeshBuilder::new();
            Box::new(move |buffer| mesher.make_mesh(buffer, &mut builder))
        };

        let mesh_buf = MeshingBuffer::new_filled(Block::Empty);

        let shader = {
            static V_SHADER_SRC: &'static str = include_str!("shader/test-v.glsl");
            static F_SHADER_SRC: &'static str = include_str!("shader/test-f.glsl");
            let v_shader = shader::compile(shader::Stage::Vertex,   V_SHADER_SRC)?;
            let f_shader = shader::compile(shader::Stage::Fragment, F_SHADER_SRC)?;
            shader::link(&[v_shader, f_shader])?
        };

        unsafe {
            shader.bind();
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Enable(gl::CULL_FACE);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::BLEND);
            gl::Enable(gl::DEPTH_TEST);
        }

        let game = Game {
            source,
            stage,
            mesher,
            mesh_buf,
        //  shader,

            player_position: P3::origin(),
            pitch: 0.,
            yaw: Complex::new(1., 0.),
            facing: V3::zeros(), // TODO this sucks
        };

        Ok(game)
    }

    fn player_chunk_coords(&self) -> P3i32 {
        self.player_position.coords
            .map(|x| (x * (1. / chunk::DIM as f32)).floor() as i32)
            .into()
    }

    pub fn tick(&mut self, inputs: &Inputs, dt: f32) {
        const MOUSE_SPEED: f32 = 0.005;
        self.pitch =
            (self.pitch + inputs.cam_delta.y * MOUSE_SPEED)
            .min(PI *  0.45)
            .max(PI * -0.45);

        self.yaw *= Complex::from_polar(&1., &(inputs.cam_delta.x * MOUSE_SPEED * -0.5));


        let player_move = {
            let yaw = self.yaw * self.yaw;
            let move_fore  = V3::new(-yaw.im, yaw.re, 0.);
            let move_right = V3::new( yaw.re, yaw.im, 0.);

            (inputs.fore  as i32 - inputs.back as i32) as f32 * move_fore +
            (inputs.right as i32 - inputs.left as i32) as f32 * move_right +
            (inputs.up    as i32 - inputs.down as i32) as f32 * V3::z()
        };

        self.facing = {
            let yaw = Versor::from_quaternion(
                Quaternion::from_parts(self.yaw.re, self.yaw.im * V3::z())
            );

            type Vu3 = na::Unit<V3>;
            let pitch = Versor::from_axis_angle(&Vu3::new_unchecked(V3::x()), -self.pitch);

            (yaw * pitch).transform_vector(&V3::y())
        };

        const SPEED: f32 = 5.;
        self.player_position += dt * SPEED * player_move;

        let stale_chunks = self.stage.relocate(self.player_chunk_coords());
        if !stale_chunks.is_empty() {
            eprintln!("loading {} stale chunks...", stale_chunks.len());

            for stale_chunk in stale_chunks {
                use stage::StaleChunk::*;
                let coords = match stale_chunk {
                    Missing(coords) => { coords }

                    Evicted { old_coords, new_coords, value } => {
                        self.source.store(old_coords, value.chunk);
                        new_coords
                    }
                };

                let chunk = self.source.load(coords);
                let stage_chunk = StageChunk::new(chunk);
                self.stage.insert_absolute(coords, stage_chunk);
            }
        }

        if inputs.build != inputs.smash {
            let selected_position = self.player_position + self.facing * 3.;
            let selected_block = selected_position.coords.map(|x| x.floor());
            let selected_chunk = selected_block.map(|x| (x / chunk::DIM as f32).floor() as i32);
            let selected_block = (selected_block.map(|x| x as i32) - selected_chunk * chunk::DIM)
                .map(|x| x as usize);
            let selected_chunk: P3i32 = selected_chunk.into();
            if let Some(chunk) = self.stage.at_absolute_mut(selected_chunk) {
                dbg!(selected_chunk);
                dbg!(selected_block);
                chunk.chunk[selected_block] = if inputs.build {
                    Block::Solid
                }
                else {
                    Block::Empty
                };

                // TODO proper mesh invalidation
                chunk.mesh = None;

                if selected_block.x == 0 {
                    if let Some(chunk) = self.stage.at_absolute_mut(selected_chunk - V3::x()) {
                        chunk.mesh = None;
                    }
                }

                if selected_block.y == 0 {
                    if let Some(chunk) = self.stage.at_absolute_mut(selected_chunk - V3::y()) {
                        chunk.mesh = None;
                    }
                }

                if selected_block.z == 0 {
                    if let Some(chunk) = self.stage.at_absolute_mut(selected_chunk - V3::z()) {
                        chunk.mesh = None;
                    }
                }
            }
        }
    }

    fn refresh_meshes(&mut self) {
        for rel in SpaceIter::new(
            self.stage.relative_mins(),
            self.stage.relative_maxs() - V3::repeat(1))
        {
            match self.stage.at_relative(rel) {
                Some(chunk) if chunk.mesh.is_none() => { }
                _ => { continue; }
            };

            let mesh = {
                //eprintln!("meshing chunk {}", rel);
                //let abs = self.stage.relative_to_absolute(rel);
                let ok = fill_meshing_buffer(&mut self.mesh_buf, &self.stage, rel);
                if ok.is_none() { continue; }
                (self.mesher)(self.mesh_buf.whole_slice())
            };

            if let Some(chunk) = self.stage.at_relative_mut(rel) {
                chunk.mesh = Some(mesh);
            }
        }
    }

    pub fn draw(&mut self, screen_dims: V2) {
        self.refresh_meshes();

        let aspect = screen_dims.x / screen_dims.y;
        let view_to_clip = Perspective::new(aspect, 90.0 * (PI / 180.0), 0.1, 1000.0);

        let world_to_view = {
            Motion::look_at_rh(
                &self.player_position,
                &(self.player_position + self.facing),
                &V3::z()
            ).to_homogeneous()
            //let reference: Versor = Versor::face_towards(&V3::y(), &V3::z());

            //let translation = Translation::from(-self.player_position.coords);
            //let rotation = reference * pitch * yaw;

            //rotation * translation
        };

        let world_to_clip = view_to_clip.as_matrix()
                          * world_to_view;

        unsafe {
            gl::Viewport(0, 0, screen_dims.x as i32, screen_dims.y as i32);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        for chunk_coords in self.stage.absolute_coords_iter() {
            if let Some(chunk) = self.stage.at_absolute(chunk_coords) {
                if let Some(mesh) = &chunk.mesh {
                    let model_to_world = na::Translation::from(
                        chunk_coords.coords.map(|x| (x * chunk::DIM) as f32)
                    );

                    let model_to_clip = world_to_clip
                                      * model_to_world.to_homogeneous();

                    unsafe {
                        gl::UniformMatrix4fv(0, 1, gl::FALSE, model_to_clip.as_ptr());
                    }

                    mesh.draw();
                }
            }
        }
    }
}

