use std::{
    env,
    fs::{read, write},
    path::{Path, PathBuf},
};

use shaderc::{CompileOptions, Compiler, ShaderKind, SourceLanguage};

fn shader<T: Into<PathBuf> + AsRef<Path>>(source: T, out: T, kind: ShaderKind) {
    let compiler = Compiler::new().unwrap();
    let mut options = CompileOptions::new().unwrap();
    options.set_source_language(SourceLanguage::HLSL);
    let result = compiler
        .compile_into_spirv(
            &str::from_utf8(&read(&source).unwrap()).unwrap(),
            kind,
            source.into().file_name().unwrap().to_str().unwrap(),
            "main",
            Some(&options),
        )
        .unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();
    write(Path::new(&out_dir).join(out.into()), result.as_binary_u8()).unwrap();
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=resources/*");
    println!("cargo::rerun-if-changed=resources/");

    shader(
        "resources/rect.vert.hlsl",
        "rect.vert.spv",
        ShaderKind::Vertex,
    );
    shader(
        "resources/rect.frag.hlsl",
        "rect.frag.spv",
        ShaderKind::Fragment,
    );
}
