
### `fed-dariz/arizona-lrciv.txt` — D. Ariz. Local Rules of Civil Procedure

- Source: https://www.azd.uscourts.gov/sites/azd/files/local-rules/LRCiv%202025.pdf (official site; 145 pages, sha256 614b8b1b8ffa83f23bca8d672c62e1526006f07dd5557dafde38a54256663984).
- Edition: D. Ariz. Local Rules of Civil Procedure (LRCiv), 2025-2026 edition, effective 12/01/2025, as listed current on https://www.azd.uscourts.gov/local-rules. Fetched with curl (robots.txt crawl-delay 10; one request).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 dd1ae5357e7186ef7b0d725b8e3c3748835795db302b3536a59f5483569e9b5c. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-dariz/original/arizona-lrciv.pdf` as served (sha256 614b8b1b8ffa83f23bca8d672c62e1526006f07dd5557dafde38a54256663984); extraction sha256 dd1ae5357e7186ef7b0d725b8e3c3748835795db302b3536a59f5483569e9b5c.

### `fed-dariz/azd-lrcrim-2025-2026.txt` — D. Ariz. Local Rules of Criminal Procedure (LRCrim, 2025-2026)

- Source: https://www.azd.uscourts.gov/sites/azd/files/local-rules/LRCRIM%202025.pdf (official site; 88 pages, sha256 2114eac2c26cd8b5ec8db71ae0ca79589fccff58e450df98806a5de61d8b1964).
- Edition: Local Rules of Criminal Procedure (2025-2026) as listed at https://www.azd.uscourts.gov/local-rules (the PDF states no separate effective date). robots.txt of www.azd.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): azd-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 613d9fe4cae662fefbd81bc61a29bfea4c746c76b1ac8488bcfe03c20833c782. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-dariz/original/azd-lrcrim-2025-2026.pdf` as served (sha256 2114eac2c26cd8b5ec8db71ae0ca79589fccff58e450df98806a5de61d8b1964); extraction sha256 613d9fe4cae662fefbd81bc61a29bfea4c746c76b1ac8488bcfe03c20833c782.

### `fed-dariz/arizona-lrciv-sectioned.txt` — D. Ariz. LRCiv, sectioned

- Derived in the pod (lg-f3q8.37) from `fed-dariz/arizona-lrciv.txt` (extract sha256 dd1ae5357e7186ef7b0d725b8e3c3748835795db302b3536a59f5483569e9b5c); nothing fetched. One `## LRCiv <n>` line is inserted before each rule's own heading line (`LRCiv 1.1` through `LRCiv 83.10`, 67 rules). Rule text was not edited: deleting the marker lines (`perl -ne 'print unless /^## LRCiv /'`) restores the extract byte for byte.
- Edition: as the extract, 2025-2026 edition, effective 12/01/2025.
- Sectioned sha256 0da384ed858ac43637c3ab1cc134d4294aa2575d7a72cd14cc53157198e5c365 (the `path_sha256` pinned by `lit/azd/Lit.toml`).
