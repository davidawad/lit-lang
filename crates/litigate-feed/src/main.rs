//! `litigate-feed`: one poll of the opinion feed ([`litigate_feed`]).

use clap::Parser;
use litigate_feed::bead::{Br, Filer};
use litigate_feed::courtlistener::{Client, BASE_URL};
use litigate_feed::http::Ureq;
use litigate_feed::inventory::{build_elements, Inventory};
use litigate_feed::state::State;
use litigate_feed::{date, default_since, poll, watched, Options};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

/// Poll `CourtListener` for new opinions from the modeled courts or citing a
/// precedent layer, and file a bead for each (lg-8fdj.1).
#[derive(Parser)]
#[command(name = "litigate-feed", version)]
struct Args {
    /// The repository root (holding `lit/`).
    #[arg(long, default_value = ".")]
    repo: PathBuf,
    /// The state file: opinions already filed, the last complete poll.
    #[arg(long, default_value = "opinion-feed-state.json")]
    state: PathBuf,
    /// The first filing date to poll (default: the day before the last
    /// complete poll, else yesterday).
    #[arg(long, value_name = "YYYY-MM-DD")]
    since: Option<String>,
    /// File the beads (`br create`) and record them in the state file;
    /// without it, print what would be filed and record nothing.
    #[arg(long)]
    file: bool,
    /// The `br` program filing the beads.
    #[arg(long, default_value = "br")]
    br: String,
    /// The `CourtListener` host.
    #[arg(long, default_value = BASE_URL)]
    base_url: String,
    /// The least time between requests, in milliseconds.
    #[arg(long, default_value_t = 1000)]
    min_interval_ms: u64,
    /// The most opinions read in one run; the rest wait for the next.
    #[arg(long, default_value_t = 100)]
    max_opinions: usize,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(&args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("litigate-feed: {e}");
            ExitCode::from(1)
        }
    }
}

fn run(args: &Args) -> Result<ExitCode, String> {
    let inventory = Inventory::read(&args.repo)?;
    let elements = build_elements(&args.repo, &inventory, &watched(&inventory))?;
    let mut state = State::load(&args.state)?;
    let today = date::today();
    let since = args
        .since
        .clone()
        .unwrap_or_else(|| default_since(&state, &today));
    if date::parse_days(&since).is_none() {
        return Err(format!("--since `{since}` is not YYYY-MM-DD"));
    }
    let token = std::env::var("COURTLISTENER_API_TOKEN").ok();
    let mut client = Client::new(Ureq::new(token), &args.base_url)
        .min_interval(Duration::from_millis(args.min_interval_ms));
    let mut br = Br {
        program: args.br.clone(),
    };
    let filer: Option<&mut dyn Filer> = if args.file { Some(&mut br) } else { None };
    let options = Options {
        since,
        today,
        max_opinions: args.max_opinions,
    };
    let report = poll(
        &inventory,
        &elements,
        &mut client,
        &mut state,
        filer,
        &options,
    )?;
    if args.file {
        state.save(&args.state)?;
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&report).unwrap_or_default()
    );
    Ok(ExitCode::from(u8::from(!report.errors.is_empty())))
}
