
### `bankr-inn/innb-lbr-2026-10-27.txt` — Local Rules of the U.S. Bankruptcy Court for the Northern District of Indiana (cover dated October 27, 2026)

- Source: https://www.innb.uscourts.gov/sites/innb/files/LocalRules.pdf (official site; 126 pages, sha256 3e4b79468488c0a1d2373ba6545775f02cfa5b5781f44b19e65bf12415b9f110).
- Edition: Local Rules, cover dated 'October 27, 2026' (the day of retrieval), the 'Print Friendly Version (PDF)' of https://www.innb.uscourts.gov/court-info/local-rules-and-orders/local-rules; no separate effective date on the cover. robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 e7a4bfce60626c4de030e80eac96c162a3f15214811fe4cd918c7648e5ad495f. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-inn/original/innb-lbr-2026-10-27.pdf` as served (sha256 3e4b79468488c0a1d2373ba6545775f02cfa5b5781f44b19e65bf12415b9f110); extraction sha256 e7a4bfce60626c4de030e80eac96c162a3f15214811fe4cd918c7648e5ad495f.

### `bankr-inn/innb-lbr-2026-10-27-sectioned.txt` — N.D. Ind. Local Rules, sectioned

- Derived in the pod (lg-u561.21) from `bankr-inn/innb-lbr-2026-10-27.txt` (extract sha256 e7a4bfce60626c4de030e80eac96c162a3f15214811fe4cd918c7648e5ad495f); nothing fetched. One `## LBR <n>` line is inserted before each rule's centred body heading `B-<n>` (after the table of contents; the court's `B-` prefix dropped, so `LBR 1001-1` heads rule B-1001-1, through `LBR 9070-1`, with `LBR 3002.1-1` and `LBR 3017.1-1` keeping their points; 84 sections, abrogated rules included with their notes); the cover, the Publication Page of amending orders and the table of contents are unmarked, and each local form stays inside the section of the rule it follows. Rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Edition: as the extract, the consolidation with cover date October 27, 2026; its latest amending orders are dated October 26, 2026 (B-2016-1, B-9011-1).
- Sectioned sha256 5af35eecc69d593c1b4d2619d7e35a28ba006ba8c22f10d9f0ed34a99034a95f (the `path_sha256` pinned by `lit/bankr-inn/Lit.toml` for source `innb-lbr`).
