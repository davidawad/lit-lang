//! One module per subcommand; [`dispatch`] routes the parsed command.
mod build;
mod check;
mod diff;
mod explain;
mod fix;
mod fmt;
mod import;
mod lock;
mod query;
mod run;
mod test;
mod value;
mod where_;

use crate::cli::Command;
use crate::out::Outcome;

/// Runs `command`.
pub fn dispatch(command: Command) -> Outcome {
    match command {
        Command::Describe => Ok(crate::out::Report::of(crate::describe::describe())),
        Command::Lock { dir, locked } => lock::lock(dir.as_deref(), locked),
        Command::Check {
            paths,
            profile,
            dates,
            library,
        } => check::check(&paths, profile.into(), &dates, &library.lib),
        Command::Build {
            paths,
            out,
            profile,
            dates,
            library,
        } => build::build(&paths, &out, profile.into(), &dates, &library.lib),
        Command::Test {
            paths,
            filter,
            library,
        } => test::test(&paths, filter.as_deref(), &library.lib),
        Command::Fmt { paths, check } => fmt::fmt(&paths, check),
        Command::Run { case, op, library } => run::run(&case, &op, &library.lib),
        Command::Where { id, paths, library } => where_::where_(&id, &paths, &library.lib),
        Command::Explain { code } => explain::explain(&code),
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
