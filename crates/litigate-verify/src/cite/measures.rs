/// The lines of `body` under its pinpoint `markers` in turn (`(d)`, then
/// `(1)` under it), each running to the next marker no deeper than its
/// own; a marker the body does not write stops the narrowing there.
fn subdivision(body: &str, markers: &[String]) -> String {
    let lines: Vec<&str> = body.lines().collect();
    let indent = |l: &str| l.len() - l.trim_start().len();
    let marker = |l: &str| {
        let t = l.trim_start();
        t.starts_with('(')
            && t[1..].split_once(')').is_some_and(|(m, _)| {
                !m.is_empty() && m.len() <= 5 && m.chars().all(char::is_alphanumeric)
            })
    };
    let (mut first, mut last) = (0, lines.len());
    for m in markers {
        let open = format!("({m})");
        let Some(at) = (first..last).find(|&i| lines[i].trim_start().starts_with(&open)) else {
            break;
        };
        let depth = indent(lines[at]);
        last = (at + 1..last)
            .find(|&i| marker(lines[i]) && indent(lines[i]) <= depth + 1)
            .unwrap_or(last);
        first = at;
    }
    lines[first..last].join("\n")
}

/// The most words between a count and its measure: `fourteen (14)
/// calendar days`.
const COUNT_WORDS: usize = 3;

/// What a rule counts a period in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Measure {
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl Measure {
    fn of(unit: DayUnit) -> Self {
        match unit {
            DayUnit::Calendar | DayUnit::Court | DayUnit::Business => Self::Day,
            DayUnit::Week => Self::Week,
            DayUnit::Month => Self::Month,
            DayUnit::Year => Self::Year,
            DayUnit::Hour => Self::Hour,
        }
    }

    /// The measure a word names (`days`, `day,`, `hours`), if any.
    fn named(word: &str) -> Option<Self> {
        let word = word.trim_end_matches(|c: char| !c.is_ascii_alphabetic());
        Some(match word.strip_suffix('s').unwrap_or(word) {
            "hour" => Self::Hour,
            "day" => Self::Day,
            "week" => Self::Week,
            "month" => Self::Month,
            "year" => Self::Year,
            _ => return None,
        })
    }

    const fn monthly(self) -> bool {
        matches!(self, Self::Month | Self::Year)
    }

    /// `n` of this measure and the same period in the others it converts
    /// to exactly.
    fn equivalents(self, n: u64) -> Vec<(u64, Self)> {
        let mut out = vec![(n, self)];
        match self {
            Self::Day => {
                out.extend(n.checked_mul(24).map(|hours| (hours, Self::Hour)));
                if n.is_multiple_of(7) {
                    out.push((n / 7, Self::Week));
                }
            }
            Self::Week => {
                out.extend(n.checked_mul(7).map(|days| (days, Self::Day)));
                out.extend(n.checked_mul(168).map(|hours| (hours, Self::Hour)));
            }
            Self::Month if n.is_multiple_of(12) => out.push((n / 12, Self::Year)),
            Self::Year => out.extend(n.checked_mul(12).map(|months| (months, Self::Month))),
            Self::Hour | Self::Month => {}
        }
        out
    }

    fn prose(self, n: u64) -> String {
        let noun = match self {
            Self::Hour => "hour",
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
            Self::Year => "year",
        };
        format!("{n} {noun}{}", if n == 1 { "" } else { "s" })
    }
}

/// Every count `body` states with its measure: a count in digits or words
/// up to [`COUNT_WORDS`] words before a measure (`14 days`, `fourteen (14)
/// calendar days`, `one hundred eighty days`, `35days`), hyphened to it
/// (`14-day`, `seven-day`), or an ordinal day (`the 21st day`, `the next
/// day`); `the prior Monday` is a week where the body counts from a
/// Monday (`hearings scheduled on Mondays`, Bankr. M.D. La. SO 2025-1),
/// and no count where it counts from another day (`falls on a Saturday
/// ... the following Monday`, two days). A page opening on a measure, under
/// any running head, takes a count ending one of the previous page's last
/// lines (`within 3`, `seven (7)`): the footer and running head a
/// `pdftotext -layout` extract sets between them (`within 3` / `Page 38` /
/// `days`, Bankr. S.D. Tex. BLR 9017-1(c); `seven (7)` / `[13]` / `days`,
/// TNWB LBR 2083-1(b)(2)) are not the rule's words.
fn counts(body: &str) -> Vec<(u64, Measure)> {
    let text: String = body
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | ',') {
                c
            } else {
                ' '
            }
        })
        .collect();
    let mut words: Vec<String> = Vec::new();
    for w in text.split_whitespace() {
        let w = w
            .trim_matches(|c: char| matches!(c, '-' | ','))
            .to_lowercase();
        let at = w.find(|c: char| !c.is_ascii_digit() && c != ',');
        match at.map(|i| w.split_at(i)) {
            Some((n, m)) if !n.is_empty() && Measure::named(m).is_some() => {
                words.extend([n.to_owned(), m.to_owned()]);
            }
            _ => words.push(w),
        }
    }
    let mut out = Vec::new();
    for (i, word) in words.iter().enumerate() {
        if let Some(m) = Measure::named(word) {
            let from = i.saturating_sub(COUNT_WORDS + 3);
            for j in from..i {
                for k in (j + 1)..=i {
                    if k - j > 3 || i - k > COUNT_WORDS || within_count(&words, j, k, i) {
                        continue;
                    }
                    if let Some(n) = number(&words[j..k].join(" ")) {
                        out.push((n, m));
                    }
                }
            }
        } else if let Some((count, measure)) = word.rsplit_once('-') {
            if let (Some(n), Some(m)) = (number(count), Measure::named(measure)) {
                out.push((n, m));
            }
        } else if relative_weekday(&words, i)
            && words.iter().enumerate().any(|(j, w)| {
                j != i && !relative_weekday(&words, j) && w.trim_end_matches('s') == word.as_str()
            })
        {
            out.push((1, Measure::Week));
        }
    }
    let lines: Vec<&str> = body.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let Some(page) = line.strip_prefix('\u{c}') else {
            continue;
        };
        let head = lines[i + 1..lines.len().min(i + PAGE_FOOT)].iter();
        let Some(m) = std::iter::once(&page)
            .chain(head)
            .find_map(|l| Measure::named(&l.split_whitespace().next()?.to_lowercase()))
        else {
            continue;
        };
        out.extend(
            lines[i.saturating_sub(PAGE_FOOT)..i]
                .iter()
                .filter_map(|l| {
                    let mut w = l.split_whitespace().rev();
                    let (count, before) = (w.next()?, w.next()?);
                    let count = count.trim_matches(['(', ')']);
                    (INTRODUCING.contains(&before.to_lowercase().as_str())
                        || number(&before.to_lowercase()) == number(count))
                    .then(|| number(count))?
                })
                .map(|n| (n, m)),
        );
    }
    out
}

/// Whether `words[j..k]` is only part of a longer count before the
/// measure at `i`: `one`, `one hundred` and `fifty` in `one hundred fifty
/// (150) days` are not counts of their own.
fn within_count(words: &[String], j: usize, k: usize, i: usize) -> bool {
    (j.saturating_sub(3)..=j).any(|a| {
        (k..=i.min(a + 3)).any(|b| (a, b) != (j, k) && number(&words[a..b].join(" ")).is_some())
    })
}

/// Whether `words[i]` is a weekday counted from another (`the prior
/// Monday`, `the following Monday`).
fn relative_weekday(words: &[String], i: usize) -> bool {
    WEEKDAYS.contains(&words[i].as_str())
        && i > 0
        && matches!(
            words[i - 1].as_str(),
            "prior" | "preceding" | "following" | "next"
        )
}

/// The words that introduce a period's count (`within 3`, `at least 7`).
const INTRODUCING: [&str; 5] = ["within", "least", "than", "additional", "of"];

/// The most lines a page's footer and running head take below its text.
const PAGE_FOOT: usize = 12;

const WEEKDAYS: [&str; 7] = [
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
];

/// The count `phrase` writes, in digits (`14`, `1,000`), words
/// (`fourteen`, `twenty-one`, `one hundred eighty`) up to a thousand, or
/// as an ordinal (`21st`, `twenty-first`, `next`).
fn number(phrase: &str) -> Option<u64> {
    static SPELLED: LazyLock<HashMap<String, u64>> = LazyLock::new(|| {
        (1..=1000)
            .flat_map(|n| [(spelled(n), n), (ordinal_of(&spelled(n)), n)])
            .chain([("next".to_owned(), 1), ("following".to_owned(), 1)])
            .collect()
    });
    let digits = phrase.replace(',', "");
    let ordinal = ["st", "nd", "rd", "th"]
        .iter()
        .find_map(|x| digits.strip_suffix(x))
        .unwrap_or(&digits);
    if !ordinal.is_empty() && ordinal.len() <= 6 && ordinal.chars().all(|c| c.is_ascii_digit()) {
        return ordinal.parse().ok();
    }
    SPELLED.get(&phrase.replace(" and ", " ")).copied()
}

/// The ordinal of a spelled count: `twenty-one` → `twenty-first`.
fn ordinal_of(spelled: &str) -> String {
    let at = spelled.rfind([' ', '-']).map_or(0, |i| i + 1);
    let (head, last) = spelled.split_at(at);
    let last = match last {
        "one" => "first".to_owned(),
        "two" => "second".to_owned(),
        "three" => "third".to_owned(),
        "five" => "fifth".to_owned(),
        "eight" => "eighth".to_owned(),
        "nine" => "ninth".to_owned(),
        "twelve" => "twelfth".to_owned(),
        l if l.ends_with('y') => format!("{}ieth", &l[..l.len() - 1]),
        l => format!("{l}th"),
    };
    format!("{head}{last}")
}

/// Whether `body` holds `quote`, a [`GAP`] in it matching one to
/// [`GAP_WORDS`] whitespace-free words. Punctuation may end the words
/// before the gap: `five … pages` holds in `five, single-spaced pages`
/// (LCvR16-2(d)(3), N.D. Okla.).
fn holds(body: &str, quote: &str) -> bool {
    let Some((before, after)) = quote.split_once(&format!(" {GAP} ")) else {
        return body.contains(quote);
    };
    body.match_indices(before).any(|(i, _)| {
        let mut rest =
            body[i + before.len()..].trim_start_matches(|c: char| c.is_ascii_punctuation());
        (0..GAP_WORDS).any(|_| {
            let Some(r) = rest.strip_prefix(' ') else {
                return false;
            };
            let word = r.find(' ').unwrap_or(r.len());
            rest = &r[word..];
            word > 0 && rest.strip_prefix(' ').is_some_and(|r| r.starts_with(after))
        })
    })
}

/// `body` without the page number a `pdftotext -layout` extract sets on
/// its own line before each page break (form feed): a quote running across
/// the break (`3,500` / `89` / `words`, LR 72.2(c)(1)(A), D. Minn.) is
/// the rule's words with the page's folio between them.
fn unpaged(body: &str) -> Cow<'_, str> {
    if !body.contains('\u{c}') {
        return Cow::Borrowed(body);
    }
    let lines: Vec<&str> = body.lines().collect();
    let folio = |l: &str| {
        let l = l.trim();
        !l.is_empty()
            && l.len() <= 8
            && (l.chars().all(|c| c.is_ascii_digit())
                || l.chars().all(|c| matches!(c, 'i' | 'v' | 'x' | 'l' | 'c')))
    };
    let kept: Vec<&str> = lines
        .iter()
        .enumerate()
        .filter(|&(i, l)| {
            let breaks = lines.get(i + 1).is_some_and(|n| n.starts_with('\u{c}'));
            !(breaks && folio(l))
        })
        .map(|(_, l)| *l)
        .collect();
    Cow::Owned(kept.join("\n"))
}
