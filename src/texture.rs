
use {
    gl::types::*,
};

pub struct Texture2D(GLuint);

impl Texture2D {
    pub fn bind(&self) {
        unsafe { gl::BindTexture(gl::TEXTURE_2D, self.0); }
    }

    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Texture2D, Error> {
        let im = image::open(path)?.to_rgba();
        let width  = im.width()  as i32;
        let height = im.height() as i32;

        unsafe {
            let mut name: GLuint = 0;
            gl::CreateTextures(gl::TEXTURE_2D, 1, &mut name);

            const PARAMS: [(GLenum, GLuint); 5] = [
                (gl::TEXTURE_MIN_FILTER, gl::NEAREST),
                (gl::TEXTURE_MAG_FILTER, gl::NEAREST),
                (gl::TEXTURE_MAX_LEVEL,  0),
                (gl::TEXTURE_WRAP_S,     gl::CLAMP_TO_EDGE),
                (gl::TEXTURE_WRAP_T,     gl::CLAMP_TO_EDGE),
            ];
            for (pname, val) in &PARAMS {
                gl::TextureParameteriv(name, *pname, &(*val as i32));
            }

            gl::TextureStorage2D(name, 1, gl::RGBA8, width, height);
            gl::TextureSubImage2D(
                name, 0,
                0, 0, width, height,
                gl::RGBA, gl::UNSIGNED_BYTE, im.into_raw().as_ptr() as *const std::ffi::c_void
            );

            Ok(Texture2D(name))
        }
    }
}

impl Drop for Texture2D {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.0); }
    }
}

pub type Error = image::ImageError;

//pub fn load_atlas(
//    path: impl AsRef<std::path::Path>,
//    tile_width:  i32,
//    tile_height: i32,
////  base_index:  u32,
//)   -> Result<GLuint, Box<dyn Error>>
//{
//    assert!(tile_width > 1 && tile_height > 1, "Invalid parameters");
//
//    eprintln!("loading {}", path.as_ref().display());
//    let im = image::open(path)?.to_rgba();
//    let width  = im.width()  as i32;
//    let height = im.height() as i32;
//
//    if width % tile_width != 0 || height % tile_height != 0 {
//        // warn about margin?
//    }
//
//    let columns = width  / tile_width;
//    let rows    = height / tile_height;
//    let tile_count = columns * rows;
//
//    unsafe {
//        let mut tex: GLuint = 0;
//        gl::CreateTextures(gl::TEXTURE_2D_ARRAY, 1, &mut tex);
//
//        const PARAMS: [(GLenum, GLuint); 5] = [
//            (gl::TEXTURE_MIN_FILTER, gl::NEAREST),
//            (gl::TEXTURE_MAG_FILTER, gl::NEAREST),
//            (gl::TEXTURE_MAX_LEVEL,  0),
//            (gl::TEXTURE_WRAP_S,     gl::CLAMP_TO_EDGE),
//            (gl::TEXTURE_WRAP_T,     gl::CLAMP_TO_EDGE),
//        ];
//        for (pname, val) in &PARAMS {
//            gl::TextureParameteriv(tex, *pname, &(*val as i32));
//        }
//
//        gl::TextureStorage3D(tex, 1, gl::RGBA8, tile_width, tile_height, tile_count);
//
//        for tile_y in 0..rows {
//            for tile_x in 0..columns {
//                let index = tile_y * columns + tile_x;
//                let tile_image_buf = im
//                    .view(
//                        (tile_x * tile_width) as u32,
//                        (tile_y * tile_height) as u32,
//                        tile_width as u32,
//                        tile_height as u32,
//                    )
//                    .to_image()
//                    .into_vec();
//
//                gl::TextureSubImage3D(
//                    tex, 0,
//                    0, 0, index,
//                    tile_width, tile_height, 1,
//                    gl::RGBA, gl::UNSIGNED_BYTE,
//                    tile_image_buf.as_ptr() as *const std::ffi::c_void
//                );
//            }
//        }
//
//        Ok(tex)
//    }
//}

