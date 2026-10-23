//! The workspace module graph: module names, import resolution across
//! packages (SPEC §4.3), and import-cycle breaking (L1003). It depends only
//! on module headers and manifests, so editing a module body never
//! invalidates it.
use crate::db::inputs::{SourceFile, Workspace};
use crate::db::queries::{manifest, module_header};
use crate::resolve::suggest;
use std::collections::{BTreeSet, HashMap};

/// Where one `import` points.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportTarget {
    /// A module (possibly in a dependency package).
    Module(SourceFile),
    /// No such module; nearest visible module name, if close.
    Unknown(Option<String>),
    /// The module exists in a package that is not a declared dependency.
    Undeclared(String),
    /// This import closes a cycle (module names along the cycle).
    Cycle(Vec<String>),
}

/// One module file in the graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileNode {
    /// Module name.
    pub module: String,
    /// Another file of the same package already declares this module.
    pub duplicate: bool,
    /// Import targets in source order.
    pub imports: Vec<ImportTarget>,
}

/// Graph over every file of the workspace.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModuleGraph {
    nodes: HashMap<SourceFile, FileNode>,
}

impl ModuleGraph {
    /// The file's node.
    #[must_use]
    pub fn node(&self, file: SourceFile) -> Option<&FileNode> {
        self.nodes.get(&file)
    }

    /// Acyclic import edges of `file` (cycle-closing imports removed).
    #[must_use]
    pub fn imports(&self, file: SourceFile) -> Vec<SourceFile> {
        self.node(file)
            .map(|n| {
                n.imports
                    .iter()
                    .filter_map(|t| match t {
                        ImportTarget::Module(f) => Some(*f),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Builds the graph.
pub fn build(db: &dyn salsa::Database, ws: Workspace) -> ModuleGraph {
    let packages = ws.packages(db);
    let info: Vec<_> = packages
        .iter()
        .map(|p| {
            let m = manifest(db, *p).as_ref().ok();
            let name = m.map(|m| m.manifest.name.clone());
            let mut visible: BTreeSet<String> = m
                .map(|m| m.manifest.dependencies.keys().cloned().collect())
                .unwrap_or_default();
            visible.insert("core".to_owned());
            (*p, name, visible)
        })
        .collect();
    let mut nodes = HashMap::new();
    let mut index: HashMap<(usize, String), SourceFile> = HashMap::new();
    for (i, (pkg, _, _)) in info.iter().enumerate() {
        for file in pkg.files(db) {
            let module = module_header(db, *file).module.clone();
            let duplicate = index.contains_key(&(i, module.clone()));
            if !duplicate {
                index.insert((i, module.clone()), *file);
            }
            nodes.insert(
                *file,
                FileNode {
                    module,
                    duplicate,
                    imports: Vec::new(),
                },
            );
        }
    }
    for (i, (pkg, _, visible)) in info.iter().enumerate() {
        for file in pkg.files(db) {
            let imports = module_header(db, *file)
                .imports
                .iter()
                .map(|path| {
                    if let Some(f) = index.get(&(i, path.clone())) {
                        return ImportTarget::Module(*f);
                    }
                    let found = info.iter().enumerate().find_map(|(j, (_, name, _))| {
                        index
                            .get(&(j, path.clone()))
                            .map(|f| (name.clone().unwrap_or_default(), *f))
                    });
                    match found {
                        Some((name, f)) if visible.contains(&name) => ImportTarget::Module(f),
                        Some((name, _)) => ImportTarget::Undeclared(name),
                        None => {
                            let names = index
                                .keys()
                                .filter(|(j, _)| {
                                    *j == i
                                        || info[*j].1.as_ref().is_some_and(|n| visible.contains(n))
                                })
                                .map(|(_, m)| m.as_str());
                            ImportTarget::Unknown(suggest::nearest(path, names).map(str::to_owned))
                        }
                    }
                })
                .collect();
            nodes
                .entry(*file)
                .and_modify(|node: &mut FileNode| node.imports = imports);
        }
    }
    let order: Vec<SourceFile> = packages
        .iter()
        .flat_map(|p| p.files(db).iter().copied())
        .collect();
    break_cycles(&mut nodes, &order);
    ModuleGraph { nodes }
}

/// Depth-first search in deterministic order; every import that reaches a
/// module still on the stack becomes [`ImportTarget::Cycle`].
fn break_cycles(nodes: &mut HashMap<SourceFile, FileNode>, order: &[SourceFile]) {
    #[derive(Clone, Copy, PartialEq)]
    enum Mark {
        Active,
        Done,
    }
    let mut marks: HashMap<SourceFile, Mark> = HashMap::new();
    for root in order {
        if marks.contains_key(root) {
            continue;
        }
        let mut stack: Vec<(SourceFile, usize)> = vec![(*root, 0)];
        marks.insert(*root, Mark::Active);
        while let Some(top) = stack.last_mut() {
            let (file, next) = *top;
            let target = nodes.get(&file).and_then(|n| n.imports.get(next)).cloned();
            let Some(target) = target else {
                marks.insert(file, Mark::Done);
                stack.pop();
                continue;
            };
            top.1 += 1;
            let ImportTarget::Module(dep) = target else {
                continue;
            };
            match marks.get(&dep) {
                Some(Mark::Done) => {}
                Some(Mark::Active) => {
                    let start = stack.iter().position(|(f, _)| *f == dep).unwrap_or(0);
                    let mut cycle: Vec<String> = stack[start..]
                        .iter()
                        .filter_map(|(f, _)| nodes.get(f).map(|n| n.module.clone()))
                        .collect();
                    cycle.push(
                        nodes
                            .get(&dep)
                            .map(|n| n.module.clone())
                            .unwrap_or_default(),
                    );
                    nodes
                        .entry(file)
                        .and_modify(|n| n.imports[next] = ImportTarget::Cycle(cycle));
                }
                None => {
                    marks.insert(dep, Mark::Active);
                    stack.push((dep, 0));
                }
            }
        }
    }
}
