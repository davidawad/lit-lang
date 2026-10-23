//! Salsa inputs. Everything else is derived by tracked queries.
use crate::pkg::load::LoadedPackage;

/// The set of packages under analysis.
#[salsa::input(debug)]
pub struct Workspace {
    /// Packages in load order (roots first).
    #[returns(ref)]
    pub packages: Vec<PackageInput>,
}

/// One package directory.
#[salsa::input(debug)]
pub struct PackageInput {
    /// Repository-relative directory.
    #[returns(ref)]
    pub root: String,
    /// `Lit.toml` text.
    #[returns(ref)]
    pub manifest_text: String,
    /// `Lit.lock` text, if present.
    #[returns(ref)]
    pub lock_text: Option<String>,
    /// Source files.
    #[returns(ref)]
    pub files: Vec<SourceFile>,
    /// Vendored L0 text digests by repo-relative path
    /// ([`LoadedPackage::texts`]).
    #[returns(ref)]
    pub texts: std::collections::BTreeMap<String, Option<String>>,
}

/// One `.lit` source file.
#[salsa::input(debug)]
pub struct SourceFile {
    /// Package-relative POSIX path.
    #[returns(ref)]
    pub relative: String,
    /// Contents.
    #[returns(ref)]
    pub text: String,
    /// Owning package.
    #[returns(copy)]
    pub package: PackageInput,
    /// Owning workspace.
    #[returns(copy)]
    pub workspace: Workspace,
}

impl PackageInput {
    /// Repository-relative path of `Lit.toml`.
    pub fn manifest_path(self, db: &dyn salsa::Database) -> String {
        join(self.root(db), "Lit.toml")
    }
}

impl SourceFile {
    /// Repository-relative path (the diagnostic `FileId`).
    pub fn path(self, db: &dyn salsa::Database) -> String {
        join(self.package(db).root(db), self.relative(db))
    }
}

fn join(root: &str, relative: &str) -> String {
    if root == "." || root.is_empty() {
        relative.to_owned()
    } else {
        format!("{root}/{relative}")
    }
}

/// Creates the inputs for `packages` in a fresh workspace.
pub fn create_workspace(db: &mut dyn salsa::Database, packages: &[LoadedPackage]) -> Workspace {
    use salsa::Setter;
    let ws = Workspace::new(db, Vec::new());
    let inputs = packages
        .iter()
        .map(|p| {
            let pkg = PackageInput::new(
                db,
                p.root.clone(),
                p.manifest.clone(),
                p.lock.clone(),
                Vec::new(),
                p.texts.clone(),
            );
            let files = p
                .files
                .iter()
                .map(|f| SourceFile::new(db, f.relative.clone(), f.text.clone(), pkg, ws))
                .collect();
            pkg.set_files(db).to(files);
            pkg
        })
        .collect();
    ws.set_packages(db).to(inputs);
    ws
}
