//! Explicit holes in generated documents (SPEC §8.1 "Holes"): each `?` the
//! source wrote in an emitted field becomes a `lowered.holes` entry whose
//! pointer names that field in the document, so a dev build's gaps stay
//! visible as gaps instead of reading as fields nobody authored.
use litigate_engine::model::{pointer_token, Hole};
use std::collections::BTreeMap;

/// The litgraph field a dotted `.lit` field path is emitted as, as a JSON
/// pointer suffix (`p` is `probability`, `duration.min` is `duration/min`,
/// `attrs.weight` is `attrs/weight`), or `None` for a field no document
/// carries (an `event` pattern, a flag list): its hole is checked, never
/// emitted.
fn field(path: &str) -> Option<String> {
    // `split` yields at least one part.
    let mut parts = path.split('.');
    let head = match parts.next().unwrap_or_default() {
        "p" => "probability",
        "effort" => "hours",
        "within" | "deadline" => "deadline",
        "payoff_by_flag" => "payoffByFlag",
        "court_listener_url" => "courtListenerUrl",
        "action_id" => "actionId",
        f @ ("cost" | "authority" | "duration" | "attrs" | "note" | "valence" | "cite"
        | "payoff" | "group" | "stake") => f,
        _ => return None,
    };
    let keyed = matches!(head, "attrs" | "payoffByFlag");
    Some(parts.fold(head.to_owned(), |out, part| {
        let token = if keyed {
            pointer_token(part)
        } else {
            camel(part)
        };
        format!("{out}/{token}")
    }))
}

/// `extension_authority` as `extensionAuthority`.
fn camel(snake: &str) -> String {
    let mut out = String::with_capacity(snake.len());
    let mut upper = false;
    for c in snake.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// The holes of one emitted value at `base` (`/edges/3`), from its
/// field-path map.
pub(crate) fn at(base: &str, holes: &BTreeMap<String, Option<String>>) -> Vec<Hole> {
    holes
        .iter()
        .filter_map(|(path, reason)| {
            Some(Hole {
                pointer: format!("{base}/{}", field(path)?),
                reason: reason.clone(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_paths_map_to_litgraph_pointers() {
        for (path, want) in [
            ("p", Some("probability")),
            ("effort", Some("hours")),
            (
                "within.extension_authority",
                Some("deadline/extensionAuthority"),
            ),
            ("deadline.note", Some("deadline/note")),
            ("duration.min", Some("duration/min")),
            ("attrs.a_b/c", Some("attrs/a_b~1c")),
            ("payoff_by_flag.won_it", Some("payoffByFlag/won_it")),
            ("court_listener_url", Some("courtListenerUrl")),
            ("action_id", Some("actionId")),
            ("cite", Some("cite")),
            ("event", None),
        ] {
            assert_eq!(field(path).as_deref(), want, "{path}");
        }
    }

    #[test]
    fn holes_carry_their_reasons_and_drop_unemitted_fields() {
        let holes = BTreeMap::from([
            ("p".to_owned(), Some("no data".to_owned())),
            ("sets".to_owned(), None),
        ]);
        assert_eq!(
            at("/edges/0", &holes),
            [Hole {
                pointer: "/edges/0/probability".into(),
                reason: Some("no data".into()),
            }]
        );
    }
}
