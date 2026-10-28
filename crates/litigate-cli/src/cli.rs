//! Command-line grammar (SPEC §9.3). Every subcommand produces one envelope.
use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};
use litigate_base::Profile;

/// `litigate`: check, compile, and query legal procedure modules.
#[derive(Debug, Parser)]
#[command(
    name = "litigate",
    version = crate::VERSION,
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

/// The date deadlines are judged at (lg-rpmf).
#[derive(Debug, Clone, Default, Args)]
pub struct Today {
    /// Judge deadlines at this date: when a matter's `as_of` is older,
    /// one past due by then with no act lapses, with an L4016 warning
    /// (default: `LITIGATE_TODAY`, else the system date, unless `--as-of`
    /// is given; `--today` with the matter's `as_of` reproduces it).
    #[arg(long, value_name = "DATE")]
    pub today: Option<String>,
}

/// How an engine op runs on a compiled matter or package.
#[derive(Debug, Clone, Default, Args)]
pub struct OpFlags {
    /// Op argument, e.g. `--arg node=answer-due --arg top=5` (repeatable;
    /// numbers, `true`/`false` and JSON are typed, anything else a string).
    #[arg(long = "arg", value_name = "K=V")]
    pub args: Vec<String>,
    /// Scenario parameter, e.g. `--set rate=900` (repeatable, numeric):
    /// overrides the matter's `analysis { params }`.
    #[arg(long = "set", value_name = "K=V")]
    pub set: Vec<String>,
    /// Do not compose links between the used packages (the engine's
    /// request `links: false`).
    #[arg(long)]
    pub no_links: bool,
    /// Shared options.
    #[command(flatten)]
    pub library: Library,
}

/// An engine op's target and flags.
#[derive(Debug, Clone, Args)]
pub struct OpArgs {
    /// A `.case` matter (its scenario), or a package (`.lit` file or
    /// package directory: its pack under the default scenario).
    pub target: String,
    /// Shared options.
    #[command(flatten)]
    pub flags: OpFlags,
}

/// The engine ops (the `describe` op documents their arguments),
/// each run on a compiled matter or package. `explain` is the
/// `explain` command given a path instead of a diagnostic code.
#[derive(Debug, Subcommand)]
pub enum EngineOp {
    /// Value and best line under the optimal policy.
    Solve(OpArgs),
    /// Absorption probabilities and expected totals under the optimal policy.
    Chain(OpArgs),
    /// Monte Carlo outcome distribution, tail risk, and the chance of a loss.
    Simulate(OpArgs),
    /// Shortest / k-shortest lines to a node, the terminals, or an outcome.
    Path(OpArgs),
    /// The N-objective Pareto frontier.
    Pareto(OpArgs),
    /// Value curve and policy breakpoints over one parameter (`--arg param=rate`).
    Sweep(OpArgs),
    /// What the answer is most sensitive to.
    Tornado(OpArgs),
    /// Uncalibrated probabilities and durations ranked by decision sensitivity.
    Calibration(OpArgs),
    /// Credible intervals over every chance node's Dirichlet posterior.
    Posterior(OpArgs),
    /// Value of information: EVPI per chance node, EVSI per study.
    Voi(OpArgs),
    /// Graph structure: summary, scc, dominators, mincut, betweenness, reachability.
    Structure(OpArgs),
    /// Concrete due dates and computation steps for authored deadline specs.
    Deadlines(OpArgs),
    /// Each side's walk-away value, the ZOPA, and when to settle.
    Settle(OpArgs),
    /// Base against a variant scenario (`--arg variant={..} --arg inner={..}`).
    Compare(OpArgs),
    /// The graph's nodes and edges (a neighborhood with `--arg node=ID`).
    Graph(OpArgs),
    /// A metric evaluated on every edge (`--arg spec=...`).
    Metric(OpArgs),
    /// The engine's content diagnostics.
    Lint(OpArgs),
}

impl EngineOp {
    /// The engine op name and its arguments.
    #[must_use]
    pub fn parts(&self) -> (&'static str, &OpArgs) {
        match self {
            Self::Solve(a) => ("solve", a),
            Self::Chain(a) => ("chain", a),
            Self::Simulate(a) => ("simulate", a),
            Self::Path(a) => ("path", a),
            Self::Pareto(a) => ("pareto", a),
            Self::Sweep(a) => ("sweep", a),
            Self::Tornado(a) => ("tornado", a),
            Self::Calibration(a) => ("calibration", a),
            Self::Posterior(a) => ("posterior", a),
            Self::Voi(a) => ("voi", a),
            Self::Structure(a) => ("structure", a),
            Self::Deadlines(a) => ("deadlines", a),
            Self::Settle(a) => ("settle", a),
            Self::Compare(a) => ("compare", a),
            Self::Graph(a) => ("graph", a),
            Self::Metric(a) => ("metric", a),
            Self::Lint(a) => ("lint", a),
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

/// `lint-filing`'s paper: its counts, the certificates it carries, and
/// its kind for an event that declares none.
#[derive(Debug, Clone, Default, Args)]
pub struct PaperFlags {
    /// The paper's word count, without the parts its limit excludes.
    #[arg(long, value_name = "N")]
    pub words: Option<u64>,
    /// The paper's page count, without the parts its limit excludes.
    #[arg(long, value_name = "N")]
    pub pages: Option<u64>,
    /// The paper's lines of monospaced type, without the excluded parts.
    #[arg(long, value_name = "N")]
    pub lines: Option<u64>,
    /// The paper as UTF-8 text: its words are counted, excluded parts
    /// included (instead of --words).
    #[arg(long, value_name = "FILE")]
    pub document: Option<String>,
    /// The paper kind (a `limit`), only for an event that declares none.
    #[arg(long, value_name = "KIND")]
    pub paper: Option<String>,
    /// A certificate the paper carries (repeatable).
    #[arg(long, value_name = "KIND")]
    pub certificate: Vec<String>,
}

/// The subcommands that print one envelope.
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
        today: Today,
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
    /// Run inline `test` blocks and the expect tests under a path
    /// (`tests/expect/<case>/`, failing on a value that moved), or
    /// (`--impact BASE`) the expect tests of `tests/expect` at BASE and in
    /// the working tree, listing every deadline, verdict and expectation
    /// that moved.
    Test {
        /// Package directories or files, or directories of expect tests.
        #[arg(required_unless_present = "impact", conflicts_with = "impact")]
        paths: Vec<String>,
        /// Only tests whose name contains this text.
        #[arg(long, value_name = "TEXT", conflicts_with = "impact")]
        filter: Option<String>,
        /// Compare `tests/expect` at this git revision with the working
        /// tree (run from the repository root).
        #[arg(long, value_name = "BASE")]
        impact: Option<String>,
        /// Record the values each expect test (`tests/expect/<case>/`) finds
        /// in its `expect.toml`, in place of those it had; say why in the
        /// commit (SPEC §5.8, expect tests).
        #[arg(long, conflicts_with = "impact")]
        bless: bool,
        /// Shared options.
        #[command(flatten)]
        library: Library,
    },
    /// Write `docs/COVERAGE.md`, the per-jurisdiction coverage table,
    /// from a repository's `lit/`, `outlines/` and `tests/expect/`.
    Coverage {
        /// Repository root (default `.`).
        root: Option<String>,
        /// Exit 2 instead of writing when the committed table is stale.
        #[arg(long)]
        check: bool,
    },
    /// Format `.lit` / `.case` files canonically.
    Fmt {
        /// Files or directories.
        #[arg(required_unless_present = "stdin", conflicts_with = "stdin")]
        paths: Vec<String>,
        /// Report L9004 instead of writing.
        #[arg(long, conflicts_with = "stdin")]
        check: bool,
        /// Format standard input (a `.lit` module or a `.case` matter, told
        /// apart by parsing) to standard output, with no envelope: the
        /// editor formatter protocol. A parse error prints its diagnostics
        /// to standard error and exits 2 with nothing on standard output.
        #[arg(long)]
        stdin: bool,
    },
    /// Compile a matter or package and run an engine op on it.
    Run {
        /// The `.case` file, `.lit` file, or package directory.
        case: String,
        /// The engine op: a name (`chain`) or a JSON object (`{"op":"solve"}`).
        #[arg(long, default_value = "chain")]
        op: String,
        /// Shared options.
        #[command(flatten)]
        flags: OpFlags,
    },
    /// An engine op on a compiled matter or package.
    #[command(flatten)]
    Op(EngineOp),
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
        /// What the filer says of the paper, checked against its limit.
        #[command(flatten)]
        paper: Box<PaperFlags>,
        /// Shared options.
        #[command(flatten)]
        library: Library,
    },
    /// The moves a party may legally make in a matter's posture: each
    /// edge out of the replayed marking it may take, kept only when replay
    /// accepts it on the date.
    Actions {
        /// The `.case` file.
        case: String,
        /// The party whose moves are listed.
        #[arg(long, value_name = "PARTY")]
        party: String,
        /// The date the moves would be made (default: the matter's `as_of`).
        #[arg(long, value_name = "DATE")]
        date: Option<String>,
        /// Shared options.
        #[command(flatten)]
        library: Library,
    },
    /// Match PACER/RECAP docket entries to declared event forms and advance
    /// the matter only by the entries the text supports and replay accepts.
    MatchDocket {
        /// The `.case` file.
        case: String,
        /// The docket: a JSON array of entries (`entry_number`, `date_filed`,
        /// `description`), or an object holding one under `docket_entries` or `results`.
        docket: String,
        /// Write the advanced matter to this file.
        #[arg(long, value_name = "FILE")]
        out: Option<String>,
        /// Shared options.
        #[command(flatten)]
        library: Library,
    },
    /// Derive a `calibration` declaration from replayed, sourced docket histories:
    /// chance odds and edge durations observed in the matters' docketed
    /// events, each estimate with its provenance and selection criteria.
    CalibrateReplay {
        /// The `.case` files, or directories of them: each a docket history
        /// citing its docket in a `source "Docket ..."` with a url.
        #[arg(required = true)]
        cases: Vec<String>,
        /// The calibration set's id.
        #[arg(long)]
        id: String,
        /// The calibration set's title.
        #[arg(long)]
        title: Option<String>,
        /// Withhold an estimate resting on fewer observations.
        #[arg(long, value_name = "N", default_value_t = 1)]
        min_n: u64,
        /// Attest that every history's docket matches were reviewed against
        /// the docket; required to write the set.
        #[arg(long)]
        reviewed: bool,
        /// Write the set's `calibration` declaration to this `.lit` file,
        /// in the package it calibrates (needs `--reviewed`).
        #[arg(long, value_name = "FILE")]
        out: Option<String>,
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
    /// The long explanation of a diagnostic code; given a `.case`,
    /// `.lit`, or package directory instead, the engine's `explain` op on
    /// it (who decides, every option with its value and what happens next).
    Explain {
        /// A code such as `L3004`, or a matter or package path.
        code: String,
        /// Shared options (a path only).
        #[command(flatten)]
        flags: OpFlags,
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
        /// Judge deadlines at `--today`.
        #[command(flatten)]
        today: Today,
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
    /// Alternate timelines: replay a matter in every world a `.world` file
    /// spans (its layers, one option per choice point) at `--as-of` and
    /// `--known-at`, and report which filings are timely in which worlds.
    Worlds {
        /// The `.case` file.
        case: String,
        /// The `.world` file: layers, location, modality, choice points.
        #[arg(long, value_name = "FILE")]
        world: String,
        /// Package directories (or directories searched for packages) to
        /// load besides the library (repeatable).
        #[arg(long = "path", value_name = "PATH")]
        paths: Vec<String>,
        /// Add this layer to every world (repeatable).
        #[arg(long = "layer", value_name = "NAME")]
        layers: Vec<String>,
        /// Make this layer a choice point: every world with and without it
        /// (repeatable).
        #[arg(long, value_name = "NAME")]
        vary: Vec<String>,
        /// Judge the matter at this date instead of its `as_of`.
        #[arg(long, value_name = "DATE")]
        as_of: Option<String>,
        /// Select the versions recorded by this date instead of the
        /// matter's `known_at`.
        #[arg(long, value_name = "DATE")]
        known_at: Option<String>,
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
    /// Gives `check` and `timeline` `today` as their `--today` when they
    /// have neither `--today` nor `--as-of` (which reproduces history).
    pub fn default_today(&mut self, today: &str) {
        if let Self::Check {
            dates, today: t, ..
        }
        | Self::Timeline {
            dates, today: t, ..
        } = self
        {
            if t.today.is_none() && dates.as_of.is_none() {
                t.today = Some(today.to_owned());
            }
        }
    }

    /// The envelope `op` name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Describe => "describe",
            Self::Lock { .. } => "lock",
            Self::Check { .. } => "check",
            Self::Build { .. } => "build",
            Self::Test { .. } => "test",
            Self::Coverage { .. } => "coverage",
            Self::Fmt { .. } => "fmt",
            Self::Run { .. } => "run",
            Self::Op(op) => op.parts().0,
            Self::Calendar { .. } => "calendar",
            Self::LintFiling { .. } => "lint-filing",
            Self::Actions { .. } => "actions",
            Self::MatchDocket { .. } => "match-docket",
            Self::CalibrateReplay { .. } => "calibrate-replay",
            Self::Where { .. } => "where",
            Self::Explain { .. } => "explain",
            Self::ExplainValue { .. } => "explain-value",
            Self::Diff { .. } => "diff",
            Self::Compat { .. } => "compat",
            Self::Query { .. } => "query",
            Self::Timeline { .. } => "timeline",
            Self::Worlds { .. } => "worlds",
            Self::Import { .. } => "import",
            Self::Fix { .. } => "fix",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_op_subcommand_runs_the_engine_op_it_is_named_for() {
        let ops = EngineOp::augment_subcommands(clap::Command::new("ops"));
        let names: Vec<&str> = ops.get_subcommands().map(clap::Command::get_name).collect();
        assert_eq!(names.len(), 17);
        // The envelope name, engine op and target `argv` parses to, if an op.
        let op = |argv: &[&str]| match &Cli::try_parse_from(argv).map(|c| c.command) {
            Ok(command @ Command::Op(op)) => {
                Some((command.name(), op.parts().0, op.parts().1.target.clone()))
            }
            _ => None,
        };
        for name in names {
            assert_eq!(
                op(&["litigate", name, "x.case"]),
                Some((name, name, "x.case".to_owned()))
            );
        }
        assert_eq!(op(&["litigate", "check", "x.case"]), None);
    }
}
