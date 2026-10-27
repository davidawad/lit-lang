// SPDX-License-Identifier: GPL-3.0-or-later
// The `litigate` CLI over an in-memory file tree, through litigate-wasm's
// `litigate` binding: `check`, `build`, `run`, the op subcommands and the
// verified case boundary (`timeline`, `calendar`, `actions`,
// `lint-filing`: replayed history, derived deadlines, posture and
// markings), with the CLI's arguments and envelope, files passed in (and
// `build`'s and `calendar`'s outputs handed back) instead of a disk.
import type { LitigateEnvelope, LitigateRun, Profile, Timeline } from "./generated/schemas.js";

/** The litigate-wasm functions this module calls. */
export interface LitigateWasm {
  litigate(args_json: string, files_json: string, with_library: boolean, today?: string): string;
  litigate_library(): string;
  litigate_cases(): string;
}

/** Files by repository-relative POSIX path (`lit/frcp/Lit.toml`, `cases/x.case`, ...) to text. */
export type Files = Readonly<Record<string, string>>;

/** The `litigate check` checking profile (`--profile`), as diagnostics carry it. */
export type { Profile };

/** Options shared by {@link Litigate.check} and {@link Litigate.build}. */
export interface CheckOptions {
  /** The file tree the paths resolve in. */
  readonly files?: Files;
  /**
   * Fill in every `lit/` path `files` does not give from the `lit/`
   * library embedded in the wasm (default `true`), so a matter's `uses`
   * resolve with no library passed.
   */
  readonly library?: boolean;
  /** `--profile` (the CLI's default is `dev`). */
  readonly profile?: Profile;
  /** `--as-of DATE`. */
  readonly asOf?: string;
  /** `--known-at DATE`. */
  readonly knownAt?: string;
  /**
   * The date `check` judges deadlines at, as the native binary judges them
   * at its `LITIGATE_TODAY` or system date (default: today's UTC date;
   * an explicit `asOf` turns it off).
   */
  readonly today?: string;
  /** `--lib DIR` library directories (default `lit`). */
  readonly lib?: readonly string[];
}

/** {@link Litigate.build} options. */
export interface BuildOptions extends CheckOptions {
  /** `--out DIR`: the prefix of every output path (the CLI's default is `build`). */
  readonly out?: string;
  /** `--pack-file ID=STEM` mappings, pack id to file stem. */
  readonly packFiles?: Readonly<Record<string, string>>;
}

/** Options shared by the case-boundary commands. */
export interface CaseOptions {
  /** The file tree the paths resolve in. */
  readonly files?: Files;
  /** Fill in the `lit/` library embedded in the wasm (default `true`), as {@link CheckOptions.library}. */
  readonly library?: boolean;
  /** `--lib DIR` library directories (default `lit`). */
  readonly lib?: readonly string[];
}

/** {@link Litigate.timeline} options. */
export interface TimelineOptions extends CaseOptions {
  /** `--as-of DATE`. */
  readonly asOf?: string;
  /** `--known-at DATE`. */
  readonly knownAt?: string;
  /** Judge deadlines at this date when the matter's `as_of` is older (default: today's UTC date). */
  readonly today?: string;
  /** `--layers` toggles: `ID` or `+ID` enables (only those named), `-ID` disables. */
  readonly layers?: readonly string[];
  /** `--world W`: only this world and its ancestors. */
  readonly world?: string;
}

/** {@link Litigate.calendar} options. */
export interface CalendarOptions extends CaseOptions {
  /** `--out FILE`: also write the `.ics` there (handed back in `files`). */
  readonly out?: string;
  /** `--all`: every deadline the events created, not only the open ones. */
  readonly all?: boolean;
  /** `--remind DAYS` before each due date. */
  readonly remind?: readonly number[];
}

/** {@link Litigate.actions} options. */
export interface ActionsOptions extends CaseOptions {
  /** `--date DATE` the moves would be made (default: the matter's `as_of`). */
  readonly date?: string;
}

/** What the filer says of the paper (`lint-filing`'s paper flags). */
export interface Paper {
  /** `--words N`. */
  readonly words?: number;
  /** `--pages N`. */
  readonly pages?: number;
  /** `--lines N`. */
  readonly lines?: number;
  /** `--document FILE`: the paper as UTF-8 text in `files`, its words counted (instead of `words`). */
  readonly document?: string;
  /** `--paper KIND`. */
  readonly paper?: string;
  /** `--certificate KIND` (repeatable). */
  readonly certificates?: readonly string[];
}

/** A proposed filing for {@link Litigate.lintFiling}. */
export interface Filing {
  /** The event as the matter would record it, e.g. `Answered(complaint)`. */
  readonly event: string;
  /** The filing date. */
  readonly date: string;
  /** The party making the filing (`--by`), or the party served (`--on`). */
  readonly party: { readonly by: string } | { readonly on: string };
  /** What the filer says of the paper. */
  readonly paper?: Paper;
}

/** {@link Litigate.timeline}'s result: the envelope's `result` as the `Timeline` it was serialized from. */
export interface TimelineResult extends LitigateResult {
  /** The matter's or module's timeline; absent when it did not resolve. */
  readonly timeline?: Timeline;
}

/** One check or build: the exit code, the parsed envelope and what was written. */
export interface LitigateResult {
  /** 0 ok, 2 an error diagnostic, 1 a usage or I/O failure (`envelope` then absent). */
  readonly code: number;
  /** The JSON envelope `litigate --format json` prints; absent on a usage or I/O failure. */
  readonly envelope?: LitigateEnvelope;
  /** The usage or I/O failure message, else empty. */
  readonly stderr: string;
  /** Files `build` wrote, by path (`OUT/packs/<id>.json`, ...); empty for `check`. */
  readonly files: Files;
}

/** The `litigate` CLI over in-memory files. */
export class Litigate {
  readonly #wasm: LitigateWasm;

  constructor(wasm: LitigateWasm) {
    this.#wasm = wasm;
  }

  /**
   * Runs `litigate ARGS` (no program name) over `files`, exactly as the
   * CLI over a disk holding them. `check`, `build`, `worlds` (alternate
   * timelines), `run`, the op subcommands (`solve`, `chain`, ...) and
   * `engine` run; any other command exits 1. `check` judges deadlines
   * at `today` unless told a date, as the native binary does at its own
   * today (default: today's UTC date).
   */
  run(args: readonly string[], files: Files = {}, library = true, today: string = utcToday()): LitigateRun {
    const json = this.#wasm.litigate(JSON.stringify(args), JSON.stringify(files), library, today);
    return parse<LitigateRun>(json);
  }

  /** `litigate check PATHS`: typecheck and verify packages and matters. */
  check(paths: readonly string[], options: CheckOptions = {}): LitigateResult {
    return this.#envelope(["check", ...flags(options), ...paths], options);
  }

  /** `litigate build PATHS`: check, then lower; the outputs come back in `files`. */
  build(paths: readonly string[], options: BuildOptions = {}): LitigateResult {
    const out = options.out === undefined ? [] : ["--out", options.out];
    const names = Object.entries(options.packFiles ?? {}).flatMap(([id, stem]) => ["--pack-file", `${id}=${stem}`]);
    return this.#envelope(["build", ...out, ...names, ...flags(options), ...paths], options);
  }

  /**
   * `litigate timeline FILE`: a matter's replayed history, deadlines and
   * statuses (or a module's), judged at `today` unless `asOf` is given.
   */
  timeline(file: string, options: TimelineOptions = {}): TimelineResult {
    const args = ["timeline", ...dates(options), ...libs(options), file];
    for (const layer of options.layers ?? []) args.push(`--layers=${layer}`);
    if (options.world !== undefined) args.push("--world", options.world);
    const run = this.#envelope(args, options);
    const result = run.envelope?.result;
    return result === undefined || result === null ? run : { ...run, timeline: result as Timeline };
  }

  /** `litigate calendar CASE`: the matter's computed deadlines as an iCalendar file. */
  calendar(file: string, options: CalendarOptions = {}): LitigateResult {
    const args = ["calendar", ...libs(options), file];
    if (options.out !== undefined) args.push("--out", options.out);
    if (options.all === true) args.push("--all");
    for (const days of options.remind ?? []) args.push("--remind", String(days));
    return this.#envelope(args, options);
  }

  /** `litigate actions CASE --party PARTY`: the moves replay accepts for the party in the posture, with the marking. */
  actions(file: string, party: string, options: ActionsOptions = {}): LitigateResult {
    const args = ["actions", ...libs(options), file, "--party", party];
    if (options.date !== undefined) args.push("--date", options.date);
    return this.#envelope(args, options);
  }

  /** `litigate lint-filing CASE`: replay a proposed filing; is it legal and timely, what it discharges and triggers. */
  lintFiling(file: string, filing: Filing, options: CaseOptions = {}): LitigateResult {
    const [mode, party] = "by" in filing.party ? ["--by", filing.party.by] : ["--on", filing.party.on];
    const args = ["lint-filing", ...libs(options), file, filing.event, "--date", filing.date, mode, party];
    const paper = filing.paper ?? {};
    if (paper.words !== undefined) args.push("--words", String(paper.words));
    if (paper.pages !== undefined) args.push("--pages", String(paper.pages));
    if (paper.lines !== undefined) args.push("--lines", String(paper.lines));
    if (paper.document !== undefined) args.push("--document", paper.document);
    if (paper.paper !== undefined) args.push("--paper", paper.paper);
    for (const c of paper.certificates ?? []) args.push("--certificate", c);
    return this.#envelope(args, options);
  }

  /** The `lit/` library embedded in the wasm, by path. */
  library(): Files {
    return parse<Record<string, string>>(this.#wasm.litigate_library());
  }

  /** The `cases/` matters embedded in the wasm, by path. */
  cases(): Files {
    return parse<Record<string, string>>(this.#wasm.litigate_cases());
  }

  #envelope(args: readonly string[], options: CaseOptions & { readonly today?: string }): LitigateResult {
    const run = this.run(["--format", "json", ...args], options.files, options.library, options.today);
    const files: Record<string, string> = {};
    for (const [path, text] of Object.entries(run.files)) if (text !== undefined) files[path] = text;
    const result = { code: run.code, stderr: run.stderr, files };
    return run.stdout === "" ? result : { ...result, envelope: parse<LitigateEnvelope>(run.stdout) };
  }
}

/** The CLI flags of `options` shared by check and build. */
function flags(options: CheckOptions): string[] {
  const out: string[] = options.profile === undefined ? [] : ["--profile", options.profile];
  return [...out, ...dates(options), ...libs(options)];
}

/** `--as-of` and `--known-at`, when given. */
function dates(options: { readonly asOf?: string; readonly knownAt?: string }): string[] {
  const out: string[] = [];
  if (options.asOf !== undefined) out.push("--as-of", options.asOf);
  if (options.knownAt !== undefined) out.push("--known-at", options.knownAt);
  return out;
}

/** `--lib DIR` for each library directory. */
function libs(options: CaseOptions): string[] {
  return (options.lib ?? []).flatMap((dir) => ["--lib", dir]);
}

/** Today's date in UTC (`YYYY-MM-DD`), the native binary's default today. */
function utcToday(): string {
  return new Date().toISOString().slice(0, 10);
}

/** The binding's JSON, typed by the Rust type it was serialized from. */
function parse<T>(json: string): T {
  const value: unknown = JSON.parse(json);
  return value as T;
}
