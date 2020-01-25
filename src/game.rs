
use {
    crate::{
        block::{self, Block},
        chunk::{self, Chunk, BlockCoords, Coords as ChunkCoords},
        chunk_maker,
        chunk_source,
        chunk_store,
        math::*,
        mesher,
        shader,
        stage,
        texture::Texture2D,
    },
    std::rc::Rc,
};

const STAGE_RADIUS: i32 = 10;

const FOV: f32 = 90.;
const ZOOM_FACTOR: f32 = 5.;

const SPRINT_FACTOR: f32 = 3.;
const PLAYER_SPEED: f32 = 5.;

const EDIT_INTERVAL: f32 = 0.1;

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

#[derive(Clone, Copy)]
pub struct Inputs {
    pub fore:  bool,
    pub left:  bool,
    pub back:  bool,
    pub right: bool,
    pub up:    bool,
    pub down:  bool,
    pub fast:  bool,

    pub cam_delta: V2,
    pub zoom:  bool,

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
            fast:  false,

            cam_delta: V2::zeros(),
            zoom:      false,

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


struct Facing {
    pitch:  f32,
    yaw:    Complex,
    cached: std::cell::Cell<Option<(V2, V3)>>,
}

impl Facing {
    fn new() -> Facing {
        Facing {
            pitch:  0.,
            yaw:    Complex::new(0., 1.),
            cached: None.into(),
        }
    }

    fn get(&self) -> (V2, V3) {
        if let Some(facing) = self.cached.get() {
            return facing;
        }

        let flat = V2::new(self.yaw.re, self.yaw.im);
        let (s, c) = self.pitch.sin_cos();
        let facing = (c * flat).push(s);

        let result = (flat, facing);
        self.cached.set(Some(result));
        result
    }

    fn flat(&self) -> V2 {
        self.get().0
    }

    fn direction(&self) -> V3 {
        self.get().1
    }

    fn update(&mut self, d_pitch: f32, d_yaw: f32) {
        self.cached.set(None);

        self.pitch = (self.pitch - d_pitch)
            .min(PI *  0.47)
            .max(PI * -0.47);

        self.yaw *= Complex::from_polar(&1., &(-0.5 * d_yaw));
    }
}

pub struct Game {
    source:   ChunkSource,
    stage:    Stage,
    mesher:   Box<MesherFn>,
    mesh_buf: MeshingBuffer,
    atlas:    Texture2D,

    player_position: P3,
    player_facing:   Facing,
    zoom:            bool,

    selected_block: BlockCoords,
    edit_timer:     f32,
}

fn chunk_clip(
    coords: ChunkCoords,
    chunk:  &Chunk,
    hitbox: Option<Box3>,
    motion: Segment)
    -> Option<(BlockCoords, box3::Intersection)>
{
    let chunk_pos = coords.block_mins().unwrap_f32().into();

    // TODO this works, but stinks
    let chunk_box = {
        let b = Box3::with_dims(chunk_pos, V3::repeat(chunk::DIM as f32));
        if let Some(hitbox) = &hitbox { b.dilate(hitbox) }
        else                          { b }
    };

    // TODO simplify check when source-inside-box intersections work
    if !chunk_box.contains(motion.source()) && chunk_box.intersect(&motion).is_none() {
        return None;
    }

    let block_box = {
        let b = Box3::with_dims(chunk_pos, V3::repeat(1.));
        if let Some(hitbox) = &hitbox { b.dilate(hitbox) }
        else                          { b }
    };

    chunk.indexed_iter()
        .filter_map(|(ijk, block)| {
            if *block == Block::Empty { return None; }
            block_box.at(ijk.map(|x| x as f32).into())
                .intersect(&motion)
                .map(|ixn| (coords.block_at_offset(ijk.map(|x| x as u8)), ixn))
        })
        .min_by_key(|(_, ixn)| OrdFloat(ixn.lambda))
}

impl Game {
    pub fn new() -> Result<Game, Box<dyn std::error::Error>> {
        let source = ChunkSource::new(
            chunk_store::Null::new(),
            chunk_maker::Test::new(12345)
        );

        let stage = Stage::new(STAGE_RADIUS, ChunkCoords::origin());

        let mesher: Box<MesherFn> = {
            use mesher::Mesher;
            let mesher = mesher::Simple::new();
        //  let mut builder = mesher::NaiveTriangleMeshBuilder::new();
            let mut builder = mesher::InstancedQuadMeshBuilder::new();
            Box::new(move |buffer| mesher.make_mesh(buffer, &mut builder))
        };

        let mesh_buf = MeshingBuffer::new_filled(Block::Empty);

        let shader = {
        //  static V_SHADER_SRC: &'static str = include_str!("shader/test-v.glsl");
            static V_SHADER_SRC: &'static str = include_str!("shader/instanced-quad-vert.glsl");
            static F_SHADER_SRC: &'static str = include_str!("shader/test-f.glsl");
            let v_shader = shader::compile(shader::Stage::Vertex,   V_SHADER_SRC)?;
            let f_shader = shader::compile(shader::Stage::Fragment, F_SHADER_SRC)?;
            shader::link(&[v_shader, f_shader])?
        };

        let atlas = Texture2D::load("atlas.png")?;

        unsafe {
            shader.bind();
            gl::ClearColor(0.4, 0.6, 1.0, 1.0);
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
            atlas,

            player_position: P3::new(0., 0., 30.),
            player_facing:   Facing::new(),
            zoom:            false,

            selected_block: BlockCoords::origin(),
            edit_timer:     EDIT_INTERVAL,
        };

        Ok(game)
    }

    fn player_chunk_coords(&self) -> ChunkCoords {
        BlockCoords::containing(self.player_position)
            .chunk()
        //self.player_position.coords
        //    .map(|x| (x * (1. / chunk::DIM as f32)).floor() as i32)
        //    .into()
    }

    fn update_chunks(&mut self) {
        let stale_chunks = self.stage.relocate(self.player_chunk_coords());
        if !stale_chunks.is_empty() {
            //eprintln!("loading {} stale chunks...", stale_chunks.len());

            const MAX_LOAD: usize = 10;
            let mut load_count = 0;

            for stale_chunk in stale_chunks {
                use stage::StaleChunk::*;
                let coords = match stale_chunk {
                    Missing(coords) => { coords }

                    Evicted { old_coords, new_coords, value } => {
                        self.source.store(old_coords, value.chunk);
                        new_coords
                    }
                };

                if load_count < MAX_LOAD {
                    let (chunk, from) = self.source.load(coords);
                    let stage_chunk = StageChunk::new(chunk);
                    self.stage.insert_absolute(coords, stage_chunk);
                    if from == chunk_source::LoadedFrom::Maker {
                        load_count += 1;
                    }
                }
            }
        }
    }

    pub fn edit_blocks(&mut self, inputs: &Inputs, dt: f32) {
        let selection_beam = Segment::new(
            self.eye_position(),
            self.player_facing.direction() * 4.
        );

        let selection = self.world_clip(None, selection_beam);

        let (kill_block, build_block) = if let Some((block, hit)) = selection {
            let build_block = block + hit.normal.map(|x| x as i32);
            (block, build_block)
        }
        else {
            let block = BlockCoords::containing(selection_beam.destination());
            (block, block)
        };

        self.selected_block = kill_block;

        // TODO actual click detection
        if self.edit_timer > 0. {
            self.edit_timer -= dt;
            return;
        }

        if inputs.build == inputs.smash {
            return;
        }

        let (value, coords, offset) = if inputs.build {
            let player_block = BlockCoords::containing(self.player_position);
            let offset = build_block - player_block;
            if offset.xy() == V2::zeros() && (0..=1).contains(&offset.z) {
                // don't concrete over your own feet
                return;
            }

            let (coords, offset) = build_block.chunk_and_offset();
            (Block::Stone, coords, offset)
        }
        else {
            let (coords, offset) = kill_block.chunk_and_offset();
            (Block::Empty, coords, offset)
        };

        if let Some(chunk) = self.stage.at_absolute_mut(coords) {
            chunk.chunk[offset] = value;

            // TODO proper mesh invalidation
            chunk.mesh = None;

            if offset.x == 0 {
                if let Some(chunk) = self.stage.at_absolute_mut(coords - V3::x()) {
                    chunk.mesh = None;
                }
            }

            if offset.y == 0 {
                if let Some(chunk) = self.stage.at_absolute_mut(coords - V3::y()) {
                    chunk.mesh = None;
                }
            }

            if offset.z == 0 {
                if let Some(chunk) = self.stage.at_absolute_mut(coords - V3::z()) {
                    chunk.mesh = None;
                }
            }
        }

        self.edit_timer = EDIT_INTERVAL;
    }

    fn world_clip(&self, hitbox: Option<Box3>, segment: Segment)
        -> Option<(BlockCoords, box3::Intersection)>
    {
        let mins = ChunkCoords::containing(segment.source()) - V3::repeat(1);
        let maxs = ChunkCoords::containing(segment.destination()) + V3::repeat(2);
        let range = SpaceIter::new(mins.unwrap(), maxs.unwrap());

        // TODO test a more sensible set of chunks/blocks
        let mut nearest_hit: Option<(BlockCoords, box3::Intersection)> = None;
        for coords in range.map(|coords| ChunkCoords::new(coords.into())) {
            let chunk = if let Some(chunk) = self.stage.at_absolute(coords) {
                &chunk.chunk
            }
            else {
                // TODO improve behaviour when nearby chunks are not loaded
                //      should probably be an error
                return None;
            };

            if let Some((block, ixn)) = chunk_clip(coords, chunk, hitbox, segment) {
                if ixn.lambda < nearest_hit.map(|(_, nh)| nh.lambda).unwrap_or(std::f32::INFINITY) {
                    nearest_hit = Some((block, ixn));
                }
            }
        }

        nearest_hit
    }

    fn move_player(&mut self, inputs: &Inputs, dt: f32) {
        let move_intent = {
            let move_fore = self.player_facing.flat();
            let move_right = V2::new(move_fore.y, -move_fore.x);

            (inputs.fore  as i32 - inputs.back as i32) as f32 * move_fore.push(0.) +
            (inputs.right as i32 - inputs.left as i32) as f32 * move_right.push(0.) +
            (inputs.up    as i32 - inputs.down as i32) as f32 * V3::z()
        };

        let speed = PLAYER_SPEED * if inputs.fast { SPRINT_FACTOR } else { 1. };

        let player_box = Box3::with_dims(P3::new(-0.4, -0.4, 0.0), V3::new(0.8, 0.8, 1.6));

        let mut remaining = 1.;
        let mut stride = dt * speed * move_intent;

        while remaining > 0. {
            let motion = Segment::new(self.player_position, stride);
            let nearest_hit = self.world_clip(Some(player_box), motion);

            if let Some((_, hit)) = nearest_hit {
                dbg!(hit.normal);
                self.player_position += hit.lambda * stride;
                remaining *= 1. - hit.lambda;
                stride += hit.normal * -hit.normal.dot(&stride);
            }
            else {
                self.player_position += stride;
                break;
            }
        }
    }

    pub fn tick(&mut self, inputs: &Inputs, dt: f32) {
        const MOUSE_SPEED: f32 = 0.005;
        let mouse_speed = MOUSE_SPEED * if inputs.zoom { 1. / ZOOM_FACTOR } else { 1. };
        self.player_facing.update(
            inputs.cam_delta.y * mouse_speed,
            inputs.cam_delta.x * mouse_speed,
        );

        self.move_player(inputs, dt);
        self.update_chunks();
        self.edit_blocks(inputs, dt);

        self.zoom = inputs.zoom;
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

    fn eye_position(&self) -> P3 {
        self.player_position + 1.5f32 * V3::z()
    }

    pub fn draw(&mut self, screen_dims: V2) {
        self.refresh_meshes();

        let aspect = screen_dims.x / screen_dims.y;

        let fov = FOV * if self.zoom { 1. / ZOOM_FACTOR } else { 1. };
        let view_to_clip = Perspective::new(aspect, fov * (PI / 180.0), 0.1, 1000.0);

        let eye_position = self.eye_position();

        let world_to_view = Motion::look_at_rh(
            &eye_position,
            &(eye_position + self.player_facing.direction()),
            &V3::z()
        ).to_homogeneous();

        let world_to_clip
            = view_to_clip.as_matrix()
            * world_to_view;

        unsafe {
            gl::Viewport(0, 0, screen_dims.x as i32, screen_dims.y as i32);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        self.atlas.bind();

        for chunk_coords in self.stage.absolute_coords_iter() {
            if let Some(chunk) = self.stage.at_absolute(chunk_coords) {
                if let Some(mesh) = &chunk.mesh {
                    let model_to_world = na::Translation::from(
                        chunk_coords.unwrap().map(|x| (x * chunk::DIM) as f32)
                    );

                    let model_to_clip
                        = world_to_clip
                        * model_to_world.to_homogeneous();

                    unsafe {
                        gl::UniformMatrix4fv(0, 1, gl::FALSE, model_to_clip.as_ptr());
                        // TODO don't hardcode texture scale
                        gl::Uniform1f(1, 1. / 16.);
                    }

                    let selected_offset = self.selected_block - chunk_coords.block_mins();
                    // TODO try_map = transpose . map
                    let selected = if
                        (-1 ..= chunk::DIM).contains(&selected_offset.x) &&
                        (-1 ..= chunk::DIM).contains(&selected_offset.y) &&
                        (-1 ..= chunk::DIM).contains(&selected_offset.z)
                    {
                        Some(selected_offset.map(|x| x as i8))
                    }
                    else {
                        None
                    };

                    mesh.draw(selected);
                }
            }
        }
    }
}

