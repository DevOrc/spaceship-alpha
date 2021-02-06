use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use texture_packer::{
    exporter::ImageExporter, importer::ImageImporter, TexturePacker, TexturePackerConfig,
};

fn main() -> std::io::Result<()> {
    compile_shaders()?;
    pack_sprites()?;

    Ok(())
}

fn pack_sprites() -> std::io::Result<()> {
    let config = TexturePackerConfig {
        max_width: 256,
        allow_rotation: false,
        texture_outlines: false,
        border_padding: 2,
        trim: false,
        ..Default::default()
    };

    let mut packer: TexturePacker<image::DynamicImage> = TexturePacker::new_skyline(config);
    let root_path = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut src_dir = PathBuf::from(&root_path);
    src_dir.push("assets");
    src_dir.push("ui");
    src_dir.push("widgets");

    pack_folder(&mut packer, &src_dir.clone(), &src_dir)?;

    let atlas = ImageExporter::export(&packer).unwrap().to_rgba8();
    let (width, height) = atlas.dimensions();
    atlas
        .save("assets/ui/widgets.png")
        .expect("Unable to save atlas");
    // let mut file = File::create("assets/ui/widgets.png").unwrap();
    // atlas
    //     .write_to(&mut file, image::ImageFormat::Png)
    //     .unwrap();

    export_sprite_locations(&packer, width as f32, height as f32)?;

    Ok(())
}

fn export_sprite_locations(
    packer: &TexturePacker<image::DynamicImage>,
    atlas_width: f32,
    atlas_height: f32,
) -> std::io::Result<()> {
    let mut file = File::create("src/ui/widget_textures.rs").unwrap();

    write!(file, "//! Note: THIS FILE IS GENERATED BY BUILD.RS\n\n")?;

    for (name, frame) in packer.get_frames() {
        let name = name
            .replace("\\", "_")
            .replace("/", "_")
            .replace(".png", "")
            .to_uppercase();
        let frame = frame.frame;

        write!(
            file,
            "pub const {}: (f32, f32, f32, f32) = ({:.5}, {:.5}, {:.5}, {:.5});\n",
            name,
            frame.x as f32 / atlas_width,
            frame.y as f32 / atlas_height,
            frame.w as f32 / atlas_width,
            frame.h as f32 / atlas_height
        )?;
    }

    Ok(())
}

fn pack_folder(
    packer: &mut TexturePacker<image::DynamicImage>,
    path: &PathBuf,
    root_dir: &PathBuf,
) -> std::io::Result<()> {
    for entry in fs::read_dir(path)? {
        if let Ok(entry) = entry {
            let path = entry.path();

            if path.is_dir() {
                pack_folder(packer, &path, root_dir)?;
            } else {
                let texture = ImageImporter::import_from_file(&path)
                    .expect(&format!("Unable to import file: {:?}", path));

                let name = path.strip_prefix(root_dir).unwrap().to_str().unwrap();

                packer.pack_own(name.to_string(), texture).unwrap();
            }
        }
    }

    Ok(())
}

fn compile_shaders() -> std::io::Result<()> {
    let root_path = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut src_dir = PathBuf::from(root_path.clone());
    src_dir.push("assets");
    src_dir.push("shaders");

    for entry in fs::read_dir(src_dir)? {
        if let Ok(entry) = entry {
            let path = entry.path();

            if path.extension().unwrap() != "spv" {
                run_glslc(path);
            }
        }
    }

    Ok(())
}

fn run_glslc(path: PathBuf) {
    let extension = path.extension().unwrap().to_str().unwrap();
    let output = path.with_extension(format!("{}.spv", extension));

    let output = Command::new("glslc")
        .args(&[path.to_str().unwrap(), "-o", output.to_str().unwrap()])
        .output()
        .expect("failed to run glslc");

    if !output.status.success() {
        panic!(
            "Failed to compile shader {:?}: {}\n\n{}",
            path,
            output.status,
            std::str::from_utf8(&output.stderr).unwrap()
        );
    }
}
