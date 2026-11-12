impl<'a> Builder<'a> {
    fn new(
        all: &'a [&'a hir::Package],
        texts: &'a [(String, String)],
        as_of: String,
        known_at: Option<String>,
    ) -> Self {
        Self {
            all,
            law: Vec::new(),
            texts: texts
                .iter()
                .map(|(f, t)| (f.as_str(), t.as_str()))
                .collect(),
            as_of,
            known_at,
            layers: Vec::new(),
            worlds: vec![TimelineWorld {
                id: "w0".to_owned(),
                label: "docket".to_owned(),
                parent: None,
                fork_row: None,
                choice: None,
            }],
            rows: Vec::new(),
            forks: vec![None],
            supersedes: Vec::new(),
            untriggered: Vec::new(),
            proofs: Vec::new(),
        }
    }

    /// Records each vendored amendment history proving that `cite` (of
    /// `pkg`, cited by a version in force from `from`) was that version's
    /// text, at `source`.
    fn prove(
        &mut self,
        pkg: &hir::Package,
        cite: Option<&str>,
        from: &str,
        source: Option<&TimelineSpan>,
    ) {
        let Some(cite) = cite else {
            return;
        };
        let citations = Citations::of_package(pkg, Some(crate::verdict::corpus()));
        for proof in citations.proofs(cite, from) {
            // A row repeated in another world repeats its proof: once.
            let at = (source.cloned(), proof);
            self.proofs.retain(|p| *p != at);
            self.proofs.push(at);
        }
    }

    /// `span` as a timeline span (byte range from the loaded text).
    fn span(&self, span: &Span) -> TimelineSpan {
        let text = texts_of(self, &span.file.0);
        let range = crate::span::byte_range(text, span);
        TimelineSpan {
            file: span.file.0.clone(),
            start: range.start,
            end: range.end,
            line: span.line,
            column: span.col,
        }
    }

    fn source(&self, span: Option<&Span>) -> Option<TimelineSpan> {
        span.map(|s| self.span(s))
    }

    /// The layer id of `pkg`, listing it on first use.
    fn layer(&mut self, pkg: &hir::Package) -> String {
        if !self.layers.iter().any(|l| l.id == pkg.manifest.name) {
            self.layers.push(layer_of(pkg));
        }
        pkg.manifest.name.clone()
    }

    fn push(&mut self, row: Draft) -> usize {
        self.rows.push(row);
        self.rows.len() - 1
    }

    /// Pushes `row`, whose cite (of `pkg`) a version in force from `from`
    /// makes, recording what proves it.
    fn push_cited(&mut self, row: Draft, pkg: &hir::Package, from: &str) -> usize {
        self.prove(pkg, row.cite.as_deref(), from, row.source.as_ref());
        self.push(row)
    }

    /// An L9008 note for each recorded proof.
    fn proof_notes(&self) -> impl Iterator<Item = TimelineDiagnostic> + '_ {
        self.proofs.iter().map(|(at, proof)| TimelineDiagnostic {
            code: DiagnosticCode::CiteProvedByHistory.as_str().to_owned(),
            severity: TimelineSeverity::Note,
            message: proof.clone(),
            source: at.clone(),
        })
    }

    /// The loaded package with pack id `pack`.
    fn package(&self, pack: &str) -> Option<&'a hir::Package> {
        self.all
            .iter()
            .copied()
            .find(|p| p.manifest.pack_id == pack)
    }

    /// The node version in force named by qualified `id`, with its package.
    fn node(&self, id: &str) -> Option<(&'a hir::Package, &'a hir::Node)> {
        let (pack, name) = id.split_once("::")?;
        let pkg = self.package(pack)?;
        hirx::nodes_at(self.all, pkg, &self.as_of, self.known_at.as_deref())
            .into_iter()
            .find(|n| n.meta.id == name)
            .map(|n| (pkg, n))
    }

    /// The deadline version selected at the timeline's dates for qualified
    /// `id`, with its package.
    fn deadline(&self, id: &str) -> Option<(&'a hir::Package, &'a hir::Deadline)> {
        let (pack, name) = id.split_once("::")?;
        let pkg = self.package(pack)?;
        hirx::selected(self.all, pkg, &self.as_of, self.known_at.as_deref())
            .into_iter()
            .find_map(|d| match d {
                Declaration::Deadline(v) if v.meta.id == name => Some((pkg, v)),
                _ => None,
            })
    }

    /// The deadline version (`id@from`) that counted `s`, with its package.
    fn counted_by(&self, s: &Scheduled) -> Option<(&'a hir::Package, &'a hir::Deadline)> {
        self.all.iter().copied().find_map(|p| {
            hirx::declarations(p).find_map(|d| match d {
                Declaration::Deadline(v)
                    if format!("{}@{}", v.meta.id, v.meta.effective.from) == s.version =>
                {
                    Some((p, v))
                }
                _ => None,
            })
        })
    }

    /// A deadline row of `world`, from declaration `decl` of `pkg`.
    #[allow(clippy::too_many_arguments)] // one row's every field
    fn deadline_row(
        &mut self,
        world: usize,
        pkg: &hir::Package,
        decl: &hir::Deadline,
        due: Option<String>,
        state: TimelineStatus,
        trigger: Option<usize>,
    ) -> usize {
        let layer = self.layer(pkg);
        // An instance is keyed by its trigger row; a deadline nothing has
        // triggered, by the event that would.
        let cause = trigger.map_or_else(
            || format!("event:{}", event_name(&decl.trigger)),
            |t| self.rows[t].key.clone(),
        );
        // The cite and the version citing it: the deadline's, else its
        // counting rule's.
        let cited = Some(&decl.meta)
            .filter(|m| m.cite.is_some())
            .or_else(|| hirx::rule_of(pkg, &decl.meta.id).map(|r| &r.meta));
        let row = Draft {
            key: format!("{}@{cause}", decl_key(pkg, &decl.meta.id)),
            world,
            kind: RowKind::Deadline,
            date: None,
            due,
            label: deadline_label(decl),
            status: state,
            cite: cited.and_then(|m| m.cite.clone()),
            source: self.source(decl.meta.span.as_ref()),
            layer,
            via: Vec::new(),
            trigger,
            replaced_by: None,
        };
        // No cite (no `cited`) has nothing to prove.
        let from = cited.map_or("", |m| m.effective.from.as_str());
        self.push_cited(row, pkg, from)
    }

    /// The `limit` row of `world` after event row `trigger`, whose event
    /// (`filed`, as written) files a paper kind: the version of the kind
    /// precedence selects over the matter's law on `date` (the event's,
    /// else `as_of` for an event a world assumes), its layer the one that
    /// set it and its provenance each layer selection passed through,
    /// base first. With none selected (no version in force, unordered
    /// claimants) the row is `unknown` with why, the base's layer
    /// deciding it. Nothing for an event that files no declared kind.
    fn limit_row(&mut self, world: usize, filed: &str, date: Option<&str>, trigger: usize) {
        let Some(kind) = paper::declared(self.all, filed) else {
            return;
        };
        let Some(base) = self.package(&kind.pack) else {
            return;
        };
        let on = date.map_or_else(|| self.as_of.clone(), str::to_owned);
        let law = self.law.clone();
        let key = format!("{}@{}", decl_key(base, &kind.name), self.rows[trigger].key);
        let draft = |layer: String| Draft {
            key: key.clone(),
            world,
            kind: RowKind::Limit,
            date: date.map(str::to_owned),
            due: None,
            label: kind.name.clone(),
            status: status(RowState::Unknown, None),
            cite: None,
            source: None,
            layer,
            via: Vec::new(),
            trigger: Some(trigger),
            replaced_by: None,
        };
        let set = match paper::governing(&law, &kind, &on, self.known_at.as_deref()) {
            Ok(set) => set,
            Err(why) => {
                let mut row = draft(self.layer(base));
                row.status.reason = Some(why);
                self.push(row);
                return;
            }
        };
        let mut layers: Vec<String> = Vec::new();
        for name in &set.provenance {
            if let Some(pkg) = self.all.iter().copied().find(|p| &p.manifest.name == name) {
                layers.push(self.layer(pkg));
            }
        }
        let Some(owner) = self
            .all
            .iter()
            .copied()
            .find(|p| p.manifest.name == set.package)
        else {
            return;
        };
        let layer = self.layer(owner);
        layers.retain(|l| *l != layer);
        let meta = &set.limit.meta;
        let row = Draft {
            label: format!("{}: {}", kind.name, limit_label(set.limit)),
            status: status(
                RowState::None,
                Some(format!("in force on {on}, set by {}", set.package)),
            ),
            cite: meta.cite.clone(),
            source: self.source(meta.span.as_ref()),
            via: layers,
            ..draft(layer)
        };
        self.push_cited(row, owner, &meta.effective.from);
    }

    /// Unknown rows in `world` for the deadlines governing `node`'s
    /// options that have no row yet among `seen` (qualified ids), each
    /// triggered by `trigger`.
    fn governing(
        &mut self,
        world: usize,
        pkg: &hir::Package,
        node: &hir::Node,
        seen: &mut Vec<String>,
        trigger: Option<usize>,
    ) {
        for edge in &node.edges {
            let Some(DeadlineWindow::Declared { deadline }) =
                edge.deadline.as_ref().map(|d| &d.window)
            else {
                continue;
            };
            let id = format!("{}::{}", pkg.manifest.pack_id, local(deadline));
            if seen.contains(&id) {
                continue;
            }
            seen.push(id.clone());
            if let Some((owner, decl)) = self.deadline(&id) {
                let reason = format!(
                    "counted from {}, which no recorded event dates as of {}",
                    hirx::event_text(&decl.trigger),
                    self.as_of
                );
                let row = self.deadline_row(
                    world,
                    owner,
                    decl,
                    None,
                    status(RowState::Unknown, Some(reason)),
                    trigger,
                );
                if trigger.is_none() {
                    self.untriggered.push((row, id));
                }
            }
        }
    }

    /// The timeline.
    fn finish(self, source: TimelineSource, mut diagnostics: Vec<TimelineDiagnostic>) -> Timeline {
        diagnostics.extend(self.proof_notes());
        // Each world's rows in date order (undated last), creation order
        // breaking ties; worlds in order.
        let mut order: Vec<usize> = (0..self.rows.len()).collect();
        order.sort_by_key(|&i| {
            let r = &self.rows[i];
            let date = r.date.clone().or_else(|| r.due.clone());
            (r.world, date.is_none(), date, i)
        });
        let mut ids = vec![String::new(); self.rows.len()];
        for (n, &i) in order.iter().enumerate() {
            ids[i] = format!("r{}", n + 1);
        }
        let world_id = |w: usize| self.worlds[w].id.clone();
        let mut rows = Vec::new();
        let mut edges = Vec::new();
        let mut last: BTreeMap<usize, usize> = BTreeMap::new();
        for &i in &order {
            let r = &self.rows[i];
            let class = self
                .layers
                .iter()
                .find(|l| l.id == r.layer)
                .map_or(LayerClass::BaseRule, |l| l.class);
            let link = |from: usize, kind| TimelineEdge {
                from: ids[from].clone(),
                to: ids[i].clone(),
                world: world_id(r.world),
                kind,
            };
            if let Some(&prev) = last.get(&r.world).filter(|&&p| r.trigger != Some(p)) {
                edges.push(link(prev, EdgeKind::Sequence));
            }
            if let Some(t) = r.trigger {
                let kind = if self.rows[t].world == r.world {
                    EdgeKind::Triggers
                } else {
                    EdgeKind::Forks
                };
                edges.push(link(t, kind));
            }
            for &(from, _) in self.supersedes.iter().filter(|&&(_, to)| to == i) {
                edges.push(link(from, EdgeKind::Supersedes));
            }
            last.insert(r.world, i);
            rows.push(TimelineRow {
                id: ids[i].clone(),
                key: r.key.clone(),
                world: world_id(r.world),
                kind: r.kind,
                date: r.date.clone(),
                due: r.due.clone(),
                label: r.label.clone(),
                status: r.status.clone(),
                cite: r.cite.clone(),
                source: r.source.clone(),
                layer: class,
                provenance: r.via.iter().chain([&r.layer]).cloned().collect(),
                trigger: r.trigger.map(|t| ids[t].clone()),
                replaced_by: r.replaced_by.map(|x| self.rows[x].key.clone()),
            });
        }
        let worlds = self
            .worlds
            .into_iter()
            .zip(&self.forks)
            .map(|(mut w, f)| {
                w.fork_row = f.map(|f| ids[f].clone());
                w
            })
            .collect();
        Timeline {
            schema: TIMELINE_SCHEMA.to_owned(),
            source,
            as_of: self.as_of,
            known_at: self.known_at,
            layers: self.layers,
            worlds,
            rows,
            edges,
            diagnostics,
        }
    }
}
