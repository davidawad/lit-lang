# Audit: lg-zmqc.12 timeline expectation refresh

The five snapshot/test changes in `f9ea7e7c2f94901e1be55aebc885af8789f2662a`
(`test(litigate-cli): refresh timeline expectations (lg-zmqc.12)`) were reviewed
against the commits that changed timeline output and the vendored sources. The
snapshot diff records the refresh commit; each section below identifies the
upstream output change as well.

## `a_cite_an_amendment_history_proves_names_it_as_the_evidence`

- **Output-changing commits:** `4fd222e3f9f972de18340c16bca1820a7c260dfe`
  (`wip(litigate): a cite whose vendored text postdates its version is a hole
  unless the eCFR amendment history proves it (lg-flu.40)`) introduced the
  history proof; `8b194236` (`feat(litigate): a timeline names the amendment
  history proving each row's cite, an L9008 note (lg-flu.40)`) surfaced it in
  timeline rows. `git log -S` on the changed assertion/snapshot row identifies
  `f9ea7e7c2` as the expectation refresh.
- **Why the new result is correct:** the cited discovery text takes effect
  2017-07-21. The pinned version history, `sources/ecfr-versions-title-37.txt`
  lines 113–115, records § 2.120 versions on 2017-01-01, 2017-01-14, and
  2017-07-21, with the latter as the latest recorded version. This supports
  the L9008 note that the cited text is in force from 2017-07-21. The test
  correctly expects that evidence instead of an L9002 hole.

## `timeline__frcp_filed_fixture`

- **Output-changing commits:** `8b194236` added the timeline's L9008
  amendment-history note. `63c43f25` (`fix(lit): clear the 31 strict holes
  lit/links inherits from frcp, itc-337 and links.lit (lg-8j3a)`) updated FRCP
  declarations and source locations represented by this fixture. The snapshot
  diff is in `f9ea7e7c2` (`git log -S` on the new Rule 4 note identifies that
  refresh).
- **Why the new result is correct:** `sources/rule-amendment-history.txt`
  lines 109–111 records the Rule 4 version effective 2017-12-01 and the
  subsequent version history through 2025-01-06. The fixture's Rule 4(m)
  declaration is effective from 2017-12-01 (`lit/frcp/periods.lit`, line 181),
  so its L9008 note and updated source span identify the cited rule version
  and its evidence. FRCP 12(a)(1)'s responsive-pleading periods remain the
  21-day and 60-day periods stated in `sources/frcp.txt` lines 975–985; the
  snapshot refresh changes provenance, not those periods.

## `timeline__reset_absorbed_period`

- **Output-changing commits:** `8b194236` emits the amendment-history notes;
  `63c43f25` updates FRCP declaration/source locations, and `42c7d2b2`
  (`fix(lit): name authorities in extension locks (lg-962.12)`) updates the
  FRAP layer's source declarations. `f9ea7e7c2` is the `git log -S` snapshot
  refresh for the changed source spans and qualified FRAP cite.
- **Why the new result is correct:** the displayed cross-appeal cite is
  explicitly `FRAP (2009 ed.) 4(a)(3)`, matching the authored post-2009 rule
  version; the vendored text at `sources/usc-28-app-frap-editions.txt` lines
  217–225 gives Rule 4(a)(3)'s 14-day cross-appeal period and its later-of
  alternative. Other displayed postjudgment clocks cite FRCP Rules 50, 59,
  and 60; `sources/frcp.txt` lines 4006–4025, 4520–4539, and 4556–4580 contain
  the corresponding rule provisions, and `sources/rule-amendment-history.txt`
  lines 129–147 records their relevant versions. The refreshed spans point to
  the current declarations; the expected deadlines and status are unchanged.

## `timeline__reset_restarted_period`

- **Output-changing commit:** `42c7d2b2` (`fix(lit): name authorities in
  extension locks (lg-962.12)`) updated the FRAP declaration/source location;
  `f9ea7e7c2` is the `git log -S` snapshot refresh for those offsets.
- **Why the new result is correct:** the rows identify `28 U.S.C. § 2107(a)`
  and `(b)`. The vendored statute at `sources/usc-additional.txt` lines
  152–165 sets a 30-day notice period generally and 60 days when the United
  States and specified federal parties are involved. The snapshot follows the
  shifted declaration at `lit/frap/appeal-periods.lit` line 115; the source
  span moved, while the statutory periods did not change.

## `timeline__ttab_opposition_matter`

- **Output-changing commits:** `b41d63e64ceaec7cd9e2b6a2bfeb21a8c40876ca`
  (`fix(lit): clear the 126 strict holes lit/ptab inherits from ttab, cafc and
  mpep (lg-bxy1)`) moves the TTAB package's supported start to 2026-06-01;
  `8b194236` supplies the history-based L9008 notes. `f9ea7e7c2` is the
  `git log -S` snapshot refresh for the package date and note text.
- **Why the new result is correct:** the source corpus is the June 2026 TBMP
  edition. `sources/PROVENANCE.md` lines 1563–1575 identifies that edition and
  the TTAB sections used; lines 1332–1352 document comparison of the June
  2024, June 2025, and June 2026 editions. `sources/tbmp-versions.txt` lines
  1–16 records the edition dates for cited §§ 112, 113.05, 207.03, and 212;
  the per-section entries distinguish sections whose text changed in 2026
  from those last changed in 2025. The 2026-06-01 package boundary and the
  L9008 notes therefore reflect the vendored source coverage and its recorded
  section histories.

## `timeline__ttab_published_fixture`

- **Output-changing commits:** `b41d63e64ceaec7cd9e2b6a2bfeb21a8c40876ca`
  (`fix(lit): clear the 126 strict holes lit/ptab inherits from ttab, cafc and
  mpep (lg-bxy1)`) moves the TTAB package's supported start to 2026-06-01;
  `8b194236` supplies history-based L9008 notes. `f9ea7e7c2` is the
  `git log -S` snapshot refresh for the fixture's effective date and notes.
- **Why the new result is correct:** this fixture uses the same TTAB package
  source edition as the opposition matter. `sources/PROVENANCE.md` lines
  1563–1575 identifies the June 2026 TBMP source and cited sections, while
  lines 1332–1352 explain the edition comparison and dates. The matching
  entries in `sources/tbmp-versions.txt` lines 1–16 record the effective
  versions represented. Its 2026-06-01 package date and amendment-history
  notes are thus consistent with the pinned material; no fixture deadline
  was altered by this refresh.

DECIDED: commit this audit as a repository document -- the PM directed the pod to record the five justifications here because bead comments cannot be posted from the pod.
