//! Where package trees and matters are read from, and build outputs
//! written to: the real filesystem ([`Disk`], the CLI's) or an in-memory
//! file tree ([`MemoryTree`], for hosts with no filesystem such as
//! WebAssembly). This is the only module in sema that touches the
//! filesystem.
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Component, Path, PathBuf};

/// A file tree. Relative paths resolve against [`Source::root`].
pub trait Source {
    /// The directory relative paths resolve against.
    ///
    /// # Errors
    /// The working directory cannot be determined.
    fn root(&self) -> io::Result<PathBuf>;
    /// A file's bytes.
    ///
    /// # Errors
    /// A missing or unreadable file.
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
    /// A file's text.
    ///
    /// # Errors
    /// A missing or unreadable file, or one that is not UTF-8.
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
    /// The entries of a directory, each `dir.join(name)`, in no set order.
    ///
    /// # Errors
    /// A missing or unreadable directory.
    fn entries(&self, dir: &Path) -> io::Result<Vec<PathBuf>>;
    /// True for the real filesystem, where a build cache may live
    /// (SPEC §4.2); false for an in-memory tree.
    fn on_disk(&self) -> bool {
        false
    }
    /// True for an existing directory.
    fn is_dir(&self, path: &Path) -> bool;
    /// True for an existing file.
    fn is_file(&self, path: &Path) -> bool;
    /// True for an existing file or directory.
    fn exists(&self, path: &Path) -> bool;
    /// The absolute, normalized path of an existing file or directory.
    ///
    /// # Errors
    /// A path that does not exist.
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf>;
    /// Writes `text` to `path`, creating its directories; on disk through
    /// a temporary file and a rename. The error names the failing path.
    ///
    /// # Errors
    /// I/O failures.
    fn write_atomic(&self, path: &Path, text: &str) -> io::Result<()>;
}

/// `error` with `path` in front of its message, keeping its kind.
fn at(path: &Path, error: &io::Error) -> io::Error {
    io::Error::new(error.kind(), format!("{}: {error}", path.display()))
}

/// The real filesystem, relative to the working directory.
#[derive(Debug, Clone, Copy, Default)]
pub struct Disk;

impl Source for Disk {
    fn root(&self) -> io::Result<PathBuf> {
        std::env::current_dir()
    }

    fn on_disk(&self) -> bool {
        true
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        std::fs::read(path)
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        std::fs::read_to_string(path)
    }

    fn entries(&self, dir: &Path) -> io::Result<Vec<PathBuf>> {
        std::fs::read_dir(dir).and_then(|listing| listing.map(|e| e.map(|e| e.path())).collect())
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        path.canonicalize()
    }

    fn write_atomic(&self, path: &Path, text: &str) -> io::Result<()> {
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).map_err(|e| at(dir, &e))?;
        }
        let tmp = path.with_extension("litigate-tmp");
        std::fs::write(&tmp, text).map_err(|e| at(&tmp, &e))?;
        std::fs::rename(&tmp, path).map_err(|e| at(path, &e))
    }
}

/// An in-memory file tree rooted at `/`: files by POSIX path, directories
/// implied by them. Writes are collected apart from the tree (see
/// [`MemoryTree::written`]) and never read back.
#[derive(Debug, Default)]
pub struct MemoryTree {
    files: BTreeMap<String, String>,
    written: RefCell<BTreeMap<String, String>>,
}

/// `path` resolved against `/`, as `/`-joined components without the
/// leading slash (`""` for the root). `..` at the root stays at the root.
fn normalize(path: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(s) => parts.push(s.to_string_lossy().into_owned()),
            Component::ParentDir => {
                parts.pop();
            }
            Component::RootDir => parts.clear(),
            Component::CurDir | Component::Prefix(_) => {}
        }
    }
    parts.join("/")
}

fn not_found(path: &Path) -> io::Error {
    io::Error::new(
        io::ErrorKind::NotFound,
        format!("{}: no such file or directory", path.display()),
    )
}

impl MemoryTree {
    /// A tree of `(path, text)` files; paths are normalized (`./a`, `/a`
    /// and `b/../a` all name `a`), and a later duplicate wins.
    pub fn new<P: AsRef<Path>>(files: impl IntoIterator<Item = (P, String)>) -> Self {
        Self {
            files: files
                .into_iter()
                .map(|(path, text)| (normalize(path.as_ref()), text))
                .collect(),
            written: RefCell::default(),
        }
    }

    /// Every file written so far, by normalized path.
    #[must_use]
    pub fn written(&self) -> BTreeMap<String, String> {
        self.written.borrow().clone()
    }

    /// The names directly under the normalized directory `dir`.
    fn children(&self, dir: &str) -> BTreeSet<&str> {
        let prefix = if dir.is_empty() {
            String::new()
        } else {
            format!("{dir}/")
        };
        self.files
            .range(prefix.clone()..)
            .map(|(path, _)| path.as_str())
            .take_while(|path| path.starts_with(&prefix))
            .filter_map(|path| path[prefix.len()..].split('/').next())
            .collect()
    }
}

impl Source for MemoryTree {
    fn root(&self) -> io::Result<PathBuf> {
        Ok(PathBuf::from("/"))
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.read_to_string(path).map(String::into_bytes)
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        self.files
            .get(&normalize(path))
            .cloned()
            .ok_or_else(|| not_found(path))
    }

    fn entries(&self, dir: &Path) -> io::Result<Vec<PathBuf>> {
        if !self.is_dir(dir) {
            return Err(not_found(dir));
        }
        Ok(self
            .children(&normalize(dir))
            .into_iter()
            .map(|name| dir.join(name))
            .collect())
    }

    fn is_dir(&self, path: &Path) -> bool {
        let path = normalize(path);
        path.is_empty() || !self.children(&path).is_empty()
    }

    fn is_file(&self, path: &Path) -> bool {
        self.files.contains_key(&normalize(path))
    }

    fn exists(&self, path: &Path) -> bool {
        self.is_file(path) || self.is_dir(path)
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        if self.exists(path) {
            Ok(Path::new("/").join(normalize(path)))
        } else {
            Err(not_found(path))
        }
    }

    fn write_atomic(&self, path: &Path, text: &str) -> io::Result<()> {
        self.written
            .borrow_mut()
            .insert(normalize(path), text.to_owned());
        Ok(())
    }
}
