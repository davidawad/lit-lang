//! Nodes and edges: `RawNode`/`RawEdge` to `NodeDecl`/`EdgeDecl`/`LinkDecl`
//! source (SPEC §8.1 and §8.4 read in reverse).
use crate::issues::Issues;
use crate::text;
use litgraph::cite::normalize::{parse_cite_string, CiteGrammar};
use litgraph::model::{Deadline, Duration, NodeKind, RawEdge, RawNode};
use std::collections::BTreeMap;

/// What one import needs to spell nodes and edges.
pub(crate) struct Spelling<'a> {
    /// The citing pack's declared citation syntax.
    pub grammar: CiteGrammar<'a>,
    /// Where diagnostics go.
    pub issues: &'a mut Issues,
}

impl Spelling<'_> {
    /// A `cite ...` clause for free text: citation syntax when every
    /// citation in it parses under the pack's grammar, else a cite hole.
    pub(crate) fn cite(&self, raw: &str) -> String {
        let refs = parse_cite_string(raw, &self.grammar);
        let recognized = !refs.is_empty() && refs.iter().all(|r| r.heading().is_some());
        text::cite(raw, recognized)
    }

    fn id_list(&mut self, items: &[String], pointer: &str, qualified: bool) -> String {
        let items: Vec<String> = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                self.issues
                    .ident(item, &format!("{pointer}/{i}"), qualified)
            })
            .collect();
        format!("[{}]", items.join(", "))
    }

    fn num_record(&mut self, record: &BTreeMap<String, f64>, pointer: &str, money: bool) -> String {
        let entries: Vec<String> = record
            .iter()
            .map(|(key, value)| {
                let at = format!("{pointer}/{key}");
                let key = self.issues.ident(key, &at, false);
                let spelled = if money {
                    text::money(*value)
                } else {
                    text::number(*value)
                };
                format!("{key}: {}", self.issues.spell(spelled, &at, "value"))
            })
            .collect();
        format!("{{ {} }}", entries.join(", "))
    }

    /// The body lines of a node (`group`, `note`, ...), without edges.
    fn node_fields(&mut self, node: &RawNode, pointer: &str) -> Vec<String> {
        let mut fields = Vec::new();
        if let Some(group) = &node.group {
            let group = self.issues.ident(group, &format!("{pointer}/group"), false);
            fields.push(format!("group {group}"));
        }
        self.common(
            &mut fields,
            node.note.as_deref(),
            node.valence.as_deref(),
            node.court_listener_url.as_deref(),
            pointer,
        );
        if !node.tags.is_empty() {
            let tags = self.id_list(&node.tags, &format!("{pointer}/tags"), false);
            fields.push(format!("tags {tags}"));
        }
        if !node.attrs.is_empty() {
            let attrs = self.num_record(&node.attrs, &format!("{pointer}/attrs"), false);
            fields.push(format!("attrs {attrs}"));
        }
        if let Some(payoff) = node.payoff {
            let at = format!("{pointer}/payoff");
            fields.push(format!(
                "payoff {}",
                self.issues.spell(text::money(payoff), &at, "payoff")
            ));
        }
        if !node.payoff_by_flag.is_empty() {
            let at = format!("{pointer}/payoffByFlag");
            let record = self.num_record(&node.payoff_by_flag, &at, true);
            fields.push(format!("payoff_by_flag {record}"));
        }
        fields
    }

    fn common(
        &mut self,
        fields: &mut Vec<String>,
        note: Option<&str>,
        valence: Option<&str>,
        url: Option<&str>,
        pointer: &str,
    ) {
        if let Some(note) = note {
            fields.push(format!("note {}", text::string(note)));
        }
        if let Some(valence) = valence {
            let valence = self
                .issues
                .ident(valence, &format!("{pointer}/valence"), false);
            fields.push(format!("valence {valence}"));
        }
        if let Some(url) = url {
            fields.push(format!("court_listener_url {}", text::string(url)));
        }
    }

    /// A node declaration with its edges nested in source order.
    ///
    /// `edges` pairs each out-edge with its index in the pack's `edges`.
    pub(crate) fn node(
        &mut self,
        index: usize,
        node: &RawNode,
        edges: &[(usize, &RawEdge)],
    ) -> String {
        let pointer = format!("/nodes/{index}");
        let id = self.issues.ident(&node.id, &format!("{pointer}/id"), false);
        let terminal = node.kind == Some(NodeKind::Terminal);
        let keyword = match node.kind {
            Some(NodeKind::Terminal) => "terminal",
            Some(NodeKind::Decision) => "decision",
            Some(NodeKind::State) | None if is_chance(&node.id, edges) => "chance",
            Some(NodeKind::State) | None => "state",
        };
        if !terminal
            && (!node.outcome.is_empty()
                || node.payoff.is_some()
                || !node.payoff_by_flag.is_empty())
        {
            self.issues.report(
                &pointer,
                "`outcome`, `payoff` and `payoffByFlag` are valid only on terminals (SPEC §5.1)",
            );
        }
        let actor = plurality_actor(edges);
        let mut header = vec![format!(
            "/// imported\n{keyword} {id}: {}",
            text::string(&node.label)
        )];
        if actor != "either" {
            let by = self.issues.ident(actor, &format!("{pointer}/actor"), false);
            header.push(format!("by {by}"));
        }
        if !node.outcome.is_empty() {
            let outcome = self.id_list(&node.outcome, &format!("{pointer}/outcome"), false);
            header.push(format!("outcome {outcome}"));
        }
        if let Some(cite) = &node.cite {
            header.push(self.cite(cite));
        }
        let header = header.join(" ");
        let mut body = self.node_fields(node, &pointer);
        for (edge_index, edge) in edges {
            body.push(self.edge(*edge_index, edge, actor));
        }
        if body.is_empty() {
            header
        } else {
            format!("{header} {{\n{}\n}}", body.join("\n"))
        }
    }

    /// An edge declaration inside its source node, whose effective actor is
    /// `node_actor`.
    fn edge(&mut self, index: usize, edge: &RawEdge, node_actor: &str) -> String {
        let pointer = format!("/edges/{index}");
        let id = match &edge.id {
            Some(id) => self.issues.ident(id, &format!("{pointer}/id"), false),
            None => "_".to_owned(),
        };
        let to = self.issues.ident(&edge.to, &format!("{pointer}/to"), false);
        let head = format!("{id} {} -> {to}", text::string(&edge.label));
        self.edge_rest(head, edge, node_actor, &pointer)
    }

    /// A `link` declaration of a links file.
    pub(crate) fn link(&mut self, index: usize, edge: &RawEdge) -> String {
        let pointer = format!("/links/{index}");
        let id = match &edge.id {
            Some(id) => self.issues.ident(id, &format!("{pointer}/id"), false),
            None => "_".to_owned(),
        };
        let from = self
            .issues
            .ident(&edge.from, &format!("{pointer}/from"), true);
        let to = self.issues.ident(&edge.to, &format!("{pointer}/to"), true);
        let head = format!(
            "/// imported\nlink {id} {}: {from} -> {to}",
            text::string(&edge.label)
        );
        self.edge_rest(head, edge, "either", &pointer)
    }

    fn edge_rest(
        &mut self,
        head: String,
        edge: &RawEdge,
        node_actor: &str,
        pointer: &str,
    ) -> String {
        let mut head = vec![head];
        if edge.actor != node_actor {
            let by = self
                .issues
                .ident(&edge.actor, &format!("{pointer}/actor"), false);
            head.push(format!("by {by}"));
        }
        if let Some(p) = edge.probability {
            let at = format!("{pointer}/probability");
            head.push(format!(
                "p {}",
                self.issues.spell(text::number(p), &at, "probability")
            ));
        }
        let head = head.join(" ");
        let fields = self.edge_fields(edge, pointer);
        if fields.is_empty() {
            head
        } else {
            format!("{head} {{\n{}\n}}", fields.join("\n"))
        }
    }

    /// The `EdgeFields` block. `replaces` is non-empty only on links:
    /// `Pack::from_json` rejects it on a pack edge before import.
    fn edge_fields(&mut self, edge: &RawEdge, pointer: &str) -> Vec<String> {
        let mut fields = Vec::new();
        if let Some(authority) = &edge.authority {
            fields.push(format!("authority {}", self.cite(authority)));
        }
        if let Some(hours) = edge.hours {
            let at = format!("{pointer}/hours");
            let spelled = text::duration(hours, "hours", false);
            fields.push(format!(
                "effort {}",
                self.issues.spell(spelled, &at, "hours")
            ));
        }
        if let Some(cost) = edge.cost {
            let at = format!("{pointer}/cost");
            fields.push(format!(
                "cost {}",
                self.issues.spell(text::money(cost), &at, "cost")
            ));
        }
        if let Some(deadline) = &edge.deadline {
            fields.push(self.deadline(deadline, &format!("{pointer}/deadline")));
        }
        if let Some(duration) = &edge.duration {
            fields.push(self.duration(duration, &format!("{pointer}/duration")));
        }
        self.common(
            &mut fields,
            None,
            edge.valence.as_deref(),
            edge.court_listener_url.as_deref(),
            pointer,
        );
        if let Some(action) = &edge.action_id {
            fields.push(format!("action_id {}", text::string(action)));
        }
        if let Some(note) = &edge.note {
            fields.push(format!("note {}", text::string(note)));
        }
        if !edge.tags.is_empty() {
            fields.push(format!(
                "tags {}",
                self.id_list(&edge.tags, &format!("{pointer}/tags"), false)
            ));
        }
        if !edge.attrs.is_empty() {
            let attrs = self.num_record(&edge.attrs, &format!("{pointer}/attrs"), false);
            fields.push(format!("attrs {attrs}"));
        }
        let flags = [
            ("sets", &edge.sets),
            ("clears", &edge.clears),
            ("requires", &edge.requires),
            ("forbids", &edge.forbids),
        ];
        for (keyword, list) in flags {
            if !list.is_empty() {
                let at = format!("{pointer}/{keyword}");
                fields.push(format!("{keyword} {}", self.id_list(list, &at, false)));
            }
        }
        if !edge.replaces.is_empty() {
            // An anonymous edge has only its derived id (`pack::a->b#0`),
            // which is written as a string (SPEC §3.1).
            let items: Vec<String> = edge
                .replaces
                .iter()
                .map(|item| {
                    if text::is_ref(item) {
                        item.clone()
                    } else {
                        text::string(item)
                    }
                })
                .collect();
            fields.push(format!("replaces [{}]", items.join(", ")));
        }
        fields
    }

    fn deadline(&mut self, deadline: &Deadline, pointer: &str) -> String {
        let unit = match deadline.unit.as_deref() {
            None | Some("calendar") => "days",
            Some("court") => "court_days",
            Some("week") => "weeks",
            Some("month") => "months",
            Some("year") => "years",
            Some(other) => {
                self.issues.report(
                    &format!("{pointer}/unit"),
                    format!(
                        "deadline unit `{other}` has no .lit spelling (calendar, court, week, \
                         month or year)"
                    ),
                );
                "days"
            }
        };
        let length = text::duration(deadline.length, unit, true);
        let out = format!(
            "deadline {}",
            self.issues.spell(
                length,
                &format!("{pointer}/length"),
                "a whole-day deadline length"
            )
        );
        let mut opts = Vec::new();
        if let Some(extendable) = deadline.extendable {
            opts.push(format!("extendable {extendable}"));
        }
        if let Some(authority) = &deadline.extension_authority {
            opts.push(format!("extension_authority {}", text::string(authority)));
        }
        if let Some(note) = &deadline.note {
            opts.push(format!("note {}", text::string(note)));
        }
        if opts.is_empty() {
            out
        } else {
            format!("{out} {{ {} }}", opts.join(", "))
        }
    }

    fn duration(&mut self, duration: &Duration, pointer: &str) -> String {
        let mut days = |value: f64, key: &str| {
            let spelled = text::duration(value, "days", false);
            self.issues
                .spell(spelled, &format!("{pointer}/{key}"), "a duration")
        };
        let mode = days(duration.mode, "mode");
        if duration.min.is_none() && duration.max.is_none() {
            return format!("duration {mode}");
        }
        let mut parts = Vec::new();
        if let Some(min) = duration.min {
            parts.push(format!("min {}", days(min, "min")));
        }
        parts.push(format!("mode {mode}"));
        if let Some(max) = duration.max {
            parts.push(format!("max {}", days(max, "max")));
        }
        format!("duration {{ {} }}", parts.join(", "))
    }
}

/// A `state` whose every out-edge carries `p` and none loops back is
/// written as the `chance` sugar (SPEC §5.1); it lowers to `state` again.
fn is_chance(id: &str, edges: &[(usize, &RawEdge)]) -> bool {
    !edges.is_empty()
        && edges
            .iter()
            .all(|(_, edge)| edge.probability.is_some() && edge.to != id)
}

/// The actor most edges of a node carry (first seen wins a tie); written
/// once as the node's `by` so edges repeat only the exceptions.
fn plurality_actor<'a>(edges: &[(usize, &'a RawEdge)]) -> &'a str {
    let mut counts: Vec<(&str, usize)> = Vec::new();
    for (_, edge) in edges {
        match counts.iter_mut().find(|(actor, _)| *actor == edge.actor) {
            Some((_, count)) => *count += 1,
            None => counts.push((edge.actor.as_str(), 1)),
        }
    }
    counts
        .iter()
        .fold(
            None,
            |best: Option<(&str, usize)>, &(actor, count)| match best {
                Some((_, top)) if top >= count => best,
                _ => Some((actor, count)),
            },
        )
        .map_or("either", |(actor, _)| actor)
}
