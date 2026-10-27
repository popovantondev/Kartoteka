#[path = "src/brand.rs"]
mod brand;
use std::{env, fs, path::PathBuf, process::Command};
fn main() {
    println!("cargo:rerun-if-changed=src/brand.rs");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=installer/app.manifest");
    if !env::var("TARGET").unwrap_or_default().contains("windows") {
        return;
    }
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let sizes = [16_u32, 20, 24, 32, 48, 64, 128, 256];
    let mut ico = vec![0, 0, 1, 0, sizes.len() as u8, 0];
    let mut images = Vec::new();
    let mut offset = 6 + 16 * sizes.len() as u32;
    for size in sizes {
        let rgba = brand::rgba(size);
        let mask_stride = ((size + 31) / 32) * 4;
        let bytes = 40 + size * size * 4 + mask_stride * size;
        ico.extend_from_slice(&[
            if size == 256 { 0 } else { size as u8 },
            if size == 256 { 0 } else { size as u8 },
            0,
            0,
            1,
            0,
            32,
            0,
        ]);
        ico.extend_from_slice(&bytes.to_le_bytes());
        ico.extend_from_slice(&offset.to_le_bytes());
        offset += bytes;
        let mut bmp = Vec::new();
        bmp.extend_from_slice(&40_u32.to_le_bytes());
        bmp.extend_from_slice(&size.to_le_bytes());
        bmp.extend_from_slice(&(size * 2).to_le_bytes());
        bmp.extend_from_slice(&[1, 0, 32, 0]);
        bmp.extend_from_slice(&0_u32.to_le_bytes());
        bmp.extend_from_slice(&(size * size * 4).to_le_bytes());
        bmp.extend_from_slice(&[0; 16]);
        for y in (0..size).rev() {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                bmp.extend_from_slice(&[rgba[i + 2], rgba[i + 1], rgba[i], rgba[i + 3]]);
            }
        }
        bmp.resize(bytes as usize, 0);
        images.push(bmp);
    }
    for bmp in images {
        ico.extend_from_slice(&bmp);
    }
    let icon = out.join("kartoteka.ico");
    fs::write(&icon, ico).unwrap();
    let rc = out.join("kartoteka.rc");
    fs::write(
        &rc,
        format!("1 ICON \"{}\"\n", icon.to_string_lossy().replace('\\', "/")),
    )
    .unwrap();
    let manifest =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("installer/app.manifest");
    let resource = format!(
        "{}\n1 24 \"{}\"\n",
        fs::read_to_string(&rc).unwrap(),
        manifest.to_string_lossy().replace('\\', "/")
    );
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    let numeric = version.replace('.', ",") + ",0";
    let version_resource = format!(
        r#"
1 VERSIONINFO
FILEVERSION {numeric}
PRODUCTVERSION {numeric}
FILEFLAGSMASK 0x3fL
FILEFLAGS 0x0L
FILEOS 0x40004L
FILETYPE 0x1L
BEGIN
 BLOCK "StringFileInfo"
 BEGIN
  BLOCK "040904B0"
  BEGIN
   VALUE "CompanyName", "popovantondev\0"
   VALUE "FileDescription", "Kartoteka\0"
   VALUE "FileVersion", "{version}\0"
   VALUE "ProductVersion", "{version}\0"
   VALUE "ProductName", "Kartoteka\0"
  END
 END
 BLOCK "VarFileInfo"
 BEGIN
  VALUE "Translation", 0x409, 1200
 END
END
"#
    );
    fs::write(&rc, resource + &version_resource).unwrap();
    let obj = out.join("kartoteka-icon.o");
    let linker = env::var_os("CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_LINKER").map(PathBuf::from);
    let windres = linker
        .as_ref()
        .and_then(|p| p.parent())
        .map(|p| p.join("x86_64-w64-mingw32-windres.exe"))
        .unwrap_or_else(|| PathBuf::from("windres"));
    let result = Command::new(windres)
        .arg("-i")
        .arg(rc)
        .arg("-o")
        .arg(&obj)
        .args(["-O", "coff"])
        .status()
        .expect("LLVM-MinGW windres is required for the application icon");
    assert!(
        result.success(),
        "Application icon resource compilation failed"
    );
    println!("cargo:rustc-link-arg-bins={}", obj.display());
}
