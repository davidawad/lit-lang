# `arkctapp` sources: provenance

### `arkctapp/arap-civil.txt` — Arkansas Rules of Appellate Procedure—Civil

- Source: https://opinions.arcourts.gov/ark/cr/en/1885/1/document.do (official court site; 22 pages, sha256 c854fb5600e8cb1595791154a4befca952fd984ae346f1e4aacf45672923d36e).
- Edition: title: "Rules of Appellate Procedure—Civil (current through October 1, 2026)"; listed current to 10/01/2026 on https://opinions.arcourts.gov/ark/cr/en/nav_date.do.
- Retrieved 2026-10-27 on carbon (the court site refuses agent pods: lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 baa4d8fdce843d9f59ad162212be7de918cd3ab5566e6ae9c6a2bbdf530dfdda. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `arkctapp/original/arap-civil.pdf` as served (sha256 c854fb5600e8cb1595791154a4befca952fd984ae346f1e4aacf45672923d36e); extraction sha256 baa4d8fdce843d9f59ad162212be7de918cd3ab5566e6ae9c6a2bbdf530dfdda.

### `arkctapp/ar-supreme-court-rules.txt` — Rules of the Supreme Court and Court of Appeals of the State of Arkansas

- Source: https://opinions.arcourts.gov/ark/cr/en/1871/1/document.do (official court site; 44 pages, sha256 7f26989c58f09c36964c5add5424f6709db177bfc367ea648aab2905b88c4484).
- Edition: title: "Rules of the Supreme Court and Court of Appeals of the State of Arkansas (current through December 15, 2025)"; listed current to 12/15/2025 on https://opinions.arcourts.gov/ark/cr/en/nav_date.do.
- Retrieved 2026-10-27 on carbon (the court site refuses agent pods: lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 039cb6ae5df1eeb38a0be219ebf093b41622768649397394a82652bbaf3c3052. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `arkctapp/original/ar-supreme-court-rules.pdf` as served (sha256 7f26989c58f09c36964c5add5424f6709db177bfc367ea648aab2905b88c4484); extraction sha256 039cb6ae5df1eeb38a0be219ebf093b41622768649397394a82652bbaf3c3052.

### `arkctapp/arap-civil-sectioned.txt` — Arkansas Rules of Appellate Procedure—Civil, sectioned

- Derived in the pod (lg-f3q8.3) from `arkctapp/arap-civil.txt` (extract sha256 baa4d8fdce843d9f59ad162212be7de918cd3ab5566e6ae9c6a2bbdf530dfdda); nothing fetched. One `## Ark. R. App. P.-Civ. Rule <n>` line is inserted before the first heading line `Rule <n>. <Title>` of each rule (Rule 1 through Rule 12; the second, paper-record heading of Rule 7 is unmarked). The marker names the body so that its headings differ from the `## Rule <n>` headings of `ar-circuit/arcp-sectioned.txt`, which `lit/ar-circuit` binds beside it. Rule text was not edited: deleting the marker lines (`sed '/^## Ark. R. App. P.-Civ. Rule /d'`) restores the extract byte for byte.
- Upstream: https://opinions.arcourts.gov/ark/cr/en/1885/1/document.do, retrieved 2026-10-27, as recorded on the `arkctapp/arap-civil.txt` entry this file is derived from.
- Edition: as the extract, current through October 1, 2026.
- Sectioned sha256 a8bd615e4e462ea7c51569640464bf46d545c4a31b8b0479cb341d6225cb0ef8 (the `path_sha256` pinned by `lit/ar-circuit/Lit.toml` and `lit/arkctapp/Lit.toml`).

### `arkctapp/ar-supreme-court-rules-sectioned.txt` — Rules of the Supreme Court and Court of Appeals of the State of Arkansas, sectioned

- Derived in the pod (lg-f3q8.3) from `arkctapp/ar-supreme-court-rules.txt` (extract sha256 039cb6ae5df1eeb38a0be219ebf093b41622768649397394a82652bbaf3c3052); nothing fetched. One `## Ark. Sup. Ct. R. <n>-<m>` line is inserted before each rule's own heading line `Rule <n>-<m>. <Title>` (Rule 1-1 through Rule 6-10, 40 rules); body lines that merely cite a rule are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Ark. Sup. Ct. R. /d'`) restores the extract byte for byte.
- Upstream: https://opinions.arcourts.gov/ark/cr/en/1871/1/document.do, retrieved 2026-10-27, as recorded on the `arkctapp/ar-supreme-court-rules.txt` entry this file is derived from.
- Edition: as the extract, current through December 15, 2025.
- Sectioned sha256 12010c2bee197d0f5ecef0c430a1c8559819a85152d0396c83551562bfe639ca (the `path_sha256` pinned by `lit/arkctapp/Lit.toml`).
