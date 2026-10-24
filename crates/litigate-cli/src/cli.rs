//! Command-line grammar (SPEC §9.3). Every subcommand produces one envelope.
use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};
use litigate_base::Profile;

/// `litigate`: check, compile, and query legal procedure modules.
#[derive(Debug, Parser)]
#[command(
    name = "litigate",
    version,
    about = "Check, compile, and query litigate rule modules (.lit) and matters (.case)"
)]
pub struct Cli {
    /// Output format (default: `human` on a terminal, else `json`).
    #[arg(long, global = true, value_enum)]
    pub format: Option<Format>,
    /// The operation.
    #[command(subcommand)]
    pub command: Command,
}

/// Output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// The JSON envelope on one line.
    Json,
    /// Diagnostics rendered with source excerpts, then the result.
    Human,
}

/// Checking profile (SPEC §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum ProfileArg {
    /// Holes and missing tests are warnings.
    #[default]
    Dev,
    /// Holes, missing tests, unresolved cites, and non-canonical format are errors.
    Strict,
}

impl From<ProfileArg> for Profile {
    fn from(value: ProfileArg) -> Self {
        match value {
            ProfileArg::Dev => Self::Dev,
            ProfileArg::Strict => Self::Strict,
        }
    }
}

/// Where packages named by a `.case` (or searched by id) are found.
#[derive(Debug, Clone, Default, Args)]
pub struct Library {
    /// Library directory searched for packages (repeatable; default `lit`).
    #[arg(long = "lib", value_name = "DIR")]
    pub lib: Vec<String>,
}

/// Build dates (SPEC §8.1).
#[derive(Debug, Clone, Default, Args)]
pub struct Dates {
    /// Select the law in force on this date (default: each package's `known_at`).
    #[arg(long, value_name = "DATE")]
    pub as_of: Option<String>,
    /// Select the versions recorded by this date (default: each package's `known_at`).
    #[arg(long, value_name = "DATE")]
    pub known_at: Option<String>,
}

impl Dates {
    /// The lowering options these dates select at.
    #[must_use]
    pub fn options(&self) -> litigate_lower::Options {
        litigate_lower::Options {
            as_of: self.as_of.clone(),
            known_at: self.known_at.clone(),
        }
    }
}

/// Structured query kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum QueryKind {
    /// Graph nodes.
    Nodes,
    /// Graph edges.
    Edges,
    /// Deadline declarations.
    Deadlines,
    /// Rule declarations (`rule duty|power|liability`).
    Rules,
    /// Explicit holes.
    Holes,
    /// Every cited declaration.
    Cites,
    /// Every versioned declaration with its in-force interval.
    Versions,
    /// Declarations that override another (`override`, `notwithstanding`, `supersedes`).
    Overrides,
}

/// Subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// The whole language and CLI contract as JSON.
    Describe,
    /// Write (or with `--locked`, verify) a package's `Lit.lock`, or the one
    /// beside each directory of `.case` matters, pinning what they `uses`.
    Lock {
        /// Package directory, or a directory of matters (default `.`).
        dir: Option<String>,
        /// Fail with L1007 instead of writing when the lock is stale.
        #[arg(long)]
        locked: bool,
        /// Verify as `--locked` does and print the lock's Merkle root,
        /// computed on read (SPEC §4.2); releases record it.
        #[arg(long)]
        root: bool,
    },
    /// Check packages and matters, discharging the verification obligations.
    Check {
        /// Package directories, `.lit` / `.case` files, or directories to search.
        #[arg(required = true)]
        paths: Vec<String>,
        /// Checking profile.
        #[arg(long, value_enum, default_value_t)]
        profile: ProfileArg,
        /// Render diagnostics from dependency packages individually.
        #[arg(long)]
        verbose: bool,
        /// Shared options.
        #[command(flatten)]
        dates: Dates,
        /// Shared options.
        #[command(flatten)]
        library: Library,
    },
    /// Check, then write litigatables and lowered pack/links/scenario JSON.
    Build {
        /// Package directories or `.case` files.
        #[arg(required = true)]
        paths: Vec<String>,
        /// Output directory.
        #[arg(long, value_name = "DIR", default_value = "build")]
        out: String,
        /// Write pack `ID` as `packs/STEM.json` (and its clock fragment as
        /// `packs/clock/STEM.json`) instead of `packs/ID.json` (repeatable).
        #[arg(long = "pack-file", value_name = "ID=STEM")]
        pack_files: Vec<String>,
        /// Checking profile.
        #[arg(long, value_enum, default_value_t)]
        profile: ProfileArg,
        /// Shared options.
        #[command(flatten)]
        dates: Dates,
        /// Shared options.
        #[command(flatten)]
        library: Library,
    },
    /// Run inline `test` blocks.
    Test {
        /// Package directories or files.
        #[arg(required = true)]
        paths: Vec<String>,
        /// Only tests whose name contains this text.
        #[arg(long, value_name = "TEXT")]
        filter: Option<String>,
        /// Shared options.
        #[command(flatten)]
        library: Library,
    },
    /// Format `.lit` / `.case` files canonically.
    Fmt {
        /// Files or directories.
        #[arg(required = true)]
        paths: Vec<String>,
        /// Report L9004 instead of writing.
        #[arg(long)]
        check: bool,
    },
    /// Compile a matter and run a litgraph operation on it.
    Run {
        /// The `.case` file.
        case: String,
        /// The litgraph op: a name (`chain`) or a JSON object (`{"op":"solve"}`).
        #[arg(long, default_value = "chain")]
        op: String,
        /// Do not compose links between the used packages (litgraph's
        /// request `links: false`).
        #[arg(long)]
        no_links: bool,
        /// Shared options.
        #[command(flatten)]
        library: Library,
    },
    /// Export a matter's computed deadlines as an iCalendar (`.ics`) file.
    Calendar {
        /// The `.case` file.
        case: String,
        /// Also write the calendar to this file.
        #[arg(long, value_name = "FILE")]
        out: Option<String>,
        /// Every deadline the events created, not only the open ones.
        #[arg(long)]
        all: bool,
        /// Remind this many days before each due date (repeatable).
        #[arg(long, value_name = "DAYS")]
        remind: Vec<u32>,
        /// Shared options.
        #[command(flatten)]
        library: Library,
    },
    /// Replay a matter with a proposed filing added: is it a legal, timely
    /// move, what does it discharge, and which deadlines does it trigger?
    #[command(group(ArgGroup::new("party").required(true).args(["by", "on"])))]
    LintFiling {
        /// The `.case` file.
        case: String,
        /// The event as the matter would record it, e.g. `Answered(complaint)`.
        event: String,
        /// The filing date.
        #[arg(long, value_name = "DATE")]
        date: String,
        /// The party making the filing.
        #[arg(long, value_name = "PARTY")]
        by: Option<String>,
        /// The party served (for a service event).
        #[arg(long, value_name = "PARTY")]
        on: Option<String>,
        /// Shared options.
        #[command(flatten)]
        library: Library,
    },
    /// Resolve an id at any layer: node, edge, declaration, diagnostic code, or cite.
    Where {
        /// `pack::node`, `node`, `decl@YYYY-MM-DD`, `L0412`, or cite text.
        id: String,
        /// Packages to search (default: the library).
        #[arg(long = "path", value_name = "PATH")]
        paths: Vec<String>,
        /// Shared options.
        #[command(flatten)]
        library: Library,
    },
    /// The long explanation of a diagnostic code.
    Explain {
        /// A code such as `L3004`.
        code: String,
    },
    /// Trace a computed deadline or edge value back to its rules and sources.
    ExplainValue {
        /// The `.case` file (or `--case`).
        case: Option<String>,
        /// A deadline or edge id (or `--value`).
        value: Option<String>,
        /// The `.case` file.
        #[arg(long = "case", value_name = "FILE", conflicts_with = "case")]
        case_flag: Option<String>,
        /// A deadline or edge id.
        #[arg(long = "value", value_name = "ID", conflicts_with = "value")]
        value_flag: Option<String>,
        /// Shared options.
        #[command(flatten)]
        library: Library,
    },
    /// Semantic diff (SPEC §8.6) of documents, packages, directories, or dates.
    Diff {
        /// JSON file, package directory, `.case`, or directory of JSON.
        a: String,
        /// The other side (default: `a` again, at `--to-as-of`).
        b: Option<String>,
        /// Compare `a`, built, against this expected JSON; differences are L6002.
        #[arg(long, value_name = "JSON", conflicts_with = "b")]
        against: Option<String>,
        /// Build date for both sides (or the first, with `--to-as-of`).
        #[arg(long, value_name = "DATE")]
        as_of: Option<String>,
        /// Build date for the second side.
        #[arg(long, value_name = "DATE")]
        to_as_of: Option<String>,
        /// Report differences as L6002 errors (exit 2).
        #[arg(long)]
        check: bool,
        /// Shared options.
        #[command(flatten)]
        library: Library,
    },
    /// The computed compatibility class (`interface`, `value`, `none`) of
    /// package release `NEW` against earlier release `OLD`, and whether its
    /// version bump is enough (L1012 otherwise).
    Compat {
        /// The earlier release's package directory.
        old: String,
        /// The new release's package directory.
        new: String,
    },
    /// Structured queries over the library.
    Query {
        /// What to list.
        #[arg(value_enum)]
        kind: QueryKind,
        /// Only ids or labels containing this text.
        #[arg(long = "match", value_name = "TEXT")]
        text: Option<String>,
        /// Only declarations whose cite contains this text.
        #[arg(long, value_name = "TEXT")]
        cite: Option<String>,
        /// Only versions in force on this date.
        #[arg(long, value_name = "DATE")]
        effective_on: Option<String>,
        /// Only declarations that override (or relate to) this id.
        #[arg(long, value_name = "ID")]
        overrides: Option<String>,
        /// Only this package (name or pack id).
        #[arg(long, value_name = "NAME")]
        package: Option<String>,
        /// Maximum matches.
        #[arg(long, default_value_t = 20)]
        limit: usize,
        /// Resume after a previous `next_cursor`.
        #[arg(long)]
        cursor: Option<String>,
        /// Packages to search (default: the library).
        #[arg(long = "path", value_name = "PATH")]
        paths: Vec<String>,
        /// Shared options.
        #[command(flatten)]
        library: Library,
    },
    /// The timeline of a `.case` or `.lit` file as JSON (`litigate.timeline/1`):
    /// rows, worlds, layer provenance, status with reason.
    Timeline {
        /// The `.case` or `.lit` file.
        file: String,
        /// Judge statuses at `--as-of` (a matter's own `as_of` by default)
        /// and select the versions recorded by `--known-at`.
        #[command(flatten)]
        dates: Dates,
        /// Layers to enable (`ID`, `+ID`: only those named) or disable
        /// (`-ID`), comma-separated or repeated.
        #[arg(
            long,
            value_name = "ID",
            value_delimiter = ',',
            allow_hyphen_values = true
        )]
        layers: Vec<String>,
        /// Only this world (e.g. `w1`) and its ancestors up to its fork.
        #[arg(long, value_name = "ID")]
        world: Option<String>,
        /// Shared options.
        #[command(flatten)]
        library: Library,
    },
    /// Turn legacy pack / links / scenario JSON into `.lit` / `.case` source.
    Import {
        /// JSON files.
        #[arg(required = true)]
        inputs: Vec<String>,
        /// Output root (`lit/<pkg>/...`, `cases/<id>.case` under it).
        #[arg(long, value_name = "DIR", default_value = ".")]
        out: String,
    },
    /// Apply machine-applicable fixes from `check`.
    Fix {
        /// Files or package directories.
        #[arg(required = true)]
        paths: Vec<String>,
        /// Only fixes of diagnostics with this code.
        #[arg(long)]
        code: Option<String>,
        /// Report the edits without writing.
        #[arg(long)]
        dry_run: bool,
        /// Also apply `maybe_incorrect` fixes.
        #[arg(long)]
        include_maybe: bool,
        /// Checking profile used to find fixes.
        #[arg(long, value_enum, default_value_t)]
        profile: ProfileArg,
        /// Shared options.
        #[command(flatten)]
        library: Library,
    },
}

impl Command {
    /// The envelope `op` name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Describe => "describe",
            Self::Lock { .. } => "lock",
            Self::Check { .. } => "check",
            Self::Build { .. } => "build",
            Self::Test { .. } => "test",
            Self::Fmt { .. } => "fmt",
            Self::Run { .. } => "run",
            Self::Calendar { .. } => "calendar",
            Self::LintFiling { .. } => "lint-filing",
            Self::Where { .. } => "where",
            Self::Explain { .. } => "explain",
            Self::ExplainValue { .. } => "explain-value",
            Self::Diff { .. } => "diff",
            Self::Compat { .. } => "compat",
            Self::Query { .. } => "query",
            Self::Timeline { .. } => "timeline",
            Self::Import { .. } => "import",
            Self::Fix { .. } => "fix",
        }
    }
}
