//! One module per subcommand; [`dispatch`] routes the parsed command.
mod build;
mod calendar;
pub(crate) mod check;
mod compat;
mod diff;
mod explain;
mod filing;
mod fix;
mod fmt;
mod import;
mod lock;
mod query;
pub(crate) mod run;
mod test;
mod timeline;
mod value;
mod where_;

pub use explain::phase;

use crate::cli::Command;
use crate::out::Outcome;
use litigate_sema::pkg::source::Source;

/// Runs `command`; `check` and `build` read (and write) `src`, every other
/// command the disk.
// One arm per subcommand, each only unpacking its arguments.
#[allow(clippy::too_many_lines)]
pub fn dispatch(command: Command, src: &dyn Source) -> Outcome {
    match command {
        Command::Describe => Ok(crate::out::Report::of(crate::describe::describe())),
        Command::Lock { dir, locked, root } => lock::lock(dir.as_deref(), locked, root),
        Command::Check {
            paths,
            profile,
            verbose,
            dates,
            library,
        } => check::check(src, &paths, profile.into(), &dates, &library.lib, verbose),
        Command::Build {
            paths,
            out,
            pack_files,
            profile,
            dates,
            library,
        } => build::build(
            src,
            &build::Args {
                paths,
                out,
                pack_files,
                profile: profile.into(),
                dates,
                lib: library.lib,
            },
        ),
        Command::Test {
            paths,
            filter,
            library,
        } => test::test(&paths, filter.as_deref(), &library.lib),
        Command::Fmt { paths, check } => fmt::fmt(&paths, check),
        Command::Run { case, op, flags } => run::run(&case, &op, &flags),
        Command::Op(op) => {
            let (name, args) = op.parts();
            run::run(&args.target, name, &args.flags)
        }
        // `execute` runs the engine's own interface before dispatching.
        Command::Engine { .. } => Err(crate::out::Failure::new(
            "engine prints its own response; run it through `execute`",
        )),
        Command::Calendar {
            case,
            out,
            all,
            remind,
            library,
        } => calendar::calendar(&calendar::Args {
            case,
            out,
            all,
            remind,
            lib: library.lib,
        }),
        Command::LintFiling {
            case,
            event,
            date,
            by,
            on,
            library,
        } => {
            let (mode, party) = match (by, on) {
                (Some(party), _) => ("by", party),
                (None, party) => ("on", party.unwrap_or_default()),
            };
            filing::lint_filing(&filing::Args {
                case,
                event,
                date,
                mode,
                party,
                lib: library.lib,
            })
        }
        Command::Where { id, paths, library } => where_::where_(&id, &paths, &library.lib),
        Command::Explain { code, flags } if std::path::Path::new(&code).exists() => {
            run::run(&code, "explain", &flags)
        }
        Command::Explain { code, .. } => explain::explain(&code),
        Command::ExplainValue {
            case,
            value,
            case_flag,
            value_flag,
            library,
        } => value::explain_value(
            case.or(case_flag).as_deref(),
            value.or(value_flag).as_deref(),
            &library.lib,
        ),
        Command::Diff {
            a,
            b,
            against,
            as_of,
            to_as_of,
            check,
            library,
        } => diff::diff(&diff::Args {
            a,
            b,
            against,
            as_of,
            to_as_of,
            check,
            lib: library.lib,
        }),
        Command::Compat { old, new } => compat::compat(&old, &new),
        Command::Query {
            kind,
            text,
            cite,
            effective_on,
            overrides,
            package,
            limit,
            cursor,
            paths,
            library,
        } => query::query(&query::Args {
            kind,
            text,
            cite,
            effective_on,
            overrides,
            package,
            limit,
            cursor,
            paths,
            lib: library.lib,
        }),
        Command::Timeline {
            file,
            dates,
            layers,
            world,
            library,
        } => timeline::timeline(&timeline::Args {
            file,
            dates,
            layers,
            world,
            lib: library.lib,
        }),
        Command::Import { inputs, out } => import::import(&inputs, &out),
        Command::Fix {
            paths,
            code,
            dry_run,
            include_maybe,
            profile,
            library,
        } => fix::fix(&fix::Args {
            paths,
            code,
            dry_run,
            include_maybe,
            profile: profile.into(),
            lib: library.lib,
        }),
    }
}
