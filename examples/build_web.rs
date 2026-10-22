use image::{Rgb, RgbImage};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{error::Error, fs, path::PathBuf, process::Command};

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<Package>,
    target_directory: PathBuf,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    manifest_path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--locked",
            "--offline",
            "--format-version",
            "1",
            "--filter-platform",
            "wasm32-unknown-unknown",
        ])
        .output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
    }
    let metadata: Metadata = serde_json::from_slice(&output.stdout)?;
    let package = metadata
        .packages
        .iter()
        .find(|p| p.name == "miniquad")
        .ok_or("No se encontro Miniquad en cargo metadata")?;
    let crate_dir = package
        .manifest_path
        .parent()
        .ok_or("Ruta de Miniquad invalida")?;
    let dist = PathBuf::from("dist");
    fs::create_dir_all(dist.join("licenses"))?;
    for source in fs::read_dir("web")? {
        let source = source?.path();
        if source.is_file() {
            fs::copy(
                &source,
                dist.join(source.file_name().ok_or("Archivo sin nombre")?),
            )?;
        }
    }
    fs::copy(crate_dir.join("js/gl.js"), dist.join("gl.js"))?;
    fs::copy(
        metadata
            .target_directory
            .join("wasm32-unknown-unknown/release/memorama_arcade.wasm"),
        dist.join("memorama_arcade.wasm"),
    )?;
    for name in ["LICENSE-MIT", "LICENSE-APACHE"] {
        fs::copy(crate_dir.join(name), dist.join("licenses").join(name))?;
    }
    for size in [180, 192, 512] {
        let icon = RgbImage::from_fn(size, size, |x, y| {
            let (u, v) = (x * 32 / size, y * 32 / size);
            let mut color = [13, 13, 26];
            if (7..19).contains(&u) && (8..23).contains(&v) {
                color = if u == 7 || u == 18 || v == 8 || v == 22 {
                    [70, 130, 220]
                } else {
                    [0, 25, 95]
                };
            }
            if (14..26).contains(&u) && (11..26).contains(&v) {
                color = if u == 14 || u == 25 || v == 11 || v == 25 {
                    [255, 204, 0]
                } else {
                    [0, 90, 55]
                };
            }
            if ((17..=22).contains(&u) && v == 15)
                || (u == 22 && (15..=18).contains(&v))
                || (u == 20 && ((18..=20).contains(&v) || v == 22))
            {
                color = [255, 255, 255];
            }
            Rgb(color)
        });
        icon.save(dist.join(format!("icon-{size}.png")))?;
    }
    let worker = dist.join("sw.js");
    let mut paths = fs::read_dir(&dist)?
        .map(|p| p.map(|p| p.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    let mut digest = Sha256::new();
    for path in paths {
        if path.is_file() && path != worker {
            digest.update(
                path.file_name()
                    .ok_or("Archivo sin nombre")?
                    .to_string_lossy()
                    .as_bytes(),
            );
            digest.update(fs::read(path)?);
        }
    }
    let hash = format!("{:x}", digest.finalize());
    fs::write(
        &worker,
        fs::read_to_string(&worker)?.replace("__BUILD_ID__", &hash[..20]),
    )?;
    println!("PWA lista en {}", dist.canonicalize()?.display());
    println!("Prueba local: miniserve dist --index index.html --port 8085");
    Ok(())
}
