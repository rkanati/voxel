
use {
    crate::{
        gl::{self, types::*},
        math::*,
    },
    image::Pixel,
};

#[derive(Debug)]
pub enum Error {
    Loading(image::ImageError),
    Dimensions,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Loading(image_err) => Some(image_err),
            _                         => None,
        }
    }
}

pub struct Texture2D(GLuint);

impl Texture2D {
    pub fn bind(&self) {
        unsafe { gl::BindTexture(gl::TEXTURE_2D, self.0); }
    }

    pub fn create_from_image(image: &image::RgbaImage, levels: u32) -> Result<Texture2D, Error> {
        let levels = levels.max(1);

        let width  = image.width()  as i32;
        let height = image.height() as i32;

        let tex = unsafe {
            let mut name: GLuint = 0;
            gl::CreateTextures(gl::TEXTURE_2D, 1, &mut name);

            let params = [
                (gl::TEXTURE_MIN_FILTER,     gl::NEAREST_MIPMAP_LINEAR),
                (gl::TEXTURE_MAG_FILTER,     gl::NEAREST),
                (gl::TEXTURE_MAX_ANISOTROPY, 8),
                (gl::TEXTURE_MAX_LEVEL,      levels),
                (gl::TEXTURE_WRAP_S,         gl::CLAMP_TO_EDGE),
                (gl::TEXTURE_WRAP_T,         gl::CLAMP_TO_EDGE),
            ];

            for (pname, val) in &params {
                gl::TextureParameteriv(name, *pname, &(*val as i32));
            }

            gl::TextureStorage2D(name, levels as i32, gl::RGBA8, width, height);

            Texture2D(name)
        };

        for level in 0 .. levels {
            // TODO optimize
            let width  = width  >> level;
            let height = height >> level;
            let reduction = image::imageops::resize(
                image,
                width as u32, height as u32,
                image::imageops::FilterType::Triangle
            );

            unsafe {
                gl::TextureSubImage2D(
                    tex.0, level as i32,
                    0, 0, width, height,
                    gl::RGBA, gl::UNSIGNED_BYTE, reduction.as_ptr() as *const _
                );
            }
        }

        Ok(tex)
    }

    pub fn load(path: impl AsRef<std::path::Path>, levels: u32) -> Result<Texture2D, Error> {
        let im =
            image::open(path).map_err(|e| Error::Loading(e))?
            .to_rgba();
        Self::create_from_image(&im, levels)
    }
}

pub struct TextureAtlas {
    texture:   Texture2D,
    padding:   V2,
    tile_dims: V2,
}

impl TextureAtlas {
    pub fn tile_dims(&self) -> V2 {
        self.tile_dims
    }

    pub fn padding(&self) -> V2 {
        self.padding
    }

    pub fn stride(&self) -> V2 {
        2. * self.padding + self.tile_dims
    }

    pub fn bind(&self) {
        self.texture.bind();
    }

    pub fn load(
        path:       impl AsRef<std::path::Path>,
        atlas_dims: V2<u32>,
        levels: u32)
        -> Result<TextureAtlas, Error>
    {
        assert!(atlas_dims.x > 0 && atlas_dims.x < 1000);
        assert!(atlas_dims.y > 0 && atlas_dims.y < 1000);

        let levels = levels.max(1);

        let atlas_dims = atlas_dims.map(|x| x as i32);
        let padding = 1 << (levels as i32 - 2).max(0);

        let src =
            image::open(path).map_err(|e| Error::Loading(e))?
            .to_rgba();

        let width  = src.width()  as i32;
        let height = src.height() as i32;
        let src_dims = V2::new(width, height);

        if src_dims.zip_map(&atlas_dims, |a, b| a % b) != V2::zeros() {
            return Err(Error::Dimensions);
        }

        let src_tile_dims = src_dims.component_div(&atlas_dims);
        let dst_tile_dims = src_tile_dims + V2::repeat(2 * padding);
        let dst_dims = dst_tile_dims.component_mul(&atlas_dims);

        let transparent = image::Rgba::from_channels(0, 0, 0, 0);
        let mut dst = image::RgbaImage::from_pixel(
            dst_dims.x as u32, dst_dims.y as u32,
            transparent
        );

        for tile_j in 0 .. atlas_dims.y {
            for tile_i in 0 .. atlas_dims.x {
                let ij = V2::new(tile_i, tile_j);
                let src_off = ij.component_mul(&src_tile_dims);
                let dst_off = ij.component_mul(&dst_tile_dims);

                for pixel_y in 0 .. dst_tile_dims.y {
                    for pixel_x in 0 .. dst_tile_dims.x {
                        let pixel_xy = V2::new(pixel_x, pixel_y);
                        let src_tile_rel = (pixel_xy - V2::repeat(padding))
                            .zip_map(&src_tile_dims, |a, b| a.min(b - 1).max(0));
                        let src_xy = src_off + src_tile_rel;

                        let dst_xy = dst_off + pixel_xy;

                        dst.put_pixel(
                            dst_xy.x as u32, dst_xy.y as u32,
                            *src.get_pixel(src_xy.x as u32, src_xy.y as u32)
                        );
                    }
                }
            }
        }

        // TODO dev option
        dst.save("atlas-pack.png");

        let dst_dims = dst_dims.map(|x| x as f32);

        let tile_dims = src_tile_dims
            .map(|x| x as f32)
            .component_div(&dst_dims);

        let padding = V2::repeat(padding as f32).component_div(&dst_dims);

        let texture = Texture2D::create_from_image(&dst, levels)?;

        Ok(TextureAtlas {
            texture,
            padding,
            tile_dims,
        })
    }
}

impl Drop for Texture2D {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.0); }
    }
}

