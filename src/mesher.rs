
use {
    crate::{
        gl::{self, types::*},
        block::{self, Block},
        math::*,
    },
    std::{
        mem,
        ptr::null as nullptr,
        rc::Rc,
    },
    rgb,
};

type RGB = rgb::RGB<u8>;

pub struct VAO(GLuint);

impl VAO {
    pub fn new() -> VAO {
        let mut name: GLuint = 0;
        unsafe { gl::CreateVertexArrays(1, &mut name); }
        VAO(name)
    }

    pub fn name(&self) -> GLuint {
        self.0
    }

    pub fn bind(&self) {
        unsafe { gl::BindVertexArray(self.0); }
    }
}

impl Drop for VAO {
    fn drop(&mut self) {
        unsafe { gl::DeleteVertexArrays(1, &mut self.0); }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    ZOut = 0,
    YOut = 1,
    XOut = 2,
    ZIn  = 3,
    YIn  = 4,
    XIn  = 5,
}

pub trait MeshBuilder {
    type Mesh: Mesh + 'static;
    fn add_quad(&mut self,
        pos: V3u8,
        dir: Direction,
        color: RGB,
        tcoords: V2u8,
        rotate: u8,
    );
    fn bake(&mut self) -> Self::Mesh;
}

pub trait Mesh {
    fn draw(&self, selected: Option<V3i8>);
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Vertex(V4u8);

//impl Vertex {
//    fn new() -> Vertex {
//        Vertex(V4::zeros())
//    }
//}

/// A mesh using a precomputed index buffer
///
/// This kind of mesh uses indices to avoid redundant vertex data.
//pub struct IndexedMesh {
//    vao:       VAO,
//    n_quads:   u32,
//    instanced: bool,
//}
//
//impl Mesh for IndexedMesh {
//    fn draw(&self) {
//        unsafe {
//            gl::BindVertexArray(self.vao.name());
//            if self.instanced {
//                gl::DrawElementsInstanced(
//                    gl::TRIANGLES,
//                    6, gl::UNSIGNED_BYTE,
//                    nullptr(),
//                    self.n_quads as i32
//                );
//            }
//            else {
//                gl::DrawElements(
//                    gl::TRIANGLES,
//                    self.n_quads as i32 * 6, gl::UNSIGNED_SHORT,
//                    nullptr()
//                );
//            }
//        }
//    }
//}
//
//pub struct IndexedMeshBuilder {
//    index_buffer: GLuint,
//    vertices:     Vec<Vertex>,
//    instanced:    bool,
//}
//
//impl Drop for IndexedMeshBuilder {
//    fn drop(&mut self) {
//        unsafe { gl::DeleteBuffers(1, &self.index_buffer); }
//    }
//}
//
//impl IndexedMeshBuilder {
//    fn make_indices(n_quads_max: u16) -> GLuint {
//        let indices: Vec<u16> =
//            (0 .. n_quads_max * 4)
//            .step_by(4)
//            .flat_map(|i| {
//                [0, 1, 2, 2, 1, 3]
//                .iter()
//                .map(move |off| i + *off)
//            })
//            .collect();
//
//        unsafe {
//            let mut buf: GLuint = 0;
//            gl::CreateBuffers(1, &mut buf);
//            gl::NamedBufferData(
//                buf,
//                mem::size_of_val(&indices) as isize,
//                indices.as_ptr() as *const _,
//                gl::STATIC_DRAW
//            );
//
//            buf
//        }
//    }
//
//    pub fn new(instanced: bool) -> IndexedMeshBuilder {
//        let index_buffer = Self::make_indices(if instanced { 1 } else { 1024 });
//        IndexedMeshBuilder { index_buffer, vertices: Vec::new(), instanced }
//    }
//}
//
//impl MeshBuilder for IndexedMeshBuilder {
//    type Mesh = IndexedMesh;
//
//    fn add_quad(&mut self, quad: &[Vertex; 4]) {
//        self.vertices.extend(quad);
//    }
//
//    fn bake(&mut self) -> IndexedMesh {
//        let n_quads = (self.vertices.len() / 4) as u32;
//        let vao = prepare_arrays(&self.vertices);
//        self.vertices.clear();
//        unsafe { gl::VertexArrayElementBuffer(vao.name(), self.index_buffer); }
//        IndexedMesh { vao, n_quads, instanced: self.instanced }
//    }
//}

/// A mesh using raw triangles
///
/// This kind of mesh uses redundant vertices to draw triangles with no indexing.
//pub struct NaiveTriangleMesh {
//    vao:     VAO,
//    n_quads: u32,
//}
//
//impl Mesh for NaiveTriangleMesh {
//    fn draw(&self) {
//        self.vao.bind();
//        unsafe { gl::DrawArrays(gl::TRIANGLES, 0, self.n_quads as i32 * 6); }
//    }
//}
//
//pub struct NaiveTriangleMeshBuilder {
//    vertices: Vec<Vertex>,
//    colors:   Vec<RGB>,
//}
//
//impl NaiveTriangleMeshBuilder {
//    pub fn new() -> NaiveTriangleMeshBuilder {
//        NaiveTriangleMeshBuilder {
//            vertices: Vec::new(),
//            colors:   Vec::new(),
//        }
//    }
//
//    fn prepare_arrays(vertices: &[Vertex], colors: &[RGB]) -> VAO {
//        let vao = VAO::new();
//
//        unsafe {
//            let mut vert_buf: GLuint = 0;
//            gl::CreateBuffers(1, &mut vert_buf);
//            gl::NamedBufferData(
//                vert_buf,
//                (mem::size_of::<Vertex>() * vertices.len()) as isize,
//                vertices.as_ptr() as *const _,
//                gl::STATIC_DRAW
//            );
//
//            gl::EnableVertexArrayAttrib(vao.name(), 0);
//            gl::VertexArrayAttribFormat(vao.name(), 0, 4, gl::UNSIGNED_BYTE, gl::FALSE, 0);
//            gl::VertexArrayAttribBinding(vao.name(), 0, 0);
//
//            gl::VertexArrayVertexBuffer(
//                vao.name(), 0,
//                vert_buf, 0, mem::size_of::<Vertex>() as i32
//            );
//
//            let mut color_buf: GLuint = 0;
//            gl::CreateBuffers(1, &mut color_buf);
//            gl::NamedBufferData(
//                color_buf,
//                (mem::size_of::<RGB>() * colors.len()) as isize,
//                colors.as_ptr() as *const _,
//                gl::STATIC_DRAW
//            );
//
//            gl::EnableVertexArrayAttrib(vao.name(), 1);
//            gl::VertexArrayAttribFormat(vao.name(), 1, 4, gl::UNSIGNED_BYTE, gl::TRUE, 0);
//            gl::VertexArrayAttribBinding(vao.name(), 1, 1);
//            //gl::VertexArrayBindingDivisor(vao.name(), 1, 1);
//
//            gl::VertexArrayVertexBuffer(
//                vao.name(), 1,
//                color_buf, 0, mem::size_of::<RGB>() as i32
//            );
//        }
//
//        vao
//    }
//}
//
//impl MeshBuilder for NaiveTriangleMeshBuilder {
//    type Mesh = NaiveTriangleMesh;
//
//    fn add_quad(&mut self, pos: V3u8, dir: Direction, color: RGB, tcoords: V2) {
//        const PROTO_QUADS: [[[u8; 3]; 4]; 6] = [
//            [[1,1,1], [0,1,1], [1,0,1], [0,0,1]],
//            [[0,1,1], [1,1,1], [0,1,0], [1,1,0]],
//            [[1,1,1], [1,0,1], [1,1,0], [1,0,0]],
//            [[1,1,1], [1,0,1], [0,1,1], [0,0,1]],
//            [[0,1,1], [0,1,0], [1,1,1], [1,1,0]],
//            [[1,1,1], [1,1,0], [1,0,1], [1,0,0]],
//        ];
//
//        let dir = dir as usize;
//        let mut quad = [Vertex::new(); 4];
//        let proto_quad: &[[u8; 3]; 4] = &PROTO_QUADS[dir];
//        for i in 0..4 {
//            let offset = V3u8::from(proto_quad[i]);
//            quad[i] = Vertex((pos + offset).push(dir as u8));
//        }
//
//        let verts = [quad[0], quad[1], quad[2], quad[2], quad[1], quad[3]];
//        self.vertices.extend(&verts);
//        self.colors.extend(std::iter::repeat(color).take(6));
//    }
//
//    fn bake(&mut self) -> NaiveTriangleMesh {
//        let n_quads = (self.vertices.len() / 6) as u32;
//        //eprintln!("baking mesh: {} quads", n_quads);
//        let vao = Self::prepare_arrays(&self.vertices, &self.colors);
//        self.vertices.clear();
//        self.colors.clear();
//        NaiveTriangleMesh { vao, n_quads }
//    }
//}




/// A mesh using instanced rendering to draw quads directly
pub struct InstancedQuadMesh {
    vao:     VAO,
    n_quads: u32,
}

impl Mesh for InstancedQuadMesh {
    fn draw(&self, selected: Option<V3i8>) {
        self.vao.bind();
        unsafe {
            if let Some(selected) = selected {
                gl::Uniform3i(1, selected.x as i32, selected.y as i32, selected.z as i32);
            }
            else {
                gl::Uniform3i(1, 127, 127, 127);
            }
            gl::DrawArraysInstanced(gl::TRIANGLE_FAN, 0, 4, self.n_quads as i32);
        }
    }
}

#[repr(C, packed)]
struct Quad {
    pos_dir: V4u8,
    color:   V4u8,
    tcoords: V2u8,
    rot_:    V2u8,
}

pub struct InstancedQuadMeshBuilder {
    quads: Vec<Quad>,
}

impl InstancedQuadMeshBuilder {
    pub fn new() -> InstancedQuadMeshBuilder {
        InstancedQuadMeshBuilder {
            quads: Vec::new(),
        }
    }

    fn prepare_arrays(quads: &[Quad]) -> VAO {
        let vao = VAO::new();

        unsafe {
            let mut buf: GLuint = 0;
            gl::CreateBuffers(1, &mut buf);
            gl::NamedBufferData(
                buf,
                (mem::size_of::<Quad>() * quads.len()) as isize,
                quads.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            gl::EnableVertexArrayAttrib(vao.name(), 0);
            gl::VertexArrayAttribIFormat(vao.name(), 0, 4, gl::UNSIGNED_BYTE, 0);
            gl::VertexArrayAttribBinding(vao.name(), 0, 0);

            gl::EnableVertexArrayAttrib(vao.name(), 1);
            gl::VertexArrayAttribFormat(vao.name(), 1, 4, gl::UNSIGNED_BYTE, gl::TRUE, 4);
            gl::VertexArrayAttribBinding(vao.name(), 1, 0);

            gl::EnableVertexArrayAttrib(vao.name(), 2);
            gl::VertexArrayAttribIFormat(vao.name(), 2, 2, gl::UNSIGNED_BYTE, 8);
            gl::VertexArrayAttribBinding(vao.name(), 2, 0);

            gl::EnableVertexArrayAttrib(vao.name(), 3);
            gl::VertexArrayAttribIFormat(vao.name(), 3, 2, gl::UNSIGNED_BYTE, 10);
            gl::VertexArrayAttribBinding(vao.name(), 3, 0);

            gl::VertexArrayVertexBuffer(
                vao.name(), 0,
                buf, 0, mem::size_of::<Quad>() as i32
            );

            gl::VertexArrayBindingDivisor(vao.name(), 0, 1);
        }

        vao
    }
}

impl MeshBuilder for InstancedQuadMeshBuilder {
    type Mesh = InstancedQuadMesh;

    fn add_quad(&mut self,
        pos:     V3u8,
        dir:     Direction,
        color:   RGB,
        tcoords: V2u8,
        rot:     u8,
    ) {
        let color: [u8; 3] = color.into();
        let quad = Quad {
            pos_dir: pos.push(dir as u8),
            color:   V3u8::from(color).push(255),
            tcoords,
            rot_: V2u8::new(rot, 0),
        };
        self.quads.push(quad);
    }

    fn bake(&mut self) -> InstancedQuadMesh {
        let n_quads = self.quads.len() as u32;
        let vao = Self::prepare_arrays(&self.quads);
        self.quads.clear();
        InstancedQuadMesh { vao, n_quads }
    }
}

pub trait Mesher {
    fn make_mesh(&self, input: block::Slice, builder: &mut impl MeshBuilder)
        -> Rc<dyn Mesh>;
}

pub struct Simple {
}

impl Simple {
    pub fn new() -> Simple {
        Simple { }
    }
}

trait BlockProps : Copy {
    fn color(self) -> RGB;
    fn tcoords(self, dir: Direction) -> V2u8;
    fn rotate(self, dir: Direction) -> bool;
}

impl BlockProps for Block {
    fn color(self) -> RGB {
        use Block::*;
        match self {
            //Stone => RGB::new(180, 180, 180),
            //Grass => RGB::new( 32, 180,  32),
            //Soil  => RGB::new( 96,  48,  32),
            Empty => unreachable!(),
            _     => RGB::new(255, 255, 255),
        }
    }

    fn tcoords(self, dir: Direction) -> V2u8 {
        use {Block::*, Direction::*};
        let (x, y) = match self {
            Stone => (0, 0),

            Grass => match dir {
                ZOut => (1, 0),
                ZIn  => (2, 0),
                _    => (3, 0),
            },

            Soil => (2, 0),

            TreeTrunk => match dir {
                ZIn | ZOut => (5, 0),
                _          => (4, 0),
            }

            Empty => unreachable!(),
        };
        V2::new(x, y)
    }

    fn rotate(self, dir: Direction) -> bool {
        use {Block::*, Direction::*};
        match self {
            Stone | Soil
                => true,

            Grass if dir == ZOut
                => true,

            TreeTrunk if dir == ZOut || dir == ZIn
                => true,

            Empty => unreachable!(),
            _     => false
        }
    }
}


impl Mesher for Simple {
    fn make_mesh(&self, input: block::Slice, builder: &mut impl MeshBuilder)
        -> Rc<dyn Mesh>
    {
        use Direction::*;

        // careful with the indices here
        for xyz in SpaceIter::new(V3::zeros(), input.dims() - V3::new(1,1,1)) {
            let pos = xyz.map(|x| x as u8);
            let hash
                = pos.x.wrapping_mul(251)
                ^ pos.y.wrapping_mul(199)
                ^ pos.z.wrapping_mul(151);

            let block = input[xyz];

            let bz = input[xyz + V3::z()];
            let by = input[xyz + V3::y()];
            let bx = input[xyz + V3::x()];

            let mut add_quad = |pos: V3u8, dir: Direction, block: Block| {
                let color   = block.color();
                let tcoords = block.tcoords(dir);
                let rotate  = if block.rotate(dir) { hash } else { 0 };
                builder.add_quad(pos, dir, color, tcoords, rotate);
            };

            if block.is_nonempty() {
                if bz.is_empty() { add_quad(pos, ZOut, block); }
                if by.is_empty() { add_quad(pos, YOut, block); }
                if bx.is_empty() { add_quad(pos, XOut, block); }
            }
            else {
                if bz.is_nonempty() { add_quad(pos + V3::z(), ZIn, bz); }
                if by.is_nonempty() { add_quad(pos + V3::y(), YIn, by); }
                if bx.is_nonempty() { add_quad(pos + V3::x(), XIn, bx); }
            }
        }

        Rc::new(builder.bake())
    }
}

