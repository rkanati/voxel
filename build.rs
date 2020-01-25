
//extern crate gl_generator;

use {
    std::{env, fs::File, path::Path},
    gl_generator::{Registry, Api, Profile, Fallbacks, GlobalGenerator}
};

fn main() {
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(&Path::new(&dest).join("bindings.rs")).unwrap();

    let extensions = [
        "GL_ARB_texture_filter_anisotropic"
    ];

    Registry::new(Api::Gl, (4, 5), Profile::Core, Fallbacks::All, extensions)
        .write_bindings(GlobalGenerator, &mut file)
        .unwrap();
}

