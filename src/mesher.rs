
use {
    crate::{
        math::*,
        block,
    },
    std::{
        mem,
        ptr::null as nullptr,
        rc::Rc,
    },
    gl::types::*,
};

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
}

impl Drop for VAO {
    fn drop(&mut self) {
        unsafe { gl::DeleteVertexArrays(1, &mut self.0); }
    }
}

pub trait MeshBuilder {
    type Mesh: Mesh + 'static;
    fn add_quad(&mut self, quad: &[Vertex; 4]);
    fn bake(&mut self) -> Self::Mesh;
}

pub trait Mesh {
    fn draw(&self);
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Vertex(V4u8);

impl Vertex {
    fn new() -> Vertex {
        Vertex(V4::zeros())
    }
}

/// A mesh using a precomputed index buffer
///
/// This kind of mesh uses indices to avoid redundant vertex data.
pub struct IndexedMesh {
    vao:       VAO,
    n_quads:   u32,
    instanced: bool,
}

impl Mesh for IndexedMesh {
    fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.vao.name());
            if self.instanced {
                gl::DrawElementsInstanced(
                    gl::TRIANGLES,
                    6, gl::UNSIGNED_BYTE,
                    nullptr(),
                    self.n_quads as i32
                );
            }
            else {
                gl::DrawElements(
                    gl::TRIANGLES,
                    self.n_quads as i32 * 6, gl::UNSIGNED_SHORT,
                    nullptr()
                );
            }
        }
    }
}

pub struct IndexedMeshBuilder {
    index_buffer: GLuint,
    vertices:     Vec<Vertex>,
    instanced:    bool,
}

impl Drop for IndexedMeshBuilder {
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, &self.index_buffer); }
    }
}

impl IndexedMeshBuilder {
    fn make_indices(n_quads_max: u16) -> GLuint {
        let indices: Vec<u16> =
            (0 .. n_quads_max * 4)
            .step_by(4)
            .flat_map(|i| {
                [0, 1, 2, 2, 1, 3]
                .iter()
                .map(move |off| i + *off)
            })
            .collect();

        unsafe {
            let mut buf: GLuint = 0;
            gl::CreateBuffers(1, &mut buf);
            gl::NamedBufferData(
                buf,
                mem::size_of_val(&indices) as isize,
                indices.as_ptr() as *const _,
                gl::STATIC_DRAW
            );

            buf
        }
    }

    pub fn new(instanced: bool) -> IndexedMeshBuilder {
        let index_buffer = Self::make_indices(if instanced { 1 } else { 1024 });
        IndexedMeshBuilder { index_buffer, vertices: Vec::new(), instanced }
    }
}

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
pub struct NaiveTriangleMesh {
    vao:     VAO,
    n_quads: u32,
}

pub struct NaiveTriangleMeshBuilder {
    vertices: Vec<Vertex>,
}

impl NaiveTriangleMeshBuilder {
    pub fn new() -> NaiveTriangleMeshBuilder {
        NaiveTriangleMeshBuilder { vertices: Vec::new() }
    }

    fn prepare_arrays(vertices: &[Vertex]) -> VAO {
        let vao = VAO::new();

        unsafe {
            let mut vert_buf: GLuint = 0;
            gl::CreateBuffers(1, &mut vert_buf);
            gl::NamedBufferData(
                vert_buf,
                (mem::size_of::<Vertex>() * vertices.len()) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW
            );

            gl::EnableVertexArrayAttrib(vao.name(), 0);
            gl::VertexArrayAttribFormat(vao.name(), 0, 4, gl::UNSIGNED_BYTE, gl::FALSE, 0);
            gl::VertexArrayAttribBinding(vao.name(), 0, 0);

            gl::VertexArrayVertexBuffer(
                vao.name(), 0,
                vert_buf, 0, mem::size_of::<Vertex>() as i32
            );
        }

        vao
    }
}

impl MeshBuilder for NaiveTriangleMeshBuilder {
    type Mesh = NaiveTriangleMesh;

    fn add_quad(&mut self, quad: &[Vertex; 4]) {
        let verts = [quad[0], quad[1], quad[2], quad[2], quad[1], quad[3]];
        self.vertices.extend(&verts);
    }

    fn bake(&mut self) -> NaiveTriangleMesh {
        let n_quads = (self.vertices.len() / 6) as u32;
        //eprintln!("baking mesh: {} quads", n_quads);
        let vao = Self::prepare_arrays(&self.vertices);
        self.vertices.clear();
        NaiveTriangleMesh { vao, n_quads }
    }
}

impl Mesh for NaiveTriangleMesh {
    fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.vao.name());
            gl::DrawArrays(gl::TRIANGLES, 0, self.n_quads as i32 * 6);
        }
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

impl Mesher for Simple {
    fn make_mesh(&self, input: block::Slice, builder: &mut impl MeshBuilder)
        -> Rc<dyn Mesh>
    {
        enum Direction {
            ZOut = 0,
            YOut = 1,
            XOut = 2,
            ZIn  = 3,
            YIn  = 4,
            XIn  = 5,
        }
        use Direction::*;

        const PROTO_QUADS: [[[u8; 3]; 4]; 6] = [
            [[1,1,1], [0,1,1], [1,0,1], [0,0,1]],
            [[0,1,1], [1,1,1], [0,1,0], [1,1,0]],
            [[1,1,1], [1,0,1], [1,1,0], [1,0,0]],
            [[1,1,1], [1,0,1], [0,1,1], [0,0,1]],
            [[0,1,1], [0,1,0], [1,1,1], [1,1,0]],
            [[1,1,1], [1,1,0], [1,0,1], [1,0,0]],
        ];

        // careful with the indices here
        for xyz in SpaceIter::new(V3::new(0,0,0), input.dims() - V3::new(1,1,1)) {
            let pos = xyz.map(|x| x as u8);

            let quad = |dir| {
                let dir = dir as usize;
                let mut verts = [Vertex::new(); 4];
                let proto_quad: &[[u8; 3]; 4] = &PROTO_QUADS[dir];
                for i in 0..4 {
                    let offset = V3u8::from(proto_quad[i]);
                    verts[i] = Vertex((pos + offset).push(dir as u8));
                }
                verts
            };

            let block = input[xyz];

            let bz = input[xyz + V3::z()];
            let by = input[xyz + V3::y()];
            let bx = input[xyz + V3::x()];

            if block.is_nonempty() {
                if bz.is_empty()    { builder.add_quad(&quad(ZOut)); }
                if by.is_empty()    { builder.add_quad(&quad(YOut)); }
                if bx.is_empty()    { builder.add_quad(&quad(XOut)); }
            }
            else {
                if bz.is_nonempty() { builder.add_quad(&quad(ZIn)); }
                if by.is_nonempty() { builder.add_quad(&quad(YIn)); }
                if bx.is_nonempty() { builder.add_quad(&quad(XIn)); }
            }
        }

        Rc::new(builder.bake())
    }
}

