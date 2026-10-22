//! Splitting a pack's nodes into modules by group.
//!
//! Lowering emits nodes in module order (package-relative path order), then
//! source order (SPEC §4.1), and node order is observable (Monte Carlo
//! sampling visits nodes in order). So a module holds a contiguous run of
//! the pack's nodes, and module files are numbered in node order. Module
//! imports must be acyclic (L1003), so runs whose nodes reach each other
//! both ways are merged, together with every run between them, until the
//! module graph is a DAG.
use litgraph::model::Pack;
use std::collections::BTreeMap;
use std::ops::Range;

/// One module's share of the pack: a contiguous node range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Section {
    /// Module name segment (`pleading`, `pleading-to-trial`).
    pub name: String,
    /// Node indices, contiguous and in pack order.
    pub nodes: Range<usize>,
}

/// The module sections of `pack`, in node order.
pub(crate) fn sections(pack: &Pack) -> Vec<Section> {
    let groups: Vec<Option<&str>> = pack.nodes.iter().map(|n| n.group.as_deref()).collect();
    let mut runs: Vec<Range<usize>> = Vec::new();
    for (index, group) in groups.iter().enumerate() {
        match runs.last_mut() {
            Some(run) if groups[run.start] == *group => run.end = index + 1,
            _ => runs.push(index..index + 1),
        }
    }
    let index_of: BTreeMap<&str, usize> = pack
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.id.as_str(), index))
        .collect();
    let arcs: Vec<(usize, usize)> = pack
        .edges
        .iter()
        .filter_map(|edge| {
            Some((
                *index_of.get(edge.from.as_str())?,
                *index_of.get(edge.to.as_str())?,
            ))
        })
        .collect();
    while let Some((first, last)) = cycle_span(&runs, &arcs) {
        let merged = runs[first].start..runs[last].end;
        runs.splice(first..=last, [merged]);
    }
    name_sections(&runs, &groups)
}

/// The closest pair of runs `(i, j)`, `i < j`, that reach each other (the
/// smallest span, then the earliest), if any.
fn cycle_span(runs: &[Range<usize>], arcs: &[(usize, usize)]) -> Option<(usize, usize)> {
    let count = runs.len();
    let run_of = |node: usize| runs.iter().position(|run| run.contains(&node));
    let mut reach = vec![vec![false; count]; count];
    for &(from, to) in arcs {
        if let (Some(a), Some(b)) = (run_of(from), run_of(to)) {
            reach[a][b] = true;
        }
    }
    // Warshall's transitive closure.
    for k in 0..count {
        let via = reach[k].clone();
        for row in &mut reach {
            if row[k] {
                for (cell, &through) in row.iter_mut().zip(&via) {
                    *cell = *cell || through;
                }
            }
        }
    }
    (0..count)
        .flat_map(|i| (i + 1..count).map(move |j| (i, j)))
        .filter(|&(i, j)| reach[i][j] && reach[j][i])
        .min_by_key(|&(i, j)| (j - i, i))
}

/// A group id as a module name segment: `pleading-group` -> `pleading`.
fn short(group: Option<&str>) -> &str {
    group.map_or("ungrouped", |g| {
        g.strip_suffix("-group")
            .filter(|s| !s.is_empty())
            .unwrap_or(g)
    })
}

fn name_sections(runs: &[Range<usize>], groups: &[Option<&str>]) -> Vec<Section> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    runs.iter()
        .map(|run| {
            let first = short(groups[run.start]);
            let last = short(groups[run.end - 1]);
            let base = if first == last {
                first.to_owned()
            } else {
                format!("{first}-to-{last}")
            };
            let count = seen.entry(base.clone()).or_insert(0);
            *count += 1;
            let name = if *count == 1 {
                base
            } else {
                format!("{base}-{count}")
            };
            Section {
                name,
                nodes: run.clone(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
