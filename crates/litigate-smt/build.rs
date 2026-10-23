//! Fetches the pinned Z3 release and points the linker at its static
//! `libz3.a`, so every build of this crate links the same solver.
//!
//! `z3-sys` (default features) declares `#[link(name = "z3")]`; this
//! script adds the release's `bin/` directory, which holds only the static
//! archive we extract, to the link search path. The archive is checked
//! against the SHA-256 pinned below before anything is extracted. Set
//! `LITIGATE_SMT_Z3_ARCHIVE` to a local copy of the zip for offline builds
//! (Nix, air-gapped CI); it is verified against the same hash.

use std::borrow::Cow;
use std::env;
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

/// The pinned Z3 release. Keep in step with `PINNED_Z3_VERSION` in
/// `src/z3_backend.rs` (a test checks the linked library reports it).
const Z3_VERSION: &str = "5.1.0";

/// Official release assets (<https://github.com/Z3Prover/z3/releases/tag/z3-5.1.0>)
/// with the SHA-256 digests GitHub publishes for them.
const ASSETS: &[(&str, &str, &str, &str)] = &[
    (
        "linux",
        "x86_64",
        "z3-5.1.0-x64-glibc-2.39.zip",
        "f47be8d27d3230e823bf1eeede2fe0abaca55bb78d0b59974370e6689a92284a",
    ),
    (
        "linux",
        "aarch64",
        "z3-5.1.0-arm64-glibc-2.38.zip",
        "2832cfd43c6862fbaf78834b5f1e35d031b4a87e85456b696c1c6f609cd5609e",
    ),
    (
        "macos",
        "x86_64",
        "z3-5.1.0-x64-osx-13.3.zip",
        "f25cabdb01a32942a42227c36902c0846c759dcaabef4aca3b2e86331447f6ff",
    ),
    (
        "macos",
        "aarch64",
        "z3-5.1.0-arm64-osx-13.3.zip",
        "81d29e934fd863079a74af35eecaeaef8047e0e12414d33ca322b358d68383db",
    ),
];

const ARCHIVE_ENV: &str = "LITIGATE_SMT_Z3_ARCHIVE";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed={ARCHIVE_ENV}");

    let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let Some(&(_, _, asset, sha256)) = ASSETS.iter().find(|a| a.0 == os && a.1 == arch) else {
        fail(&format!(
            "no pinned Z3 {Z3_VERSION} static release for target {arch}-{os}; supported: \
             x86_64/aarch64 Linux (glibc) and macOS (see crates/litigate-smt/README.md)"
        ));
    };

    let out = PathBuf::from(env::var("OUT_DIR").unwrap_or_else(|_| fail("OUT_DIR unset")));
    let lib_dir = out.join(format!("z3-{Z3_VERSION}-{sha256}"));
    let lib = lib_dir.join("libz3.a");
    if !(lib.exists() && lib_dir.join("LICENSE.txt").exists()) {
        let zip = load_archive(asset);
        let digest = hex(&Sha256::digest(&zip));
        if digest != sha256 {
            fail(&format!(
                "{asset}: SHA-256 {digest} does not match the pinned {sha256}; refusing to link it"
            ));
        }
        extract_lib(&zip, &lib_dir, &lib);
    }
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-env=LITIGATE_SMT_Z3_ASSET={asset}");
}

/// The release zip: from `LITIGATE_SMT_Z3_ARCHIVE` if set, else GitHub.
fn load_archive(asset: &str) -> Vec<u8> {
    if let Ok(path) = env::var(ARCHIVE_ENV) {
        println!("cargo:rerun-if-changed={path}");
        return fs::read(&path).unwrap_or_else(|e| fail(&format!("{ARCHIVE_ENV}={path}: {e}")));
    }
    let url = format!("https://github.com/Z3Prover/z3/releases/download/z3-{Z3_VERSION}/{asset}");
    let mut body = Vec::new();
    ureq::get(&url)
        .call()
        .and_then(|r| {
            r.into_body()
                .into_reader()
                .read_to_end(&mut body)
                .map_err(ureq::Error::from)
        })
        .unwrap_or_else(|e| {
            fail(&format!(
                "downloading {url}: {e}. Offline? Point {ARCHIVE_ENV} at a copy of {asset}."
            ))
        });
    body
}

/// Extracts `bin/libz3.a` and `LICENSE.txt` (and nothing else) into
/// `lib_dir`, after checking the licence is still MIT: cargo-deny sees the
/// `z3`/`z3-sys` crates, not this native archive, so the check lives here.
fn extract_lib(zip: &[u8], lib_dir: &Path, lib: &Path) {
    let mut archive = zip::ZipArchive::new(Cursor::new(zip))
        .unwrap_or_else(|e| fail(&format!("reading Z3 archive: {e}")));
    let licence = member(&mut archive, "/LICENSE.txt");
    if !String::from_utf8_lossy(&licence).contains("MIT License") {
        fail("Z3 archive LICENSE.txt is no longer the MIT License; review before linking");
    }
    let bytes = member(&mut archive, "/bin/libz3.a");
    fs::create_dir_all(lib_dir).unwrap_or_else(|e| fail(&format!("{}: {e}", lib_dir.display())));
    fs::write(lib_dir.join("LICENSE.txt"), licence)
        .unwrap_or_else(|e| fail(&format!("{}: {e}", lib_dir.display())));
    // Write then rename, so an interrupted build never leaves a torn archive.
    let tmp = lib.with_extension("a.partial");
    fs::write(&tmp, &bytes)
        .and_then(|()| fs::rename(&tmp, lib))
        .unwrap_or_else(|e| fail(&format!("{}: {e}", lib.display())));
}

/// The bytes of the one archive member whose path ends with `suffix`.
fn member(archive: &mut zip::ZipArchive<Cursor<&[u8]>>, suffix: &str) -> Vec<u8> {
    let name = archive
        .file_names()
        .filter_map(Result::ok)
        .find(|n| n.ends_with(suffix))
        .map_or_else(
            || fail(&format!("Z3 archive has no *{suffix}")),
            Cow::into_owned,
        );
    let mut bytes = Vec::new();
    archive
        .by_name(&name)
        .and_then(|mut f| f.read_to_end(&mut bytes).map_err(Into::into))
        .unwrap_or_else(|e| fail(&format!("extracting {name}: {e}")));
    bytes
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

fn fail(msg: &str) -> ! {
    eprintln!("litigate-smt build: {msg}");
    std::process::exit(1)
}
