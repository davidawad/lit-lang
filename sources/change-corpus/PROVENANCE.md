# Change corpus — provenance

Every file under `sources/change-corpus/`, with its source URL, retrieval
date, method and sha256, in the form `scripts/ci-scoped provenance` checks
(lg-h46r). The two `MANIFEST.json` files are the full machine-readable
record (pin cites, pending-case language, holdings, derived fields); this
file restates their provenance fields per file and records the manifests
and the files they leave out. `rules/README.md` and `rulings/README.md`
describe the collection (lg-fq2.2, lg-fq2.3). Everything here is a
government publication or a court opinion; no client material.

## Manifests

### `rules/MANIFEST.json`

- Written 2026-10-23 by lg-fq2.2 while the artefacts below were downloaded
  (govinfo.gov, supremecourt.gov, uscourts.gov, e.g.
  <https://www.govinfo.gov/content/pkg/USREPORTS-308/pdf/USREPORTS-308.pdf>);
  retrieved 2026-10-23 is the download date of every artefact it lists.
- Method: compiled by hand from the downloaded documents; one entry per
  artefact with its source URL, `retrieved_at` and sha256.

- sha256: `f8b81b4143ac87e35362b759acf14704c784386020ee606931a972f11d6bbbf4`.

### `rulings/MANIFEST.json`

- Written 2026-10-23 (`generated_utc` 2026-10-23T08:48:41Z) by lg-fq2.3
  from the opinions below (Caselaw Access Project static files, e.g.
  <https://static.case.law/f2d/518/html/0749-01.html>, and CourtListener
  storage); retrieved 2026-10-23 is the download date of every opinion it
  lists.
- Method: compiled by hand from the downloaded opinions; one entry per
  ruling with its source URL, `retrieved_at` and the sha256 of every
  stored file.

- sha256: `a69830f0847cccc2ad385a4b96f74620bf0b7ed4a13bcefa25e12ba17d316c00`.

## Rules: FRCP 6 / FRAP 26 version history (`rules/`)

### `rules/us-reports/ur308-civ-1938-order.txt`

- Supreme Court order adopting FRCP 6 (1938 amendment), U.S. Reports vol. 308 (adopting-order, FRCP 6, effective 1938-09-16).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-308/pdf/USREPORTS-308.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:27:29Z). Downloaded the source PDF (not stored: 141063615 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 717-724,838-838 of USREPORTS-308.pdf).
- `us-reports/ur308-civ-1938-order.txt` sha256: `e8cf76f3231d74681e423fbbee8db1129f05a266cb318f058ef6ebaaa1b80c32`.
- Source document sha256: `180cb7e5e277f8acaecaa2feded8a6d0a94b7c69f4c0a30747a655d944d1456a`.
- Manifest id `ur308-civ-1938-order`.

### `rules/us-reports/ur308-civ-1938-text.txt`

- Amended text of FRCP 6 (1938 amendment), U.S. Reports vol. 308 (amendment-text, FRCP 6, effective 1938-09-16).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-308/pdf/USREPORTS-308.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:27:29Z). Downloaded the source PDF (not stored: 141063615 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 740-742 of USREPORTS-308.pdf).
- `us-reports/ur308-civ-1938-text.txt` sha256: `eb178c5d933ad53e622429b2e7dd162a59b6748baa4b1d31c7d4f5bf945fd4bd`.
- Source document sha256: `180cb7e5e277f8acaecaa2feded8a6d0a94b7c69f4c0a30747a655d944d1456a`.
- Manifest id `ur308-civ-1938-text`.

### `rules/us-reports/ur329-civ-1946-order.txt`

- Supreme Court order adopting FRCP 6 (1946 amendment), U.S. Reports vol. 329 (adopting-order, FRCP 6, effective 1948-03-19).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-329/pdf/USREPORTS-329.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:27:55Z). Downloaded the source PDF (not stored: 70787873 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 933-937,969-969 of USREPORTS-329.pdf).
- `us-reports/ur329-civ-1946-order.txt` sha256: `6f7e42b2186cec6f9f6a306324d9995c81becabd51b44408a0aa0710f124fc27`.
- Source document sha256: `d36cb90b0b8aec1919e02c7f3cbc3a97cab9cdd21c97ca445f07294febc0c276`.
- Manifest id `ur329-civ-1946-order`.

### `rules/us-reports/ur329-civ-1946-text.txt`

- Amended text of FRCP 6 (1946 amendment), U.S. Reports vol. 329 (amendment-text, FRCP 6, effective 1948-03-19).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-329/pdf/USREPORTS-329.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:27:55Z). Downloaded the source PDF (not stored: 70787873 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 941-942 of USREPORTS-329.pdf).
- `us-reports/ur329-civ-1946-text.txt` sha256: `f3d76f20c747460a6e19500d463cc7eb68c0fd33ba8788935ac0625cb8e5d70d`.
- Source document sha256: `d36cb90b0b8aec1919e02c7f3cbc3a97cab9cdd21c97ca445f07294febc0c276`.
- Manifest id `ur329-civ-1946-text`.

### `rules/us-reports/ur374-civ-1963-order.txt`

- Supreme Court order adopting FRCP 6 (1963 amendment), U.S. Reports vol. 374 (adopting-order, FRCP 6, effective 1963-07-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-374/pdf/USREPORTS-374.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:45Z). Downloaded the source PDF (not stored: 41624239 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 631-635,641-642 of USREPORTS-374.pdf).
- `us-reports/ur374-civ-1963-order.txt` sha256: `e4f8e2045a4d8d6c18613429ceb8639afad123f6920a1041df92c5e9409b4f35`.
- Source document sha256: `a8c8e4f44720abbe020b5cbe71122b8b9c063230ed80ac6ae172025b8b04e932`.
- Manifest id `ur374-civ-1963-order`.

### `rules/us-reports/ur374-civ-1963-text.txt`

- Amended text of FRCP 6 (1963 amendment), U.S. Reports vol. 374 (amendment-text, FRCP 6, effective 1963-07-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-374/pdf/USREPORTS-374.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:45Z). Downloaded the source PDF (not stored: 41624239 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 648-649 of USREPORTS-374.pdf).
- `us-reports/ur374-civ-1963-text.txt` sha256: `ac1b024237559d6b1ece30ec9b44a84df3ee2696c47a5b6356be92260c407cad`.
- Source document sha256: `a8c8e4f44720abbe020b5cbe71122b8b9c063230ed80ac6ae172025b8b04e932`.
- Manifest id `ur374-civ-1963-text`.

### `rules/us-reports/ur389-civ-1967-order.txt`

- Supreme Court order adopting FRCP 6 (1967 amendment), U.S. Reports vol. 389 (adopting-order, FRCP 6, effective 1968-07-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-389/pdf/USREPORTS-389.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:58Z). Downloaded the source PDF (not stored: 64076490 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 969-971 of USREPORTS-389.pdf).
- `us-reports/ur389-civ-1967-order.txt` sha256: `700a2b8ce733e0a34b65e35a9b89a60c19178c0585f31c66edeb7dcd4e23e802`.
- Source document sha256: `9666486d9ef1f197affe3f0fdee04e93a955db53c77e6182feff26319f786078`.
- Manifest id `ur389-civ-1967-order`.

### `rules/us-reports/ur389-civ-1967-text.txt`

- Amended text of FRCP 6 (1967 amendment), U.S. Reports vol. 389 (amendment-text, FRCP 6, effective 1968-07-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-389/pdf/USREPORTS-389.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:58Z). Downloaded the source PDF (not stored: 64076490 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1025-1028 of USREPORTS-389.pdf).
- `us-reports/ur389-civ-1967-text.txt` sha256: `0eae11a2d238d7e4e68b306c5960272ba88238a1cd3b638730e17c09e60149cf`.
- Source document sha256: `9666486d9ef1f197affe3f0fdee04e93a955db53c77e6182feff26319f786078`.
- Manifest id `ur389-civ-1967-text`.

### `rules/us-reports/ur401-civ-1971-order.txt`

- Supreme Court order adopting FRCP 6 (1971 amendment), U.S. Reports vol. 401 (adopting-order, FRCP 6, effective 1971-07-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-401/pdf/USREPORTS-401.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:24:25Z). Downloaded the source PDF (not stored: 266184147 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1061-1064 of USREPORTS-401.pdf).
- `us-reports/ur401-civ-1971-order.txt` sha256: `381b08b4f1ddf4eb996d9cb15fedb3e04a7f8f06bb944a8724fc1e3640c1ed0a`.
- Source document sha256: `118fe75ce71f051c46e8000a56f4dbaf0d9290b39f045ba2f265a37527537130`.
- Manifest id `ur401-civ-1971-order`.

### `rules/us-reports/ur401-civ-1971-text.txt`

- Amended text of FRCP 6 (1971 amendment), U.S. Reports vol. 401 (amendment-text, FRCP 6, effective 1971-07-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-401/pdf/USREPORTS-401.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:24:25Z). Downloaded the source PDF (not stored: 266184147 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1065-1066 of USREPORTS-401.pdf).
- `us-reports/ur401-civ-1971-text.txt` sha256: `1de8fe3aadf1c2a10a03343c9a129ee6b8823095bd5ef6bb24f0a928143f177f`.
- Source document sha256: `118fe75ce71f051c46e8000a56f4dbaf0d9290b39f045ba2f265a37527537130`.
- Manifest id `ur401-civ-1971-text`.

### `rules/us-reports/ur461-civ-1983-order.txt`

- Supreme Court order adopting FRCP 6 (1983 amendment), U.S. Reports vol. 461 (adopting-order, FRCP 6, effective 1983-08-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-461/pdf/USREPORTS-461.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:24:01Z). Downloaded the source PDF (not stored: 71556877 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1057-1060 of USREPORTS-461.pdf).
- `us-reports/ur461-civ-1983-order.txt` sha256: `5578a03b3fa6c76fde5e67758629124bbbe277338170ef1ef88f6e484eca9bcc`.
- Source document sha256: `46d7d267b6c534d6cbacbebb4c0dada42ea18a4a4c11e408ade62f36acdc5b1e`.
- Manifest id `ur461-civ-1983-order`.

### `rules/us-reports/ur461-civ-1983-text.txt`

- Amended text of FRCP 6 (1983 amendment), U.S. Reports vol. 461 (amendment-text, FRCP 6, effective 1983-08-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-461/pdf/USREPORTS-461.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:24:01Z). Downloaded the source PDF (not stored: 71556877 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1061-1062 of USREPORTS-461.pdf).
- `us-reports/ur461-civ-1983-text.txt` sha256: `c8508408976d226d498966363476c35e9366dfed4ea520dcebd7c69b9730d06a`.
- Source document sha256: `46d7d267b6c534d6cbacbebb4c0dada42ea18a4a4c11e408ade62f36acdc5b1e`.
- Manifest id `ur461-civ-1983-text`.

### `rules/us-reports/ur471-civ-1985-order.txt`

- Supreme Court order adopting FRCP 6 (1985 amendment), U.S. Reports vol. 471 (adopting-order, FRCP 6, effective 1985-08-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-471/pdf/USREPORTS-471.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:22:34Z). Downloaded the source PDF (not stored: 77994595 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1095-1098 of USREPORTS-471.pdf).
- `us-reports/ur471-civ-1985-order.txt` sha256: `e61371688fe81698acd96719b4c8039bc68949d13d7e4e4628d1e7f8bb4b4ce4`.
- Source document sha256: `dbb84e615d64cb6b117caab5108c03e1ad580ab2b02fb817c2e7c7aa6c2af328`.
- Manifest id `ur471-civ-1985-order`.

### `rules/us-reports/ur471-civ-1985-text.txt`

- Amended text of FRCP 6 (1985 amendment), U.S. Reports vol. 471 (amendment-text, FRCP 6, effective 1985-08-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-471/pdf/USREPORTS-471.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:22:34Z). Downloaded the source PDF (not stored: 77994595 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1099-1100 of USREPORTS-471.pdf).
- `us-reports/ur471-civ-1985-text.txt` sha256: `b9f4b9f317da5e9e3c0981429f57ea5062b905237f58310d1d8462b76c2e89ac`.
- Source document sha256: `dbb84e615d64cb6b117caab5108c03e1ad580ab2b02fb817c2e7c7aa6c2af328`.
- Manifest id `ur471-civ-1985-text`.

### `rules/us-reports/ur480-civ-1987-order.txt`

- Supreme Court order adopting FRCP 6 (1987 amendment), U.S. Reports vol. 480 (adopting-order, FRCP 6, effective 1987-08-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-480/pdf/USREPORTS-480.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:24:26Z). Downloaded the source PDF (not stored: 290353395 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 857-859 of USREPORTS-480.pdf).
- `us-reports/ur480-civ-1987-order.txt` sha256: `35a3108591150d6d8f06bfc1d7adad636ecdaac26d62dd2375ed067b5e20f000`.
- Source document sha256: `1749b05f8071b3d5db04dd174ab2996c1aeda581b940cc83983e65484e7e85ca`.
- Manifest id `ur480-civ-1987-order`.

### `rules/us-reports/ur480-civ-1987-text.txt`

- Amended text of FRCP 6 (1987 amendment), U.S. Reports vol. 480 (amendment-text, FRCP 6, effective 1987-08-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-480/pdf/USREPORTS-480.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:24:26Z). Downloaded the source PDF (not stored: 290353395 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 864-864 of USREPORTS-480.pdf).
- `us-reports/ur480-civ-1987-text.txt` sha256: `3249ab9859316b3a0cf31fbd19d1dd40d34ca8ab218e4b47a14ae25e881a9162`.
- Source document sha256: `1749b05f8071b3d5db04dd174ab2996c1aeda581b940cc83983e65484e7e85ca`.
- Manifest id `ur480-civ-1987-text`.

### `rules/us-reports/ur526-civ-1999-order.txt`

- Supreme Court order adopting FRCP 6 (1999 amendment), U.S. Reports vol. 526 (adopting-order, FRCP 6, effective 1999-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-526/pdf/USREPORTS-526.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:35Z). Downloaded the source PDF (not stored: 5543500 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1174-1176 of USREPORTS-526.pdf).
- `us-reports/ur526-civ-1999-order.txt` sha256: `8ab7743444b8b5079b028af46e40f4d029b8be2569bea16c942bd361b2eebe39`.
- Source document sha256: `c41b80ba744b25902fd7e617daa888d614a76fab470acb079aafc9c2a3cfa514`.
- Manifest id `ur526-civ-1999-order`.

### `rules/us-reports/ur526-civ-1999-text.txt`

- Amended text of FRCP 6 (1999 amendment), U.S. Reports vol. 526 (amendment-text, FRCP 6, effective 1999-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-526/pdf/USREPORTS-526.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:35Z). Downloaded the source PDF (not stored: 5543500 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1177-1178 of USREPORTS-526.pdf).
- `us-reports/ur526-civ-1999-text.txt` sha256: `cd893afb0c05bed74b47f024f58fb5dba4fb5525bd65fc8bede27a12259046b4`.
- Source document sha256: `c41b80ba744b25902fd7e617daa888d614a76fab470acb079aafc9c2a3cfa514`.
- Manifest id `ur526-civ-1999-text`.

### `rules/us-reports/ur532-civ-2001-order.txt`

- Supreme Court order adopting FRCP 6 (2001 amendment), U.S. Reports vol. 532 (adopting-order, FRCP 6, effective 2001-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-532/pdf/USREPORTS-532.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:32Z). Downloaded the source PDF (not stored: 5783892 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1138-1140 of USREPORTS-532.pdf).
- `us-reports/ur532-civ-2001-order.txt` sha256: `6fdb0e3c13e616456170fa671259ca8064c97976fc3b332466d6478aff8b6d2b`.
- Source document sha256: `d31728b595f4712d9700e60871cda3f5c7490360c969aa0aecf5a1b2fe80bcb4`.
- Manifest id `ur532-civ-2001-order`.

### `rules/us-reports/ur532-civ-2001-text.txt`

- Amended text of FRCP 6 (2001 amendment), U.S. Reports vol. 532 (amendment-text, FRCP 6, effective 2001-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-532/pdf/USREPORTS-532.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:32Z). Downloaded the source PDF (not stored: 5783892 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1141-1143 of USREPORTS-532.pdf).
- `us-reports/ur532-civ-2001-text.txt` sha256: `05957cafd5e798416c08542cb3f89a77c5b2518a916543f838ed29be4ca58ca0`.
- Source document sha256: `d31728b595f4712d9700e60871cda3f5c7490360c969aa0aecf5a1b2fe80bcb4`.
- Manifest id `ur532-civ-2001-text`.

### `rules/us-reports/ur544-civ-2005-order.txt`

- Supreme Court order adopting FRCP 6 (2005 amendment), U.S. Reports vol. 544 (adopting-order, FRCP 6, effective 2005-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-544/pdf/USREPORTS-544.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:33Z). Downloaded the source PDF (not stored: 3714571 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1102-1104 of USREPORTS-544.pdf).
- `us-reports/ur544-civ-2005-order.txt` sha256: `b725d5126a971b369bf717573f313cacf2cf88d32f3ab26a12a9cc8a00f9d477`.
- Source document sha256: `e40e6385620c128442e72cd92a0ac719692a4e3496ec2ed057d3002122b17de3`.
- Manifest id `ur544-civ-2005-order`.

### `rules/us-reports/ur544-civ-2005-text.txt`

- Amended text of FRCP 6 (2005 amendment), U.S. Reports vol. 544 (amendment-text, FRCP 6, effective 2005-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-544/pdf/USREPORTS-544.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:33Z). Downloaded the source PDF (not stored: 3714571 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1105-1106 of USREPORTS-544.pdf).
- `us-reports/ur544-civ-2005-text.txt` sha256: `287814366865dce0d4b4461d856d61918645b7b6d065cb6e5855176e581ea37f`.
- Source document sha256: `e40e6385620c128442e72cd92a0ac719692a4e3496ec2ed057d3002122b17de3`.
- Manifest id `ur544-civ-2005-text`.

### `rules/us-reports/ur550-civ-2007-order.txt`

- Supreme Court order adopting FRCP 6 (2007 amendment), U.S. Reports vol. 550 (adopting-order, FRCP 6, effective 2007-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-550/pdf/USREPORTS-550.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:32Z). Downloaded the source PDF (not stored: 4293323 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 811-813 of USREPORTS-550.pdf).
- `us-reports/ur550-civ-2007-order.txt` sha256: `cb121e0dd361ae9b01b66ff8376981bac6a5cd869da123555f63efd3c428062a`.
- Source document sha256: `97c8620819021002a27ee5b1b7fa5a4c65690cbc4bc4cffca14fd71121785a2b`.
- Manifest id `ur550-civ-2007-order`.

### `rules/us-reports/ur550-civ-2007-text.txt`

- Amended text of FRCP 6 (2007 amendment), U.S. Reports vol. 550 (amendment-text, FRCP 6, effective 2007-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-550/pdf/USREPORTS-550.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:32Z). Downloaded the source PDF (not stored: 4293323 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 827-829 of USREPORTS-550.pdf).
- `us-reports/ur550-civ-2007-text.txt` sha256: `f0d6b561eedacfaab7890e59fc125624c2efe27aebc32cf5d832e24287752912`.
- Source document sha256: `97c8620819021002a27ee5b1b7fa5a4c65690cbc4bc4cffca14fd71121785a2b`.
- Manifest id `ur550-civ-2007-text`.

### `rules/us-reports/ur556-civ-2009-order.txt`

- Supreme Court order adopting FRCP 6 (2009 amendment), U.S. Reports vol. 556 (adopting-order, FRCP 6, effective 2009-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-556/pdf/USREPORTS-556.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:35Z). Downloaded the source PDF (not stored: 6239156 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1297-1299 of USREPORTS-556.pdf).
- `us-reports/ur556-civ-2009-order.txt` sha256: `63be29633a821c44e60fd46df9f41e2192a097c5c0de6fd8eebed75dce12a62d`.
- Source document sha256: `bb76f3bf9eb752d2ee23fa0c3cf1129e96ab1cce32f9b7d19d6a33046f9dd4c3`.
- Manifest id `ur556-civ-2009-order`.

### `rules/us-reports/ur556-civ-2009-text.txt`

- Amended text of FRCP 6 (2009 amendment), U.S. Reports vol. 556 (amendment-text, FRCP 6, effective 2009-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-556/pdf/USREPORTS-556.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:35Z). Downloaded the source PDF (not stored: 6239156 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1300-1303 of USREPORTS-556.pdf).
- `us-reports/ur556-civ-2009-text.txt` sha256: `efc97d53b83a97d5508091795dd7f13bf3cd92e1eae2ab3d1fd24e11577bf5bd`.
- Source document sha256: `bb76f3bf9eb752d2ee23fa0c3cf1129e96ab1cce32f9b7d19d6a33046f9dd4c3`.
- Manifest id `ur556-civ-2009-text`.

### `rules/us-reports/ur578-civ-2016-order.txt`

- Supreme Court order adopting FRCP 6 (2016 amendment), U.S. Reports vol. 578 (adopting-order, FRCP 6, effective 2016-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-578/pdf/USREPORTS-578.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:33Z). Downloaded the source PDF (not stored: 2651256 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 873-875 of USREPORTS-578.pdf).
- `us-reports/ur578-civ-2016-order.txt` sha256: `cbdc4ef6884d8977c0f681851d1123ab2e1063841288b6d963b5e902460d04e7`.
- Source document sha256: `640ef4e730f3b15ff7f5a0faf56650cf975905c3068632df9363fa7a030a6b81`.
- Manifest id `ur578-civ-2016-order`.

### `rules/us-reports/ur578-civ-2016-text.txt`

- Amended text of FRCP 6 (2016 amendment), U.S. Reports vol. 578 (amendment-text, FRCP 6, effective 2016-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-578/pdf/USREPORTS-578.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:33Z). Downloaded the source PDF (not stored: 2651256 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 876-877 of USREPORTS-578.pdf).
- `us-reports/ur578-civ-2016-text.txt` sha256: `6f1092e4a1a3100600005a171c362401a8c23703fcece7d17d256763541cd71d`.
- Source document sha256: `640ef4e730f3b15ff7f5a0faf56650cf975905c3068632df9363fa7a030a6b81`.
- Manifest id `ur578-civ-2016-text`.

### `rules/us-reports/ur389-app-1967-order.txt`

- Supreme Court order adopting FRAP 26 (1967 amendment), U.S. Reports vol. 389 (adopting-order, FRAP 26, effective 1968-07-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-389/pdf/USREPORTS-389.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:58Z). Downloaded the source PDF (not stored: 64076490 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 969-971 of USREPORTS-389.pdf).
- `us-reports/ur389-app-1967-order.txt` sha256: `5f082bf263730b2568b4aa411989e665303d85323f79160d9706c3ba545f955a`.
- Source document sha256: `9666486d9ef1f197affe3f0fdee04e93a955db53c77e6182feff26319f786078`.
- Manifest id `ur389-app-1967-order`.

### `rules/us-reports/ur389-app-1967-text.txt`

- Amended text of FRAP 26 (1967 amendment), U.S. Reports vol. 389 (amendment-text, FRAP 26, effective 1968-07-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-389/pdf/USREPORTS-389.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:58Z). Downloaded the source PDF (not stored: 64076490 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 998-1000 of USREPORTS-389.pdf).
- `us-reports/ur389-app-1967-text.txt` sha256: `0fd6e8f743148c1f72d70d49912025526eb68c8b5f7a1fff97abc6b2672e0eaa`.
- Source document sha256: `9666486d9ef1f197affe3f0fdee04e93a955db53c77e6182feff26319f786078`.
- Manifest id `ur389-app-1967-text`.

### `rules/us-reports/ur401-app-1971-order.txt`

- Supreme Court order adopting FRAP 26 (1971 amendment), U.S. Reports vol. 401 (adopting-order, FRAP 26, effective 1971-07-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-401/pdf/USREPORTS-401.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:24:25Z). Downloaded the source PDF (not stored: 266184147 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1062-1064 of USREPORTS-401.pdf).
- `us-reports/ur401-app-1971-order.txt` sha256: `75d9384dce45553d523abea199b828c7ffc977ac59aa97d45afc400aa77adf14`.
- Source document sha256: `118fe75ce71f051c46e8000a56f4dbaf0d9290b39f045ba2f265a37527537130`.
- Manifest id `ur401-app-1971-order`.

### `rules/us-reports/ur401-app-1971-text.txt`

- Amended text of FRAP 26 (1971 amendment), U.S. Reports vol. 401 (amendment-text, FRAP 26, effective 1971-07-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-401/pdf/USREPORTS-401.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:24:25Z). Downloaded the source PDF (not stored: 266184147 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1073-1076 of USREPORTS-401.pdf).
- `us-reports/ur401-app-1971-text.txt` sha256: `f3a852ce17771e862451a504a8629d5a2687f473fc924d2e535fa26abc0a016d`.
- Source document sha256: `118fe75ce71f051c46e8000a56f4dbaf0d9290b39f045ba2f265a37527537130`.
- Manifest id `ur401-app-1971-text`.

### `rules/us-reports/ur475-app-1986-order.txt`

- Supreme Court order adopting FRAP 26 (1986 amendment), U.S. Reports vol. 475 (adopting-order, FRAP 26, effective 1986-07-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-475/pdf/USREPORTS-475.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:24:13Z). Downloaded the source PDF (not stored: 171851754 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1135-1138 of USREPORTS-475.pdf).
- `us-reports/ur475-app-1986-order.txt` sha256: `22d595e3486407e0cef82ec44a400af7166a74b646017145c62d58c9c7187c68`.
- Source document sha256: `0f46ebc9b881d12fbd0b7cc10ca68a345aa6496de53b2dc9fc7f88aa0a007ac7`.
- Manifest id `ur475-app-1986-order`.

### `rules/us-reports/ur475-app-1986-text.txt`

- Amended text of FRAP 26 (1986 amendment), U.S. Reports vol. 475 (amendment-text, FRAP 26, effective 1986-07-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-475/pdf/USREPORTS-475.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:24:13Z). Downloaded the source PDF (not stored: 171851754 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1147-1149 of USREPORTS-475.pdf).
- `us-reports/ur475-app-1986-text.txt` sha256: `7add98cd937c3db7cdd76df852e30c08880b4e45aff0d353b3efbdcf8fdb79e6`.
- Source document sha256: `0f46ebc9b881d12fbd0b7cc10ca68a345aa6496de53b2dc9fc7f88aa0a007ac7`.
- Manifest id `ur475-app-1986-text`.

### `rules/us-reports/ur490-app-1989-order.txt`

- Supreme Court order adopting FRAP 26 (1989 amendment), U.S. Reports vol. 490 (adopting-order, FRAP 26, effective 1989-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-490/pdf/USREPORTS-490.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:24:43Z). Downloaded the source PDF (not stored: 199958758 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1141-1144 of USREPORTS-490.pdf).
- `us-reports/ur490-app-1989-order.txt` sha256: `f6b350c9945a2662483c6ddd7216af516c568130f7bf4e05c542f72b80da2c46`.
- Source document sha256: `eed74c6fc7d78361aaeb8ea66cf14c24419872ba23541d8cabb4ac6b36eb561b`.
- Manifest id `ur490-app-1989-order`.

### `rules/us-reports/ur490-app-1989-text.txt`

- Amended text of FRAP 26 (1989 amendment), U.S. Reports vol. 490 (amendment-text, FRAP 26, effective 1989-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-490/pdf/USREPORTS-490.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:24:43Z). Downloaded the source PDF (not stored: 199958758 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1148-1150 of USREPORTS-490.pdf).
- `us-reports/ur490-app-1989-text.txt` sha256: `14228b24608d4bf68ed99d38fb010c2de39be1663bdac994e96133b487d87ae2`.
- Source document sha256: `eed74c6fc7d78361aaeb8ea66cf14c24419872ba23541d8cabb4ac6b36eb561b`.
- Manifest id `ur490-app-1989-text`.

### `rules/us-reports/ur500-app-1991-order.txt`

- Supreme Court order adopting FRAP 26 (1991 amendment), U.S. Reports vol. 500 (adopting-order, FRAP 26, effective 1991-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-500/pdf/USREPORTS-500.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:24:27Z). Downloaded the source PDF (not stored: 152307539 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 831-834 of USREPORTS-500.pdf).
- `us-reports/ur500-app-1991-order.txt` sha256: `2fb1ea363f0f8af5bce731dd18f84b17685ad0def52ddbeadb42c4578e42fa04`.
- Source document sha256: `00253a6f2d517e9d9650d31b1c2d3427d6a9ec7014d7bf4b5ffeb50be988bda0`.
- Manifest id `ur500-app-1991-order`.

### `rules/us-reports/ur500-app-1991-text.txt`

- Amended text of FRAP 26 (1991 amendment), U.S. Reports vol. 500 (amendment-text, FRAP 26, effective 1991-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-500/pdf/USREPORTS-500.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:24:27Z). Downloaded the source PDF (not stored: 152307539 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 836-838 of USREPORTS-500.pdf).
- `us-reports/ur500-app-1991-text.txt` sha256: `c0d45e4db0fbaaeb2d82602e8c2c4123be73615afc7f636d9d7cac5b5740d6d1`.
- Source document sha256: `00253a6f2d517e9d9650d31b1c2d3427d6a9ec7014d7bf4b5ffeb50be988bda0`.
- Manifest id `ur500-app-1991-text`.

### `rules/us-reports/ur517-app-1996-order.txt`

- Supreme Court order adopting FRAP 26 (1996 amendment), U.S. Reports vol. 517 (adopting-order, FRAP 26, effective 1996-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-517/pdf/USREPORTS-517.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:36Z). Downloaded the source PDF (not stored: 5027247 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1329-1332 of USREPORTS-517.pdf).
- `us-reports/ur517-app-1996-order.txt` sha256: `ccc82943f016373fdd2903116493964da38b192baa3dedb59f31ceb4994e9eb0`.
- Source document sha256: `e50b566eddd0f453c69aa4f002db0cb0033871d12518c1690248677c3a2791c1`.
- Manifest id `ur517-app-1996-order`.

### `rules/us-reports/ur517-app-1996-text.txt`

- Amended text of FRAP 26 (1996 amendment), U.S. Reports vol. 517 (amendment-text, FRAP 26, effective 1996-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-517/pdf/USREPORTS-517.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:36Z). Downloaded the source PDF (not stored: 5027247 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1335-1336 of USREPORTS-517.pdf).
- `us-reports/ur517-app-1996-text.txt` sha256: `5448bc6eb92604777e3da7563203162d14a46c15d2477435d71566443c61270a`.
- Source document sha256: `e50b566eddd0f453c69aa4f002db0cb0033871d12518c1690248677c3a2791c1`.
- Manifest id `ur517-app-1996-text`.

### `rules/us-reports/ur523-app-1998-order.txt`

- Supreme Court order adopting FRAP 26 (1998 amendment), U.S. Reports vol. 523 (adopting-order, FRAP 26, effective 1998-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-523/pdf/USREPORTS-523.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:33Z). Downloaded the source PDF (not stored: 4281236 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1158-1160 of USREPORTS-523.pdf).
- `us-reports/ur523-app-1998-order.txt` sha256: `90536b1328031f00ad5456545ac7ee09120e50aee5899004308b1b35f72fb466`.
- Source document sha256: `08ee36dd7df9c249e24445cfec0b91d173afe8449db427c083dec6d13206673f`.
- Manifest id `ur523-app-1998-order`.

### `rules/us-reports/ur523-app-1998-text.txt`

- Amended text of FRAP 26 (1998 amendment), U.S. Reports vol. 523 (amendment-text, FRAP 26, effective 1998-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-523/pdf/USREPORTS-523.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:33Z). Downloaded the source PDF (not stored: 4281236 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1196-1198 of USREPORTS-523.pdf).
- `us-reports/ur523-app-1998-text.txt` sha256: `e3fe43af81f3d709e2acd37921c0e0f834b43a62d6c0dfb9e266a75a827202bf`.
- Source document sha256: `08ee36dd7df9c249e24445cfec0b91d173afe8449db427c083dec6d13206673f`.
- Manifest id `ur523-app-1998-text`.

### `rules/us-reports/ur535-app-2002-order.txt`

- Supreme Court order adopting FRAP 26 (2002 amendment), U.S. Reports vol. 535 (adopting-order, FRAP 26, effective 2002-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-535/pdf/USREPORTS-535.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:34Z). Downloaded the source PDF (not stored: 4402967 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1175-1178 of USREPORTS-535.pdf).
- `us-reports/ur535-app-2002-order.txt` sha256: `8cc0f8654895bea2847ab799c773ade07f0fc0b7e0ba36b1621aca94d238ae83`.
- Source document sha256: `8483e2d471513ad1447b20cd9e501783d928a1e6a8196ad262dcbe372860f873`.
- Manifest id `ur535-app-2002-order`.

### `rules/us-reports/ur535-app-2002-text.txt`

- Amended text of FRAP 26 (2002 amendment), U.S. Reports vol. 535 (amendment-text, FRAP 26, effective 2002-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-535/pdf/USREPORTS-535.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:34Z). Downloaded the source PDF (not stored: 4402967 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1182-1184 of USREPORTS-535.pdf).
- `us-reports/ur535-app-2002-text.txt` sha256: `d5239f426ec92322744cef780ae59409cd1c4cd41b447a7e18619474f8528783`.
- Source document sha256: `8483e2d471513ad1447b20cd9e501783d928a1e6a8196ad262dcbe372860f873`.
- Manifest id `ur535-app-2002-text`.

### `rules/us-reports/ur544-app-2005-order.txt`

- Supreme Court order adopting FRAP 26 (2005 amendment), U.S. Reports vol. 544 (adopting-order, FRAP 26, effective 2005-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-544/pdf/USREPORTS-544.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:33Z). Downloaded the source PDF (not stored: 3714571 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1084-1086 of USREPORTS-544.pdf).
- `us-reports/ur544-app-2005-order.txt` sha256: `b02e3ea3faef3eeee30b388a54b7657d2c3fd454ce8936442e1bf3fc68198be4`.
- Source document sha256: `e40e6385620c128442e72cd92a0ac719692a4e3496ec2ed057d3002122b17de3`.
- Manifest id `ur544-app-2005-order`.

### `rules/us-reports/ur544-app-2005-text.txt`

- Amended text of FRAP 26 (2005 amendment), U.S. Reports vol. 544 (amendment-text, FRAP 26, effective 2005-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-544/pdf/USREPORTS-544.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:33Z). Downloaded the source PDF (not stored: 3714571 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1087-1088 of USREPORTS-544.pdf).
- `us-reports/ur544-app-2005-text.txt` sha256: `16123db8e094f6d34ae58c412f01573f514f5ab696d1ae90b02c6d4f8abd5978`.
- Source document sha256: `e40e6385620c128442e72cd92a0ac719692a4e3496ec2ed057d3002122b17de3`.
- Manifest id `ur544-app-2005-text`.

### `rules/us-reports/ur556-app-2009-order.txt`

- Supreme Court order adopting FRAP 26 (2009 amendment), U.S. Reports vol. 556 (adopting-order, FRAP 26, effective 2009-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-556/pdf/USREPORTS-556.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:35Z). Downloaded the source PDF (not stored: 6239156 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1250-1252 of USREPORTS-556.pdf).
- `us-reports/ur556-app-2009-order.txt` sha256: `b4fc054921b4127e6192578f80ab8a637fa0fa339f5000d300480452321a10b1`.
- Source document sha256: `bb76f3bf9eb752d2ee23fa0c3cf1129e96ab1cce32f9b7d19d6a33046f9dd4c3`.
- Manifest id `ur556-app-2009-order`.

### `rules/us-reports/ur556-app-2009-text.txt`

- Amended text of FRAP 26 (2009 amendment), U.S. Reports vol. 556 (amendment-text, FRAP 26, effective 2009-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-556/pdf/USREPORTS-556.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:35Z). Downloaded the source PDF (not stored: 6239156 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 1259-1261 of USREPORTS-556.pdf).
- `us-reports/ur556-app-2009-text.txt` sha256: `2907f8b37b0ad72700bac37cab51c1e2b46ea190e3155b0255bf37f47badb16c`.
- Source document sha256: `bb76f3bf9eb752d2ee23fa0c3cf1129e96ab1cce32f9b7d19d6a33046f9dd4c3`.
- Manifest id `ur556-app-2009-text`.

### `rules/us-reports/ur578-app-2016-order.txt`

- Supreme Court order adopting FRAP 26 (2016 amendment), U.S. Reports vol. 578 (adopting-order, FRAP 26, effective 2016-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-578/pdf/USREPORTS-578.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:33Z). Downloaded the source PDF (not stored: 2651256 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 847-849 of USREPORTS-578.pdf).
- `us-reports/ur578-app-2016-order.txt` sha256: `b39194e3c8f0e590b451b53b5c2dba78857c1aae3463514c6c64fe050ca754fb`.
- Source document sha256: `640ef4e730f3b15ff7f5a0faf56650cf975905c3068632df9363fa7a030a6b81`.
- Manifest id `ur578-app-2016-order`.

### `rules/us-reports/ur578-app-2016-text.txt`

- Amended text of FRAP 26 (2016 amendment), U.S. Reports vol. 578 (amendment-text, FRAP 26, effective 2016-12-01).
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-578/pdf/USREPORTS-578.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:23:33Z). Downloaded the source PDF (not stored: 2651256 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF pages 852-855 of USREPORTS-578.pdf).
- `us-reports/ur578-app-2016-text.txt` sha256: `4939726fcbf09645dd231ca8603dc6f0396d0f162a75d8c9d144ae291509fc41`.
- Source document sha256: `640ef4e730f3b15ff7f5a0faf56650cf975905c3068632df9363fa7a030a6b81`.
- Manifest id `ur578-app-2016-text`.

### `rules/scotus/frcv09.pdf`, `rules/scotus/frcv09.txt`

- Supreme Court order, Chief Justice transmittal letters and amended text, FRCP (incl. Rule 6), Mar. 26, 2009 (adopting-order, FRCP 6, effective 2009-12-01).
- Source: <https://www.supremecourt.gov/orders/courtorders/frcv09.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:16:53Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `scotus/frcv09.pdf` sha256: `966fc6224e82c4bee3dbe99b247277d44eebf660e942b0879e15af6476864ab5`.
- `scotus/frcv09.txt` sha256: `d2150f45d5f2306a3b21be12d99519ced0031a3fe66800ff75361f060bca0126`.
- Manifest id `sc-frcv09-2009`.

### `rules/scotus/frap09.pdf`, `rules/scotus/frap09.txt`

- Supreme Court order, Chief Justice transmittal letters and amended text, FRAP (incl. Rule 26), Mar. 26, 2009 (adopting-order, FRAP 26, effective 2009-12-01).
- Source: <https://www.supremecourt.gov/orders/courtorders/frap09.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:16:54Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `scotus/frap09.pdf` sha256: `e0165d121f8bad45d3f3e9b97bcbea6f24fcd80146c38739ec9ca9ff20d5f46b`.
- `scotus/frap09.txt` sha256: `09ca6028ce22ac997b54dcc3443d7edd001bd1f82b00a28213483ae6ca1403fd`.
- Manifest id `sc-frap09-2009`.

### `rules/scotus/frcv05p.pdf`, `rules/scotus/frcv05p.txt`

- Supreme Court order and amended text, FRCP Rules 6, 27, 45, Apr. 25, 2005 (adopting-order, FRCP 6, effective 2005-12-01).
- Source: <https://www.supremecourt.gov/orders/courtorders/frcv05p.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:28:55Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `scotus/frcv05p.pdf` sha256: `9b1a457559299a377e2072113c3ce95de9c6bf78e89a754c88c9276e0d36c6fc`.
- `scotus/frcv05p.txt` sha256: `e7815ff9c4ee00f134e7300745416db1df59ce3159ec14581ba66dd76cc68a1c`.
- Manifest id `sc-frcv05-2005`.

### `rules/scotus/frap05p.pdf`, `rules/scotus/frap05p.txt`

- Supreme Court order and amended text, FRAP Rules 4, 26, 27, 28, 32, 34, 35, 45, 28.1, Apr. 25, 2005 (adopting-order, FRAP 26, effective 2005-12-01).
- Source: <https://www.supremecourt.gov/orders/courtorders/frap05p.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:28:55Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `scotus/frap05p.pdf` sha256: `59c5e88e9ee31ae112f365ba28362f6c0e9ab027b030e894f13451a4237239d1`.
- `scotus/frap05p.txt` sha256: `2f439a298a52de03c0fb3408d7a8c06cab5283ec76d719f9496602432dc756f6`.
- Manifest id `sc-frap05-2005`.

### `rules/scotus/frcv07p.pdf`, `rules/scotus/frcv07p.txt`

- Supreme Court order and restyled FRCP (incl. Rule 6), Apr. 30, 2007 (adopting-order, FRCP 6, effective 2007-12-01).
- Source: <https://www.supremecourt.gov/orders/courtorders/frcv07p.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:28:56Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `scotus/frcv07p.pdf` sha256: `f43e6e62aab27c02e357ece586b499b54f83bae7496073a6f6bfd4a6e8c3220d`.
- `scotus/frcv07p.txt` sha256: `897b9f3bc4a1c673faa7bff1e22bd29bf2cf1fcbbb2480dd4e34830a595c8589`.
- Manifest id `sc-frcv07-2007`.

### `rules/congress-packages/2016-04-28-final_package_to_congress_with_revd_cover_letters_and_orders_0.pdf`, `rules/congress-packages/2016-04-28-final_package_to_congress_with_revd_cover_letters_and_orders_0.txt`

- Package transmitted to Congress Apr. 28, 2016: Supreme Court orders, rule text and Committee Notes (FRCP 6(d), FRAP 26(c) e-service 3-day removal) (transmittal, FRCP 6; FRAP 26, effective 2016-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/2016-04-28-final_package_to_congress_with_revd_cover_letters_and_orders_0.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:36:04Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `congress-packages/2016-04-28-final_package_to_congress_with_revd_cover_letters_and_orders_0.pdf` sha256: `51e144f666f03e026744b8152ad094f599464cffa9f7a471319eb518f70649c5`.
- `congress-packages/2016-04-28-final_package_to_congress_with_revd_cover_letters_and_orders_0.txt` sha256: `1731c27c9d5ca29ffc6e1863ef3c6bb18ca397aef0ab8bee902c9431ebbb6b38`.
- Manifest id `cong-2016`.

### `rules/congress-packages/2015-10-09-supreme_court_package_final_0.pdf`, `rules/congress-packages/2015-10-09-supreme_court_package_final_0.txt`

- Judicial Conference package sent to the Supreme Court (Oct. 2015), proposed FRCP 6(d)/FRAP 26(c) with Committee Notes (transmittal, FRCP 6; FRAP 26, effective 2016-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/2015-10-09-supreme_court_package_final_0.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:36:05Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `congress-packages/2015-10-09-supreme_court_package_final_0.pdf` sha256: `c8eab6a2c1661a89cc67ebd7f258dbd7c837fdb7b5a150ba7cfea20e6d882be6`.
- `congress-packages/2015-10-09-supreme_court_package_final_0.txt` sha256: `34927d97f2c8e8cfadc3ee0c01c5b81ff5408e1c0ebbd6e1673336c09d74b503`.
- Manifest id `jc-to-scotus-2015`.

### `rules/congress-packages/2019-04-25-congressional_rules_package_final_0.pdf`, `rules/congress-packages/2019-04-25-congressional_rules_package_final_0.txt`

- Package transmitted to Congress Apr. 25, 2019: FRAP order, Rule 26 amendment and Committee Note (transmittal, FRAP 26, effective 2019-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/2019-04-25-congressional_rules_package_final_0.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:36:04Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `congress-packages/2019-04-25-congressional_rules_package_final_0.pdf` sha256: `281ecabd2962a03fe85deddea1045a476edfb5d5478d66d680ae3399f8f53dc4`.
- `congress-packages/2019-04-25-congressional_rules_package_final_0.txt` sha256: `4f79937c2fcda4bba77a3abdce79d0b70b8e211117a0c8198ff189ef3c9a3d64`.
- Manifest id `cong-2019`.

### `rules/congress-packages/2023_congressional_package_april_24_2023_0.pdf`, `rules/congress-packages/2023_congressional_package_april_24_2023_0.txt`

- Package transmitted to Congress Apr. 24, 2023: orders, rule text and Notes (Juneteenth in FRCP 6(a)(6) and FRAP 26(a)(6)) (transmittal, FRCP 6; FRAP 26, effective 2023-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/2023_congressional_package_april_24_2023_0.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:36:04Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `congress-packages/2023_congressional_package_april_24_2023_0.pdf` sha256: `290a7566ee8f985a60874db840c8790b7aa0dc4218b3fe1c4ff1f3ae42b5977e`.
- `congress-packages/2023_congressional_package_april_24_2023_0.txt` sha256: `c0f1369cfd249c28e74b8ea64abbf86185c5287296778363ec17117ea27e1a15`.
- Manifest id `cong-2023`.

### `rules/congress-packages/2022_scotus_package_0.pdf`, `rules/congress-packages/2022_scotus_package_0.txt`

- Judicial Conference package sent to the Supreme Court (2022), proposed Juneteenth amendments (transmittal, FRCP 6; FRAP 26, effective 2023-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/2022_scotus_package_0.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:36:04Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `congress-packages/2022_scotus_package_0.pdf` sha256: `5981cfc9bb35013d85cf787d11d84690f20e2af1dd73e7c99e57990bb94b7eb1`.
- `congress-packages/2022_scotus_package_0.txt` sha256: `6c3af2fe19b2eada8827324aa05660699a0cb0774ec0a02180857d60f215a94d`.
- Manifest id `jc-to-scotus-2022`.

### `rules/committee-reports/f14685-1982-civil-final-draft.pdf`, `rules/committee-reports/f14685-1982-civil-final-draft.txt`

- Advisory Committee on Civil Rules final draft of proposed amendments incl. Rule 6, Mar. 9, 1982 (report, FRCP 6, effective 1983-08-01).
- Source: <https://www.uscourts.gov/file/14685/download>.
- Retrieved 2026-10-23 (2026-10-23T06:29:56Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `committee-reports/f14685-1982-civil-final-draft.pdf` sha256: `edcca6721d7e28099563d3caab4de9aeb5336637daa5d9f88a361bfb086b33f5`.
- `committee-reports/f14685-1982-civil-final-draft.txt` sha256: `86ffec43208ccf30d0c6c7dbd0e03212085f74001ffb564d54c2fa46276033f4`.
- Manifest id `rpt-civ-1982-03`.

### `rules/committee-reports/f14707-1984-civil-final-draft.pdf`, `rules/committee-reports/f14707-1984-civil-final-draft.txt`

- Advisory Committee on Civil Rules final draft of amendments incl. Rule 6(a), July 18, 1984 (report, FRCP 6, effective 1985-08-01).
- Source: <https://www.uscourts.gov/file/14707/download>.
- Retrieved 2026-10-23 (2026-10-23T06:29:56Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `committee-reports/f14707-1984-civil-final-draft.pdf` sha256: `e8638022e2d75ca887b34c07a2f69b48dcfb7f7c2d2eb2d0282f38f59945fd03`.
- `committee-reports/f14707-1984-civil-final-draft.txt` sha256: `3e5c8df8e42c971d9da7ffbbbf2765d75dfc9966858817108a0b9fe0228b8749`.
- Manifest id `rpt-civ-1984-07`.

### `rules/committee-reports/ST09-1984.pdf`, `rules/committee-reports/ST09-1984.txt`

- Standing Committee report to the Judicial Conference, Sept. 1984 (Civil Rule 6 amendment and Note) (report, FRCP 6, effective 1985-08-01).
- Source: <https://www.uscourts.gov/sites/default/files/fr_import/ST09-1984.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:32:11Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `committee-reports/ST09-1984.pdf` sha256: `4eac09b259915bf46c6c514f3fcab41513f3d2b8c61025af6e3da9df48cb7282`.
- `committee-reports/ST09-1984.txt` sha256: `f3415c65780ebc82f13b19553452bd828a843378acb4ccd2224e0231ef7bd248`.
- Manifest id `rpt-jc-1984-09`.

### `rules/committee-reports/ST09-1985.pdf`, `rules/committee-reports/ST09-1985.txt`

- Standing Committee report to the Judicial Conference, Sept. 1985 (FRAP 26(a),(c) 1986 amendment and Note) (report, FRAP 26, effective 1986-07-01).
- Source: <https://www.uscourts.gov/sites/default/files/fr_import/ST09-1985.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:32:16Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `committee-reports/ST09-1985.pdf` sha256: `0d1fcdb3271ac6762a15afb47b648432e87f01d623e7838ace08d2a2cbfe4c95`.
- `committee-reports/ST09-1985.txt` sha256: `2c6c21a434046cd856c333a927af7553b74f68be1a317f0aac5505231b87ccaa`.
- Manifest id `rpt-jc-1985-09`.

### `rules/committee-reports/ST03-1989.pdf`, `rules/committee-reports/ST03-1989.txt`

- Standing Committee report to the Judicial Conference, Mar. 1989 (FRAP 26(a) 1989 amendment and Note) (report, FRAP 26, effective 1989-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/fr_import/ST03-1989.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:32:30Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `committee-reports/ST03-1989.pdf` sha256: `bd0d90aed4e269f40954133da5beba106cacf254508d9936550f199e14a16ee1`.
- `committee-reports/ST03-1989.txt` sha256: `26f5a8a958061cc68d7b0f00209ac98cc85753d8e4f1754f0e498d5977d59004`.
- Manifest id `rpt-jc-1989-03`.

### `rules/committee-reports/ST09-1990.pdf`, `rules/committee-reports/ST09-1990.txt`

- Standing Committee report to the Judicial Conference, Sept. 1990 (FRAP 26(a) 1991 amendment) (report, FRAP 26, effective 1991-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/fr_import/ST09-1990.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:32:37Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `committee-reports/ST09-1990.pdf` sha256: `cafb3663f43d2391f0dff391bf055caa6e051e4a701e896afc08ee8dce6889f7`.
- `committee-reports/ST09-1990.txt` sha256: `65e00c478d8b2c348c2f4be0dd49250d5110aa3c09812008ed1df2cc944b7d93`.
- Manifest id `rpt-jc-1990-09`.

### `rules/committee-reports/ST9-1995.pdf`, `rules/committee-reports/ST9-1995.txt`

- Standing Committee report to the Judicial Conference, Sept. 1995 (FRAP 26(c) 1996 amendment and Note) (report, FRAP 26, effective 1996-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/fr_import/ST9-1995.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:33:05Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `committee-reports/ST9-1995.pdf` sha256: `6b06b0c4c32c5d2e6b987453c99db8788cbfa6f0fda1e5ad527dc6eb558180c9`.
- `committee-reports/ST9-1995.txt` sha256: `d6b6851cf8829f81d9594d9f958d36fb3a480344d84ec11d89c51b8780efb15d`.
- Manifest id `rpt-jc-1995-09`.

### `rules/committee-reports/ST9-1997.txt`

- Standing Committee report to the Judicial Conference, Sept. 1997 (FRAP restyling incl. Rule 26, 1998) (report, FRAP 26, effective 1998-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/fr_import/ST9-1997.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:33:16Z). Downloaded the source PDF (not stored: 31257378 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF p.266-268 Rule 26 and Note).
- `committee-reports/ST9-1997.txt` sha256: `394de4c5883baba47b03fb2410fd2197f2c8fbee59ba3c8070382a87b5da7b26`.
- Source document sha256: `217e23c9ab868a3d9d234ebc6b03b95da51864ff19a2c7278c2cb39099eceb2f`.
- Manifest id `rpt-jc-1997-09`.

### `rules/committee-reports/ST9-1998.pdf`, `rules/committee-reports/ST9-1998.txt`

- Standing Committee report to the Judicial Conference, Sept. 1998 (FRCP 6(b) conforming amendment, 1999) (report, FRCP 6, effective 1999-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/fr_import/ST9-1998.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:33:21Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `committee-reports/ST9-1998.pdf` sha256: `914294183add8e48c0c3ecedf87ba6c3eb177f9dd2847ad805ac20fdaf70c35b`.
- `committee-reports/ST9-1998.txt` sha256: `4825f8e90567e943f64055adeb04f7a9eb1195c121b1ae21406f30e389eb3e05`.
- Manifest id `rpt-jc-1998-09`.

### `rules/committee-reports/ST9-2000.pdf`, `rules/committee-reports/ST9-2000.txt`

- Standing Committee report to the Judicial Conference, Sept. 2000 (FRCP 6(e) electronic service, 2001; FRAP 26(c)) (report, FRCP 6; FRAP 26, effective 2001-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/fr_import/ST9-2000.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:33:32Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `committee-reports/ST9-2000.pdf` sha256: `29a84f276da3e7c2087ef10dc05c7e88c3efa1d375152a8c865290275bb77369`.
- `committee-reports/ST9-2000.txt` sha256: `bc0903d0d88bed7c9c024c2cf00ad0c8654dfb6ceb4a3cc75e2764d186a630fe`.
- Manifest id `rpt-jc-2000-09`.

### `rules/committee-reports/ST9-2001.txt`

- Standing Committee report to the Judicial Conference, Sept. 2001 (FRAP 26(a),(c) 2002 amendments and Notes) (report, FRAP 26, effective 2002-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/fr_import/ST9-2001.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:33:37Z). Downloaded the source PDF (not stored: 36662187 bytes, over the 20 MB cap); extracted with `pdftotext -layout` (PDF p.104-106, 110-112).
- `committee-reports/ST9-2001.txt` sha256: `ff592318c3df8a69102ce10f428624e3fc5a3b0821da581b15843ab5e02ce505`.
- Source document sha256: `1b5edd009790f80299cf4c4ad559c0698f147822a01ada198a6b5f2b8fc1474e`.
- Manifest id `rpt-jc-2001-09`.

### `rules/committee-reports/f14745-2004-05-17-civil-rules-report.pdf`, `rules/committee-reports/f14745-2004-05-17-civil-rules-report.txt`

- Civil Rules Advisory Committee report, May 17, 2004 (Rule 6(e) amendment and Note, 2005) (report, FRCP 6, effective 2005-12-01).
- Source: <https://www.uscourts.gov/file/14745/download>.
- Retrieved 2026-10-23 (2026-10-23T06:29:57Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `committee-reports/f14745-2004-05-17-civil-rules-report.pdf` sha256: `6cae7121b774a0315382c7edae519200a9565ec726659e873cbe6ee81ad17c53`.
- `committee-reports/f14745-2004-05-17-civil-rules-report.txt` sha256: `d8ef74b025f851e084bec01a0e0e4bbdb04020a8c29ebc5a75525250519ff22c`.
- Manifest id `rpt-civ-2004-05`.

### `rules/committee-reports/ST09-2008.pdf`, `rules/committee-reports/ST09-2008.txt`

- Standing Committee report to the Judicial Conference, Sept. 2008 (time-computation project: FRCP 6 and FRAP 26 text and 2009 Committee Notes) (report, FRCP 6; FRAP 26, effective 2009-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/fr_import/ST09-2008.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:34:16Z). Downloaded as served; stored unedited, with a `pdftotext -layout` extraction beside it.
- `committee-reports/ST09-2008.pdf` sha256: `8c15ca6f36d5d5c0fb544e73c37cf79de9845bef4b4e31af5d7b1b8bc9352888`.
- `committee-reports/ST09-2008.txt` sha256: `6bb72a001f54ba19c8ba3590c27299ba6201660babea0e831dd8097122c8eff9`.
- Manifest id `rpt-jc-2008-09`.

### `rules/committee-notes/note-civ-1985.txt`

- Committee Note, FRCP 6, effective 1985-08-01 (committee-note, FRCP 6, effective 1985-08-01).
- Source: <https://www.uscourts.gov/sites/default/files/fr_import/ST09-1984.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:32:11Z). Downloaded the source PDF (stored under `rules/committee-reports/`); excerpted with `pdftotext -layout` (PDF pages 43-45 of ST09-1984.pdf).
- `committee-notes/note-civ-1985.txt` sha256: `4cac959e26f4a1822d86b0f00d8578e7f5a6fbf063a0b88d7cdb9d6795efe105`.
- Source document sha256: `4eac09b259915bf46c6c514f3fcab41513f3d2b8c61025af6e3da9df48cb7282`.
- Manifest id `note-civ-1985`.

### `rules/committee-notes/note-civ-1985b.txt`

- Committee Note, FRCP 6, effective 1985-08-01 (committee-note, FRCP 6, effective 1985-08-01).
- Source: <https://www.uscourts.gov/file/14707/download>.
- Retrieved 2026-10-23 (2026-10-23T06:29:56Z). Downloaded the source PDF (stored under `rules/committee-reports/`); excerpted with `pdftotext -layout` (PDF pages 4-5 of f14707.pdf).
- `committee-notes/note-civ-1985b.txt` sha256: `78a0ad2078be91b02d2b3dd7ea7e75e0f7c400adfef4c488087d5a50423487db`.
- Source document sha256: `e8638022e2d75ca887b34c07a2f69b48dcfb7f7c2d2eb2d0282f38f59945fd03`.
- Manifest id `note-civ-1985b`.

### `rules/committee-notes/note-civ-1999.txt`

- Committee Note, FRCP 6, effective 1999-12-01 (committee-note, FRCP 6, effective 1999-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/fr_import/ST9-1998.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:33:21Z). Downloaded the source PDF (stored under `rules/committee-reports/`); excerpted with `pdftotext -layout` (PDF pages 87-89 of ST9-1998.pdf).
- `committee-notes/note-civ-1999.txt` sha256: `840003b32afb20366e26685adbd85990d8dbb3bceef392b0c150a9a82059570b`.
- Source document sha256: `914294183add8e48c0c3ecedf87ba6c3eb177f9dd2847ad805ac20fdaf70c35b`.
- Manifest id `note-civ-1999`.

### `rules/committee-notes/note-civ-2001.txt`

- Committee Note, FRCP 6, effective 2001-12-01 (committee-note, FRCP 6, effective 2001-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/fr_import/ST9-2000.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:33:32Z). Downloaded the source PDF (stored under `rules/committee-reports/`); excerpted with `pdftotext -layout` (PDF pages 66-68 of ST9-2000.pdf).
- `committee-notes/note-civ-2001.txt` sha256: `58ce3b02aca30fd9313aa212f2a3a7de0a413dced86b2ffbbbc92f00e5112842`.
- Source document sha256: `29a84f276da3e7c2087ef10dc05c7e88c3efa1d375152a8c865290275bb77369`.
- Manifest id `note-civ-2001`.

### `rules/committee-notes/note-civ-2005.txt`

- Committee Note, FRCP 6, effective 2005-12-01 (committee-note, FRCP 6, effective 2005-12-01).
- Source: <https://www.uscourts.gov/file/14745/download>.
- Retrieved 2026-10-23 (2026-10-23T06:29:57Z). Downloaded the source PDF (stored under `rules/committee-reports/`); excerpted with `pdftotext -layout` (PDF pages 2-4 of f14745.pdf).
- `committee-notes/note-civ-2005.txt` sha256: `43f6a60bfc86527c55b7da1727dedf5ed8661cdfff1216324fd81aff0954c2b8`.
- Source document sha256: `6cae7121b774a0315382c7edae519200a9565ec726659e873cbe6ee81ad17c53`.
- Manifest id `note-civ-2005`.

### `rules/committee-notes/note-civ-2009.txt`

- Committee Note, FRCP 6, effective 2009-12-01 (committee-note, FRCP 6, effective 2009-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/fr_import/ST09-2008.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:34:16Z). Downloaded the source PDF (stored under `rules/committee-reports/`); excerpted with `pdftotext -layout` (PDF pages 234-246 of ST09-2008.pdf).
- `committee-notes/note-civ-2009.txt` sha256: `f808e6e77ab8d14ae02002db7cb26a1445933ad8950fe4976673652109676546`.
- Source document sha256: `8c15ca6f36d5d5c0fb544e73c37cf79de9845bef4b4e31af5d7b1b8bc9352888`.
- Manifest id `note-civ-2009`.

### `rules/committee-notes/note-civ-2016.txt`

- Committee Note, FRCP 6, effective 2016-12-01 (committee-note, FRCP 6, effective 2016-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/2016-04-28-final_package_to_congress_with_revd_cover_letters_and_orders_0.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:36:04Z). Downloaded the source PDF (stored under `rules/committee-reports/`); excerpted with `pdftotext -layout` (PDF pages 187-189 of cong2016.pdf).
- `committee-notes/note-civ-2016.txt` sha256: `90c9f98d053945862bdc2e2eba41792360066ca71c8c319381431e4d72fbbd0a`.
- Source document sha256: `51e144f666f03e026744b8152ad094f599464cffa9f7a471319eb518f70649c5`.
- Manifest id `note-civ-2016`.

### `rules/committee-notes/note-civ-2023.txt`

- Committee Note, FRCP 6, effective 2023-12-01 (committee-note, FRCP 6, effective 2023-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/2023_congressional_package_april_24_2023_0.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:36:04Z). Downloaded the source PDF (stored under `rules/committee-reports/`); excerpted with `pdftotext -layout` (PDF pages 88-91 of cong2023.pdf).
- `committee-notes/note-civ-2023.txt` sha256: `17eeb70132c350d5908405f922255f4b9a6158ffaab4a1ccc58be86643db6f5c`.
- Source document sha256: `290a7566ee8f985a60874db840c8790b7aa0dc4218b3fe1c4ff1f3ae42b5977e`.
- Manifest id `note-civ-2023`.

### `rules/committee-notes/note-app-1986.txt`

- Committee Note, FRAP 26, effective 1986-07-01 (committee-note, FRAP 26, effective 1986-07-01).
- Source: <https://www.uscourts.gov/sites/default/files/fr_import/ST09-1985.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:32:16Z). Downloaded the source PDF (stored under `rules/committee-reports/`); excerpted with `pdftotext -layout` (PDF pages 27-28 of ST09-1985.pdf).
- `committee-notes/note-app-1986.txt` sha256: `a86e3f3972d521ead932101a0e3fa22c04db111593bed6f3958aacfc530ab222`.
- Source document sha256: `0d1fcdb3271ac6762a15afb47b648432e87f01d623e7838ace08d2a2cbfe4c95`.
- Manifest id `note-app-1986`.

### `rules/committee-notes/note-app-1989.txt`

- Committee Note, FRAP 26, effective 1989-12-01 (committee-note, FRAP 26, effective 1989-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/fr_import/ST03-1989.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:32:30Z). Downloaded the source PDF (stored under `rules/committee-reports/`); excerpted with `pdftotext -layout` (PDF pages 44-44 of ST03-1989.pdf).
- `committee-notes/note-app-1989.txt` sha256: `2a53d854b45fbf9d455de125040f5b1aff95511605651a87093ec667547e4865`.
- Source document sha256: `bd0d90aed4e269f40954133da5beba106cacf254508d9936550f199e14a16ee1`.
- Manifest id `note-app-1989`.

### `rules/committee-notes/note-app-1996.txt`

- Committee Note, FRAP 26, effective 1996-12-01 (committee-note, FRAP 26, effective 1996-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/fr_import/ST9-1995.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:33:05Z). Downloaded the source PDF (stored under `rules/committee-reports/`); excerpted with `pdftotext -layout` (PDF pages 74-76 of ST9-1995.pdf).
- `committee-notes/note-app-1996.txt` sha256: `a9d79f39e5e2cb6ca7fd025cf285fe60141636feda89b3ad4f4eb513c927e71f`.
- Source document sha256: `6b06b0c4c32c5d2e6b987453c99db8788cbfa6f0fda1e5ad527dc6eb558180c9`.
- Manifest id `note-app-1996`.

### `rules/committee-notes/note-app-1998.txt`

- Committee Note, FRAP 26, effective 1998-12-01 (committee-note, FRAP 26, effective 1998-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/fr_import/ST9-1997.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:33:16Z). Downloaded the source PDF (stored under `rules/committee-reports/`); excerpted with `pdftotext -layout` (PDF pages 266-268 of ST9-1997.pdf).
- `committee-notes/note-app-1998.txt` sha256: `d4754165b0a7c3dcf16428e3ceb6ef2d0293a262d3831b826012cfea2a7c9d28`.
- Source document sha256: `217e23c9ab868a3d9d234ebc6b03b95da51864ff19a2c7278c2cb39099eceb2f`.
- Manifest id `note-app-1998`.

### `rules/committee-notes/note-app-2002.txt`

- Committee Note, FRAP 26, effective 2002-12-01 (committee-note, FRAP 26, effective 2002-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/fr_import/ST9-2001.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:33:37Z). Downloaded the source PDF (stored under `rules/committee-reports/`); excerpted with `pdftotext -layout` (PDF pages 104-106,110-112 of ST9-2001.pdf).
- `committee-notes/note-app-2002.txt` sha256: `25042a4570d9a6a5fbdb61bc0864bd1083a2d38911a7d490726c2c3051ca1f26`.
- Source document sha256: `1b5edd009790f80299cf4c4ad559c0698f147822a01ada198a6b5f2b8fc1474e`.
- Manifest id `note-app-2002`.

### `rules/committee-notes/note-app-2009.txt`

- Committee Note, FRAP 26, effective 2009-12-01 (committee-note, FRAP 26, effective 2009-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/fr_import/ST09-2008.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:34:16Z). Downloaded the source PDF (stored under `rules/committee-reports/`); excerpted with `pdftotext -layout` (PDF pages 50-65,76-93 of ST09-2008.pdf).
- `committee-notes/note-app-2009.txt` sha256: `1a38f43a24339ebf2ce71099e93ec1947966a4b81539e9d3142ece35752f0f1b`.
- Source document sha256: `8c15ca6f36d5d5c0fb544e73c37cf79de9845bef4b4e31af5d7b1b8bc9352888`.
- Manifest id `note-app-2009`.

### `rules/committee-notes/note-app-2016.txt`

- Committee Note, FRAP 26, effective 2016-12-01 (committee-note, FRAP 26, effective 2016-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/2016-04-28-final_package_to_congress_with_revd_cover_letters_and_orders_0.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:36:04Z). Downloaded the source PDF (stored under `rules/committee-reports/`); excerpted with `pdftotext -layout` (PDF pages 89-91 of cong2016.pdf).
- `committee-notes/note-app-2016.txt` sha256: `b9a1d0894d34009e897cce44d4cdf4e1131ddf4d8878a715672aaccf2549c191`.
- Source document sha256: `51e144f666f03e026744b8152ad094f599464cffa9f7a471319eb518f70649c5`.
- Manifest id `note-app-2016`.

### `rules/committee-notes/note-app-2019.txt`

- Committee Note, FRAP 26, effective 2019-12-01 (committee-note, FRAP 26, effective 2019-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/2019-04-25-congressional_rules_package_final_0.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:36:04Z). Downloaded the source PDF (stored under `rules/committee-reports/`); excerpted with `pdftotext -layout` (PDF pages 30-31 of cong2019.pdf).
- `committee-notes/note-app-2019.txt` sha256: `7dcac3f23ffa0cd308f0a59c8dec729003d2c829625339c7b2e4a946c93c5abd`.
- Source document sha256: `281ecabd2962a03fe85deddea1045a476edfb5d5478d66d680ae3399f8f53dc4`.
- Manifest id `note-app-2019`.

### `rules/committee-notes/note-app-2023.txt`

- Committee Note, FRAP 26, effective 2023-12-01 (committee-note, FRAP 26, effective 2023-12-01).
- Source: <https://www.uscourts.gov/sites/default/files/2023_congressional_package_april_24_2023_0.pdf>.
- Retrieved 2026-10-23 (2026-10-23T06:36:04Z). Downloaded the source PDF (stored under `rules/committee-reports/`); excerpted with `pdftotext -layout` (PDF pages 31-32 of cong2023.pdf).
- `committee-notes/note-app-2023.txt` sha256: `2969b699594bdb2ca6892775533401f906b4369d60296b168d7fe842499a75f9`.
- Source document sha256: `290a7566ee8f985a60874db840c8790b7aa0dc4218b3fe1c4ff1f3ae42b5977e`.
- Manifest id `note-app-2023`.

## Rulings: timeliness opinions (`rulings/`)

### `rulings/lashley-5cir-1975.html`, `rulings/lashley-5cir-1975.json`, `rulings/lashley-5cir-1975.txt`

- Lashley v. Ford Motor Co., 518 F.2d 749 (5th Cir. 1975-09-08), No. 75-2747.
- Source: <https://static.case.law/f2d/518/html/0749-01.html>; metadata <https://static.case.law/f2d/518/cases/0749-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:08:07Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `lashley-5cir-1975.txt`: tags stripped from the .html; page labels rendered as [*N].
- `lashley-5cir-1975.html` sha256: `590273feeb0b77068a181d219e7db02614e869eaf676266d9a294f4511f7561f`.
- `lashley-5cir-1975.json` sha256: `7dba034b01631dc019dc268bf0f495919a7acc08d353699473f5d46c5eb8a0a7`.
- `lashley-5cir-1975.txt` sha256: `5c022cbeb8fc1acac748a5dffb09b652d9ff2c7e46b5a85a81c6d21d58a83fc2`.
- Manifest id `lashley-5cir-1975`.

### `rulings/cavaliere-11cir-1993.html`, `rulings/cavaliere-11cir-1993.json`, `rulings/cavaliere-11cir-1993.txt`

- Cavaliere v. Allstate Insurance, 996 F.2d 1111 (11th Cir. 1993-07-30), Nos. 92-2678, 92-2711.
- Source: <https://static.case.law/f2d/996/html/1111-01.html>; metadata <https://static.case.law/f2d/996/cases/1111-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:08:03Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `cavaliere-11cir-1993.txt`: tags stripped from the .html; page labels rendered as [*N].
- `cavaliere-11cir-1993.html` sha256: `a892bbe3dfc22f98cbf2478167dc4a08e091f1650ecb547331e7f8d1c1594bef`.
- `cavaliere-11cir-1993.json` sha256: `aa6b49394dfe1a69ba8f25ef36c3a981c4f55fcd5536edc7239d81a3052831f0`.
- `cavaliere-11cir-1993.txt` sha256: `460501b866df40adc5586487576b975b0800fb95f2dcf09f1987d5f1c4edd4fe`.
- Manifest id `cavaliere-11cir-1993`.

### `rulings/parker-10cir-1996.html`, `rulings/parker-10cir-1996.json`, `rulings/parker-10cir-1996.txt`

- Parker v. Board of Public Utilities of Kansas City, 77 F.3d 1289 (10th Cir. 1996-03-06), No. 94-3335.
- Source: <https://static.case.law/f3d/77/html/1289-01.html>; metadata <https://static.case.law/f3d/77/cases/1289-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:08:04Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `parker-10cir-1996.txt`: tags stripped from the .html; page labels rendered as [*N].
- `parker-10cir-1996.html` sha256: `cbbb511d408d06f5e4f47555ef8afa6e3d9c0d34429f049cc399e1a464e0f444`.
- `parker-10cir-1996.json` sha256: `f27b63c127aa3c556f19bf887b5f1cd439e8d36aa45386455ea4fc1d6f8becae`.
- `parker-10cir-1996.txt` sha256: `d32493792bb715aa9e0e10b78ba4a4fcc14770c9fb680d79fe8817d576365761`.
- Manifest id `parker-10cir-1996`.

### `rulings/derrington-bey-dccir-1994.html`, `rulings/derrington-bey-dccir-1994.json`, `rulings/derrington-bey-dccir-1994.txt`

- Derrington-Bey v. District of Columbia Department of Corrections, 39 F.3d 1224 (D.C. Cir. 1994-11-22), No. 94-7123.
- Source: <https://static.case.law/f3d/39/html/1224-01.html>; metadata <https://static.case.law/f3d/39/cases/1224-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:08:05Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `derrington-bey-dccir-1994.txt`: tags stripped from the .html; page labels rendered as [*N].
- `derrington-bey-dccir-1994.html` sha256: `0e95d4ca47c1274d3931002a8f0964f487a5e0f403a007f0bac8cfa5e51c68dc`.
- `derrington-bey-dccir-1994.json` sha256: `bdc6023d2e1c730126cc0b2610f6943dd598adf8510e8c6029ffe8d8119a22c4`.
- `derrington-bey-dccir-1994.txt` sha256: `5db358dc1bb18fe741e9681a82d155416f98f7b749d5de404c8a9970eb43f8ce`.
- Manifest id `derrington-bey-dccir-1994`.

### `rulings/hong-5cir-1997.html`, `rulings/hong-5cir-1997.json`, `rulings/hong-5cir-1997.txt`

- Yong Un Hong v. Smith, 129 F.3d 824 (5th Cir. 1997-12-09), No. 96-50843.
- Source: <https://static.case.law/f3d/129/html/0824-01.html>; metadata <https://static.case.law/f3d/129/cases/0824-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:08:59Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `hong-5cir-1997.txt`: tags stripped from the .html; page labels rendered as [*N].
- `hong-5cir-1997.html` sha256: `15fdc6edcba6dd8d23c66c59204c56f14fcb2adf4126f3e1e93a0b9df45e030d`.
- `hong-5cir-1997.json` sha256: `3cecc73060c39ab17e515476ef00ee182a4342b17e2cfdc4c4110231db042bed`.
- `hong-5cir-1997.txt` sha256: `658e6a502e12093c73e9e16042e559598d55bef6627b721aecd950d36a7d96cb`.
- Manifest id `hong-5cir-1997`.

### `rulings/halicki-5cir-1998.html`, `rulings/halicki-5cir-1998.json`, `rulings/halicki-5cir-1998.txt`

- Halicki v. Louisiana Casino Cruises, Inc., 151 F.3d 465 (5th Cir. 1998-09-01), No. 97-30508.
- Source: <https://static.case.law/f3d/151/html/0465-01.html>; metadata <https://static.case.law/f3d/151/cases/0465-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:08:04Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `halicki-5cir-1998.txt`: tags stripped from the .html; page labels rendered as [*N].
- `halicki-5cir-1998.html` sha256: `292eeefde493c73f83c93f1295dbd5587b32b83305ca05449c9aaca411cd30f1`.
- `halicki-5cir-1998.json` sha256: `6759003920d473d0895efd8314117b5e9805e9aaeba279fa4698e330c17bd7e7`.
- `halicki-5cir-1998.txt` sha256: `793bab4401447f8490f1bcd37978b5c5e200453ae0f66f8bd1aa0883cc6d9b7e`.
- Manifest id `halicki-5cir-1998`.

### `rulings/midwest-employers-5cir-1998.html`, `rulings/midwest-employers-5cir-1998.json`, `rulings/midwest-employers-5cir-1998.txt`

- Midwest Employers Casualty Co. v. Williams, 161 F.3d 877 (5th Cir. 1998-11-24), No. 97-31325.
- Source: <https://static.case.law/f3d/161/html/0877-01.html>; metadata <https://static.case.law/f3d/161/cases/0877-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:08:09Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `midwest-employers-5cir-1998.txt`: tags stripped from the .html; page labels rendered as [*N].
- `midwest-employers-5cir-1998.html` sha256: `a94f6f09778024697fad004c541cdf4f2de39eaf6626be18d6a0f33d32abf900`.
- `midwest-employers-5cir-1998.json` sha256: `69609310dfde8e0a75d1a18cc9a31f7f04993316d9dc66c0dd5346764e1d6ca1`.
- `midwest-employers-5cir-1998.txt` sha256: `d2dbd3b60a76307f98a607b133d3f1e9e521d464051810f4d24ef9099418e757`.
- Manifest id `midwest-employers-5cir-1998`.

### `rulings/albright-3cir-2001.html`, `rulings/albright-3cir-2001.json`, `rulings/albright-3cir-2001.txt`

- Albright v. Virtue, 273 F.3d 564 (3d Cir. 2001-12-06), No. 00-4279.
- Source: <https://static.case.law/f3d/273/html/0564-01.html>; metadata <https://static.case.law/f3d/273/cases/0564-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:09:56Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `albright-3cir-2001.txt`: tags stripped from the .html; page labels rendered as [*N].
- `albright-3cir-2001.html` sha256: `0b16569ed4b8f2c27491dcb9fbec7207d2f2fce29f3b3917925c0073a6bed48e`.
- `albright-3cir-2001.json` sha256: `0536c99414c315bd93d87ce655549530f041d4d53fe8787f4fd02a672f2b2d04`.
- `albright-3cir-2001.txt` sha256: `e120933693293f0e2b8a8aa90a674dfcee6e7e19441bca47609c11af1ff670a5`.
- Manifest id `albright-3cir-2001`.

### `rulings/jackson-v-crosby-11cir-2004.html`, `rulings/jackson-v-crosby-11cir-2004.json`, `rulings/jackson-v-crosby-11cir-2004.txt`

- Jackson v. Crosby, 375 F.3d 1291 (11th Cir. 2004-07-09), No. 04-10986.
- Source: <https://static.case.law/f3d/375/html/1291-01.html>; metadata <https://static.case.law/f3d/375/cases/1291-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:09:55Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `jackson-v-crosby-11cir-2004.txt`: tags stripped from the .html; page labels rendered as [*N].
- `jackson-v-crosby-11cir-2004.html` sha256: `b16693d0707776b327538cb3355ba96f8b99e11ff1dbd55f47b1bbb0484e0d14`.
- `jackson-v-crosby-11cir-2004.json` sha256: `6620ac9b7ab9f66951d79814dd44134131810bedd7e88964457da10855f20cbb`.
- `jackson-v-crosby-11cir-2004.txt` sha256: `0c7e1967a874c95b60180f20fe8792b17fe10a45739ff15a00d82b5cde6de988`.
- Manifest id `jackson-v-crosby-11cir-2004`.

### `rulings/kyle-9cir-1994.html`, `rulings/kyle-9cir-1994.json`, `rulings/kyle-9cir-1994.txt`

- Kyle v. Campbell Soup Co., 28 F.3d 928 (9th Cir. 1994-01-04), No. 92-16826.
- Source: <https://static.case.law/f3d/28/html/0928-01.html>; metadata <https://static.case.law/f3d/28/cases/0928-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:08:08Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `kyle-9cir-1994.txt`: tags stripped from the .html; page labels rendered as [*N].
- `kyle-9cir-1994.html` sha256: `fb43a612727c4e6d41290c5c0f74f7b6a1c8a3b9a5095d38e7bca34c215e9b29`.
- `kyle-9cir-1994.json` sha256: `e82777253ab65be147696a1690122b601dbb86ce4c8c4f2bc4e540f4cc792ace`.
- `kyle-9cir-1994.txt` sha256: `cae4293573ceff7f249ea2d0361a5e562f28d1aae81201377bdc661ec4a4b293`.
- Manifest id `kyle-9cir-1994`.

### `rulings/bowles-scotus-2007.pdf`, `rulings/bowles-scotus-2007.txt`

- Bowles v. Russell, 551 U.S. 205 (U.S. 2007-06-14), 06-5306.
- Source: <https://www.govinfo.gov/content/pkg/USREPORTS-551/pdf/USREPORTS-551-205.pdf>.
- Retrieved 2026-10-23 (2026-10-23T07:32:30Z). Downloaded as served (Official U.S. Reports volume, GPO); `bowles-scotus-2007.txt`: pdftotext -layout of the stored PDF.
- `bowles-scotus-2007.pdf` sha256: `92cad533fabb0f30b1f5998ec0b70e623fc7e0662d9d434dbc002feb78d554f3`.
- `bowles-scotus-2007.txt` sha256: `2216e8f6185a31a7ee2757d4eee47744ebe3fa74a68a0d87d25297f6a7cda923`.
- Manifest id `bowles-scotus-2007`.

### `rulings/ultimate-appliance-6cir-2010.html`, `rulings/ultimate-appliance-6cir-2010.json`, `rulings/ultimate-appliance-6cir-2010.txt`

- Ultimate Appliance CC v. Kirby Co., 601 F.3d 414 (6th Cir. 2010-04-01), No. 09-3297.
- Source: <https://static.case.law/f3d/601/html/0414-01.html>; metadata <https://static.case.law/f3d/601/cases/0414-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:09:59Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `ultimate-appliance-6cir-2010.txt`: tags stripped from the .html; page labels rendered as [*N].
- `ultimate-appliance-6cir-2010.html` sha256: `2e36c586fb2142df55c733ef6534938fde825865dae656cc5a5f4e00259b39de`.
- `ultimate-appliance-6cir-2010.json` sha256: `8e52f3c096549aba3747de59e47cab1c23a1497a6313ebed70d6084349421d1d`.
- `ultimate-appliance-6cir-2010.txt` sha256: `3270f3db61a586347c45e4e3997414fe31656b355001373d84dfac093f801c89`.
- Manifest id `ultimate-appliance-6cir-2010`.

### `rulings/benavides-dccir-1996.html`, `rulings/benavides-dccir-1996.json`, `rulings/benavides-dccir-1996.txt`

- Benavides v. Bureau of Prisons, 79 F.3d 1211 (D.C. Cir. 1996-04-02), No. 95-5147.
- Source: <https://static.case.law/f3d/79/html/1211-01.html>; metadata <https://static.case.law/f3d/79/cases/1211-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:08:02Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `benavides-dccir-1996.txt`: tags stripped from the .html; page labels rendered as [*N].
- `benavides-dccir-1996.html` sha256: `e3d2d09da4a5bf49a27a21beb21ef5944cf9b780c6b2566fac5c3842ad44ca92`.
- `benavides-dccir-1996.json` sha256: `52245fd5dd98181085c76946faee660d8d0597ab6c7d6131a52f59537f12cbb6`.
- `benavides-dccir-1996.txt` sha256: `2851b99909bdd1a9778b13ca188d09a7eae2ba4afdb5b0b1ab72e7b91bfcabf9`.
- Manifest id `benavides-dccir-1996`.

### `rulings/ryan-2cir-1999.html`, `rulings/ryan-2cir-1999.json`, `rulings/ryan-2cir-1999.txt`

- Ryan v. First Unum Life Insurance, 174 F.3d 302 (2d Cir. 1999-05-14), No. 98-9321.
- Source: <https://static.case.law/f3d/174/html/0302-01.html>; metadata <https://static.case.law/f3d/174/cases/0302-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:08:02Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `ryan-2cir-1999.txt`: tags stripped from the .html; page labels rendered as [*N].
- `ryan-2cir-1999.html` sha256: `8429e037e69f3e8384d281da2450c7b60cef46a4b66eb9397110c96b97baf3eb`.
- `ryan-2cir-1999.json` sha256: `6126b2a6b252e411127a812b4542ca8d621f71de84a473aa3bbfa2208c8c6dc4`.
- `ryan-2cir-1999.txt` sha256: `0f558bfcccc8a5b040a7ed65ca9626ec06ca2d2e2c16e5afadec253c22ea057a`.
- Manifest id `ryan-2cir-1999`.

### `rulings/bass-5cir-2000.html`, `rulings/bass-5cir-2000.json`, `rulings/bass-5cir-2000.txt`

- Bass v. United States Department of Agriculture, 211 F.3d 959 (5th Cir. 2000-05-23), No. 99-60366.
- Source: <https://static.case.law/f3d/211/html/0959-01.html>; metadata <https://static.case.law/f3d/211/cases/0959-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:08:06Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `bass-5cir-2000.txt`: tags stripped from the .html; page labels rendered as [*N].
- `bass-5cir-2000.html` sha256: `1f413eae55e4cbc6152413ee4f1405961485925c4a48e1343ee92bdcd1a77ff3`.
- `bass-5cir-2000.json` sha256: `da2e221bb80dff2f62fc8142a4c8fa1c301d462638e8fb4b3a538b3b655fc06a`.
- `bass-5cir-2000.txt` sha256: `9c30747ec5c073a1e4f0c69afe1ce510356445b5fac0d05f7466de6c61916d5f`.
- Manifest id `bass-5cir-2000`.

### `rulings/wilkens-5cir-2001.html`, `rulings/wilkens-5cir-2001.json`, `rulings/wilkens-5cir-2001.txt`

- Wilkens v. Johnson, 238 F.3d 328 (5th Cir. 2001-01-04), No. 99-41180.
- Source: <https://static.case.law/f3d/238/html/0328-01.html>; metadata <https://static.case.law/f3d/238/cases/0328-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:09:54Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `wilkens-5cir-2001.txt`: tags stripped from the .html; page labels rendered as [*N].
- `wilkens-5cir-2001.html` sha256: `7af0f3bc5341d3e570d0139a7c5ae22e1a313973442f52c6ac45a83eb3d45dd8`.
- `wilkens-5cir-2001.json` sha256: `ca7704b8215c86117fe81b62cbdfbf3014052dab7daba77218b2f7cb18e0e36d`.
- `wilkens-5cir-2001.txt` sha256: `64e8bd34fa9f4816f993c660ad9462f865911cedb4134403535ce869e6ada958`.
- Manifest id `wilkens-5cir-2001`.

### `rulings/vencor-11cir-2002.html`, `rulings/vencor-11cir-2002.json`, `rulings/vencor-11cir-2002.txt`

- Vencor Hospitals, Inc. v. Standard Life & Accident Insurance, 279 F.3d 1306 (11th Cir. 2002-01-24), No. 00-16345.
- Source: <https://static.case.law/f3d/279/html/1306-01.html>; metadata <https://static.case.law/f3d/279/cases/1306-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:09:55Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `vencor-11cir-2002.txt`: tags stripped from the .html; page labels rendered as [*N].
- `vencor-11cir-2002.html` sha256: `9683612e68284e8bdd32b793ea05281fb660b5504709b63350ad4fa1aa163181`.
- `vencor-11cir-2002.json` sha256: `3ce210f21f13edd432c4880a866f4133fc7cfe9096c757eee9042b2690ed0088`.
- `vencor-11cir-2002.txt` sha256: `7f8aca5cf1b543a504395e898cbd26a459241388cb6af9355baa3c8aeaa181c6`.
- Manifest id `vencor-11cir-2002`.

### `rulings/flanagan-5cir-1998.html`, `rulings/flanagan-5cir-1998.json`, `rulings/flanagan-5cir-1998.txt`

- Flanagan v. Johnson, 154 F.3d 196 (5th Cir. 1998-09-01), No. 97-10912.
- Source: <https://static.case.law/f3d/154/html/0196-01.html>; metadata <https://static.case.law/f3d/154/cases/0196-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:08:02Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `flanagan-5cir-1998.txt`: tags stripped from the .html; page labels rendered as [*N].
- `flanagan-5cir-1998.html` sha256: `3bbed1cf003a70c7f23eac65fcc808f7f080384dd8a2671772ccf43610aec7a0`.
- `flanagan-5cir-1998.json` sha256: `09c073b1b35960b217bb154e0c26f0767184bf5a9ba8153a4ca1dfb320880b4c`.
- `flanagan-5cir-1998.txt` sha256: `20381d072da4ddc6f1054bf1b431ebec38302d06a025c2d24d13746ed5c31b9c`.
- Manifest id `flanagan-5cir-1998`.

### `rulings/ross-2cir-1998.html`, `rulings/ross-2cir-1998.json`, `rulings/ross-2cir-1998.txt`

- Ross v. Artuz, 150 F.3d 97 (2d Cir. 1998-06-24), No. 97-2789.
- Source: <https://static.case.law/f3d/150/html/0097-01.html>; metadata <https://static.case.law/f3d/150/cases/0097-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:13:56Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `ross-2cir-1998.txt`: tags stripped from the .html; page labels rendered as [*N].
- `ross-2cir-1998.html` sha256: `2fa5e1f7c532c8cde5cffb776d391bf9d05023cf64de18ba49d6ffb18d0d19b0`.
- `ross-2cir-1998.json` sha256: `53127d9ca241bdc4512705559a5ecf4d3eadff42aa1feee80f4aeec764aedd77`.
- `ross-2cir-1998.txt` sha256: `0164f5ff6d9b6876bc6f32d9a0ee62684285c598d20b9c22426ee45ff2d8c505`.
- Manifest id `ross-2cir-1998`.

### `rulings/hernandez-4cir-2000.html`, `rulings/hernandez-4cir-2000.json`, `rulings/hernandez-4cir-2000.txt`

- Hernandez v. Caldwell, 225 F.3d 435 (4th Cir. 2000-08-28), No. 98-7640.
- Source: <https://static.case.law/f3d/225/html/0435-01.html>; metadata <https://static.case.law/f3d/225/cases/0435-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:08:01Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `hernandez-4cir-2000.txt`: tags stripped from the .html; page labels rendered as [*N].
- `hernandez-4cir-2000.html` sha256: `122540be82562190131c11277f3038cf88fd6c2d41aef5bf2285d0446c665444`.
- `hernandez-4cir-2000.json` sha256: `e546fefa2966c4b43c2368802529cb6968998a095bfaf9fa26cc8bac29f54cc3`.
- `hernandez-4cir-2000.txt` sha256: `4fa808977175a49bf864df40a5908b48bbb1df079251533babcefe77a42ea1f9`.
- Manifest id `hernandez-4cir-2000`.

### `rulings/patterson-9cir-2001.html`, `rulings/patterson-9cir-2001.json`, `rulings/patterson-9cir-2001.txt`

- Patterson v. Stewart, 251 F.3d 1243 (9th Cir. 2001-05-30), No. 00-15034.
- Source: <https://static.case.law/f3d/251/html/1243-01.html>; metadata <https://static.case.law/f3d/251/cases/1243-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:08:01Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `patterson-9cir-2001.txt`: tags stripped from the .html; page labels rendered as [*N].
- `patterson-9cir-2001.html` sha256: `ad0363d67dbaf74a0398555d655d03682c799e26b3a602a1916eda608159e4b6`.
- `patterson-9cir-2001.json` sha256: `2f051e59d2b6cbc478ba0e136f799636c2cd38a7b0979d4d059f7af4afddba2a`.
- `patterson-9cir-2001.txt` sha256: `4c38525386e4c41acd9bd719a70e4acbba4b37d8bdd7776453ba20b5ed98c404`.
- Manifest id `patterson-9cir-2001`.

### `rulings/moore-8cir-1999.html`, `rulings/moore-8cir-1999.json`, `rulings/moore-8cir-1999.txt`

- Moore v. United States, 173 F.3d 1131 (8th Cir. 1999-04-16), No. 98-1153.
- Source: <https://static.case.law/f3d/173/html/1131-01.html>; metadata <https://static.case.law/f3d/173/cases/1131-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:08:10Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `moore-8cir-1999.txt`: tags stripped from the .html; page labels rendered as [*N].
- `moore-8cir-1999.html` sha256: `ea9fb00d5eb8b7e5adb09cab730338bde78ae10d95a429e7992431f1d44b39e8`.
- `moore-8cir-1999.json` sha256: `1c09355b19c482f6499c3251273822cc2bbc577cb0cafd6646cde5d3eb5f1408`.
- `moore-8cir-1999.txt` sha256: `aa982e4b15d6185fdf2c5c3b5d5cb05b517ed375817cca93623ae6d237a1f28e`.
- Manifest id `moore-8cir-1999`.

### `rulings/mickens-2cir-1998.html`, `rulings/mickens-2cir-1998.json`, `rulings/mickens-2cir-1998.txt`

- Mickens v. United States, 148 F.3d 145 (2d Cir. 1998-06-24), No. 97-2734.
- Source: <https://static.case.law/f3d/148/html/0145-01.html>; metadata <https://static.case.law/f3d/148/cases/0145-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:08:10Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `mickens-2cir-1998.txt`: tags stripped from the .html; page labels rendered as [*N].
- `mickens-2cir-1998.html` sha256: `73219046aaf6b38313f94055686b77a2793338b1d4c7c2843568f79b0a73291d`.
- `mickens-2cir-1998.json` sha256: `e5e08b3f31e095b207f270ba7dd6940fddb54def08be081232492d1ebf53e631`.
- `mickens-2cir-1998.txt` sha256: `151afdb5094cb3626442be66d7057416b6dd5a817703365b7946e24457ff67fe`.
- Manifest id `mickens-2cir-1998`.

### `rulings/brown-v-angelone-4cir-1998.html`, `rulings/brown-v-angelone-4cir-1998.json`, `rulings/brown-v-angelone-4cir-1998.txt`

- Brown v. Angelone, 150 F.3d 370 (4th Cir. 1998-07-14), Nos. 96-7173, 96-7208.
- Source: <https://static.case.law/f3d/150/html/0370-01.html>; metadata <https://static.case.law/f3d/150/cases/0370-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:10:27Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `brown-v-angelone-4cir-1998.txt`: tags stripped from the .html; page labels rendered as [*N].
- `brown-v-angelone-4cir-1998.html` sha256: `1a8e66e4f9d9810ebbd67c73e26676f359bc12a62f756c288f1d8f1496fd6664`.
- `brown-v-angelone-4cir-1998.json` sha256: `688dba4e179904383b53a79096241a1be28411b5a91813ccdb70470b5c920954`.
- `brown-v-angelone-4cir-1998.txt` sha256: `ce1d63a39ccd2acb385f49a233d7c8e8009f2ffc9c43a15d89a216bebf3dfcfd`.
- Manifest id `brown-v-angelone-4cir-1998`.

### `rulings/burns-3cir-1998.html`, `rulings/burns-3cir-1998.json`, `rulings/burns-3cir-1998.txt`

- Burns v. Morton, 134 F.3d 109 (3d Cir. 1998-01-09), No. 97-5568.
- Source: <https://static.case.law/f3d/134/html/0109-01.html>; metadata <https://static.case.law/f3d/134/cases/0109-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:13:56Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `burns-3cir-1998.txt`: tags stripped from the .html; page labels rendered as [*N].
- `burns-3cir-1998.html` sha256: `98f84106f32e8b651c414ff632a677aa6c81990a3381c4be73cc5b4a971ea06c`.
- `burns-3cir-1998.json` sha256: `3343f593045c26804a63e38c3f1a4c809c7917fd6bdf938ff8afa04566ecc534`.
- `burns-3cir-1998.txt` sha256: `73a97a3f74fec01783f7d0b27f0ea64fde029a7bd827170213cea4ea10360458`.
- Manifest id `burns-3cir-1998`.

### `rulings/fletcher-5cir-2000.html`, `rulings/fletcher-5cir-2000.json`, `rulings/fletcher-5cir-2000.txt`

- Fletcher v. Apfel, 210 F.3d 510 (5th Cir. 2000-04-21), No. 98-20761.
- Source: <https://static.case.law/f3d/210/html/0510-01.html>; metadata <https://static.case.law/f3d/210/cases/0510-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:09:56Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `fletcher-5cir-2000.txt`: tags stripped from the .html; page labels rendered as [*N].
- `fletcher-5cir-2000.html` sha256: `e0ca66a026cc989ff1eb0c58b963ab7b35662b4bb724d550233485f1484fa617`.
- `fletcher-5cir-2000.json` sha256: `bb7968d20b8d94fa7dbba0e4cefcce9f5a81ed348a1fa29b5dbc9098737734e7`.
- `fletcher-5cir-2000.txt` sha256: `018f0d5777b30dc90c3d02d90c827969571b2f27a8acee9c6d37a4c94486bccf`.
- Manifest id `fletcher-5cir-2000`.

### `rulings/eastman-3cir-2013.html`, `rulings/eastman-3cir-2013.json`, `rulings/eastman-3cir-2013.txt`

- Eastman v. First Data Corp., 736 F.3d 675 (3d Cir. 2013-12-04), No. 13-8071.
- Source: <https://static.case.law/f3d/736/html/0675-01.html>; metadata <https://static.case.law/f3d/736/cases/0675-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:08:47Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `eastman-3cir-2013.txt`: tags stripped from the .html; page labels rendered as [*N].
- `eastman-3cir-2013.html` sha256: `8a61ac12a2d3841e88272754d7a82a7831d35cd642febb4d38239b1d0d08f173`.
- `eastman-3cir-2013.json` sha256: `0b183828d0c8e8c2c067c9c16249206839d3a91505c5176f27b435a9ffb3a6f8`.
- `eastman-3cir-2013.txt` sha256: `320eae433acd0541055c5d0bdefb8a6444a69be75f8eb4dbc541259539fa4b5f`.
- Manifest id `eastman-3cir-2013`.

### `rulings/serna-5cir-2013.html`, `rulings/serna-5cir-2013.json`, `rulings/serna-5cir-2013.txt`

- Serna v. Law Office of Joseph Onwut-Eaka, P.C., 732 F.3d 440 (5th Cir. 2013-10-07), No. 12-20529.
- Source: <https://static.case.law/f3d/732/html/0440-01.html>; metadata <https://static.case.law/f3d/732/cases/0440-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:09:57Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `serna-5cir-2013.txt`: tags stripped from the .html; page labels rendered as [*N].
- `serna-5cir-2013.html` sha256: `02b394d7c944ab96c62e1039033ae20203bca592385faf4334644d0437255813`.
- `serna-5cir-2013.json` sha256: `334eebac9bc61bc361ca488dcff43d71365b44d90056b9c94f10879f8f09b29a`.
- `serna-5cir-2013.txt` sha256: `d129bbc6c8eb7b17bfe50bc8283240b1f9dae4fb8cf2c57f5c23d792e10c5836`.
- Manifest id `serna-5cir-2013`.

### `rulings/justice-7cir-2012.html`, `rulings/justice-7cir-2012.json`, `rulings/justice-7cir-2012.txt`

- Justice v. Town of Cicero, 682 F.3d 662 (7th Cir. 2012-06-05), No. 11-3876.
- Source: <https://static.case.law/f3d/682/html/0662-01.html>; metadata <https://static.case.law/f3d/682/cases/0662-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:09:58Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `justice-7cir-2012.txt`: tags stripped from the .html; page labels rendered as [*N].
- `justice-7cir-2012.html` sha256: `5138c3dd604b1abe681dc91b0ff67cdbf94b0aadde1f57d07098a8e300db102d`.
- `justice-7cir-2012.json` sha256: `5d71f165f420bfd25c859ccbaa1542c54dd7481ead315adbd29447d1d2197229`.
- `justice-7cir-2012.txt` sha256: `54f3aa9d91fa4686d292fafbc9de481cece0503cb4fedc965756f40fb071514c`.
- Manifest id `justice-7cir-2012`.

### `rulings/lorenzo-4cir-2015.html`, `rulings/lorenzo-4cir-2015.json`, `rulings/lorenzo-4cir-2015.txt`

- Lorenzo v. Prime Communications, L.P., 806 F.3d 777 (4th Cir. 2015-11-24), Nos. 14-1622, 14-1727.
- Source: <https://static.case.law/f3d/806/html/0777-01.html>; metadata <https://static.case.law/f3d/806/cases/0777-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:09:57Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `lorenzo-4cir-2015.txt`: tags stripped from the .html; page labels rendered as [*N].
- `lorenzo-4cir-2015.html` sha256: `1b24c10d7341d2c852c26d42709eee346a66fce1634630251453a3b197bffab2`.
- `lorenzo-4cir-2015.json` sha256: `0a73731a701037f7d0cafea566e80f143d2db0595e2eb3e21e6418a57b32b80b`.
- `lorenzo-4cir-2015.txt` sha256: `bf4511df1957f8fe2aa0c971ccf555f8d05b57cf20086e3431ea8b115afc1e40`.
- Manifest id `lorenzo-4cir-2015`.

### `rulings/ramos-lopez-5cir-2016.html`, `rulings/ramos-lopez-5cir-2016.json`, `rulings/ramos-lopez-5cir-2016.txt`

- Ramos-Lopez v. Lynch, 823 F.3d 1024 (5th Cir. 2016-05-24), No. 14-60753.
- Source: <https://static.case.law/f3d/823/html/1024-01.html>; metadata <https://static.case.law/f3d/823/cases/1024-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:09:58Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `ramos-lopez-5cir-2016.txt`: tags stripped from the .html; page labels rendered as [*N].
- `ramos-lopez-5cir-2016.html` sha256: `8472e8fb089a76395b4fcb23c310fdfbcf04a0f30a416cba3359f8815e3279c7`.
- `ramos-lopez-5cir-2016.json` sha256: `5ad1cc1129e03902e23f23df3acdaf7f0df7c126fb11abfe5451aff9666d3701`.
- `ramos-lopez-5cir-2016.txt` sha256: `7f968d71812160b5e4b440b9b5f45320faa9972652c99dc0aae95cbbf561b326`.
- Manifest id `ramos-lopez-5cir-2016`.

### `rulings/avolio-2cir-1994.html`, `rulings/avolio-2cir-1994.json`, `rulings/avolio-2cir-1994.txt`

- Avolio v. County of Suffolk, 29 F.3d 50 (2d Cir. 1994-07-06), No. 691, Docket 93-7649.
- Source: <https://static.case.law/f3d/29/html/0050-01.html>; metadata <https://static.case.law/f3d/29/cases/0050-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:13:54Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `avolio-2cir-1994.txt`: tags stripped from the .html; page labels rendered as [*N].
- `avolio-2cir-1994.html` sha256: `406034c04f257955d6258829d7cd9beb6af010d6feed80b0cecaeec71a57465d`.
- `avolio-2cir-1994.json` sha256: `0548605224d61b9a4259e4ed9a25712d906b171ca7584c9208935881e0efc1f9`.
- `avolio-2cir-1994.txt` sha256: `33aeaa7da2a1400d107cd73e3fa79f81a42e0d0687ec14db073c0c89fcd421fb`.
- Manifest id `avolio-2cir-1994`.

### `rulings/scott-harris-1cir-1997.html`, `rulings/scott-harris-1cir-1997.json`, `rulings/scott-harris-1cir-1997.txt`

- Scott-Harris v. City of Fall River, 134 F.3d 427 (1st Cir. 1997-01-15), Nos. 95-1950 to 95-1952 and 95-2100.
- Source: <https://static.case.law/f3d/134/html/0427-01.html>; metadata <https://static.case.law/f3d/134/cases/0427-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:13:55Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `scott-harris-1cir-1997.txt`: tags stripped from the .html; page labels rendered as [*N].
- `scott-harris-1cir-1997.html` sha256: `ba09df0c46e170b9eb728d8b42bdbe1db3c724ec6653f77a40a209f67ee574ff`.
- `scott-harris-1cir-1997.json` sha256: `181f9dada5cfca4a460dcc73e419c60cd8a3fc8499fd5cc1b5b577dfc42eca9c`.
- `scott-harris-1cir-1997.txt` sha256: `e4fb873953b17b59d698e6523039069a223dd2d4d0381f7457ee2583c204d053`.
- Manifest id `scott-harris-1cir-1997`.

### `rulings/nunley-9cir-1995.html`, `rulings/nunley-9cir-1995.json`, `rulings/nunley-9cir-1995.txt`

- Nunley v. City of Los Angeles, 52 F.3d 792 (9th Cir. 1995-04-06), Nos. 93-56110, 93-56166.
- Source: <https://static.case.law/f3d/52/html/0792-01.html>; metadata <https://static.case.law/f3d/52/cases/0792-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:13:55Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `nunley-9cir-1995.txt`: tags stripped from the .html; page labels rendered as [*N].
- `nunley-9cir-1995.html` sha256: `68a2a7d7bbf90d44233ad9109e2a4592c28dfdee2c69f93e858e8028894a2cff`.
- `nunley-9cir-1995.json` sha256: `3c5b82b0adda3f66a7117fbb280fd00aeaedf3647301adc05fb9539f2fff7ae3`.
- `nunley-9cir-1995.txt` sha256: `d9c766ee0d968b07143bd631832b86eb42a90732a606c743901451da54c3f420`.
- Manifest id `nunley-9cir-1995`.

### `rulings/zimmer-8cir-1994.html`, `rulings/zimmer-8cir-1994.json`, `rulings/zimmer-8cir-1994.txt`

- Zimmer St. Louis, Inc. v. Zimmer Co., 32 F.3d 357 (8th Cir. 1994-08-11), Nos. 93-3796, 93-3951.
- Source: <https://static.case.law/f3d/32/html/0357-01.html>; metadata <https://static.case.law/f3d/32/cases/0357-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:13:55Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `zimmer-8cir-1994.txt`: tags stripped from the .html; page labels rendered as [*N].
- `zimmer-8cir-1994.html` sha256: `1911a2b467b7326de53ee30a5901bf0b9a9f91b1ecde96a45b0b1d078529da58`.
- `zimmer-8cir-1994.json` sha256: `b49b996cfb1af556f7fee102b504327bc8331c22cc453bdb8edc303e9b715ce2`.
- `zimmer-8cir-1994.txt` sha256: `4b26a0392628482d1d24478ac347b80a9b6f6fc7f3ebbebc971882594a096948`.
- Manifest id `zimmer-8cir-1994`.

### `rulings/miller-v-marr-10cir-1998.html`, `rulings/miller-v-marr-10cir-1998.json`, `rulings/miller-v-marr-10cir-1998.txt`

- Miller v. Marr, 141 F.3d 976 (10th Cir. 1998-04-14), No. 97-1380.
- Source: <https://static.case.law/f3d/141/html/0976-01.html>; metadata <https://static.case.law/f3d/141/cases/0976-01.json>.
- Retrieved 2026-10-23 (2026-10-23T08:13:55Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `miller-v-marr-10cir-1998.txt`: tags stripped from the .html; page labels rendered as [*N].
- `miller-v-marr-10cir-1998.html` sha256: `ec04ebbd2bd326103ffa523883bd25beba5d87751f9f9fd05081bca6ec2c1450`.
- `miller-v-marr-10cir-1998.json` sha256: `c96bd6b4b1fcaa94fd2cf5eed3efb7a61e4a062d3e88b32d0e1c2a5aae21ef3a`.
- `miller-v-marr-10cir-1998.txt` sha256: `5e800e791519ffe05aee26919a2a55e0cef1018b828c49e29409ff1590a38bf2`.
- Manifest id `miller-v-marr-10cir-1998`.

### `rulings/pierce-6cir-2021.pdf`, `rulings/pierce-6cir-2021.txt`

- Gerald Pierce v. Ocwen Loan Servicing, LLC, 987 F.3d 577 (6th Cir. 2021-02-04), 20-6057.
- Source: <https://storage.courtlistener.com/pdf/2021/02/04/gerald_pierce_v._ocwen_loan_servicing_llc.pdf>; opinion page <https://www.courtlistener.com/opinion/4853667/gerald-pierce-v-ocwen-loan-servicing-llc/>.
- Retrieved 2026-10-23 (2026-10-23T08:43:06Z). Downloaded as served (CourtListener storage copy of the court-issued PDF); `pierce-6cir-2021.txt`: pdftotext -layout of the stored PDF.
- `pierce-6cir-2021.pdf` sha256: `0d2b2d963db078ae620b20ef4cf312b03816138ca083546e205464a1d1e55937`.
- `pierce-6cir-2021.txt` sha256: `7305d3b18d2e99c9634be5c4c909e0377d30540b57782b366ffc37c21a1173cd`.
- Manifest id `pierce-6cir-2021`.

### `rulings/goncalves-1cir-2025.pdf`, `rulings/goncalves-1cir-2025.txt`

- Goncalves v. Bondi, 138 F.4th 58 (1st Cir. 2025-05-20), 24-1511.
- Source: <https://storage.courtlistener.com/pdf/2025/05/20/goncalves_v._bondi.pdf>; opinion page <https://www.courtlistener.com/opinion/10587505/goncalves-v-bondi/>.
- Retrieved 2026-10-23 (2026-10-23T08:43:23Z). Downloaded as served (CourtListener storage copy of the court-issued PDF); `goncalves-1cir-2025.txt`: pdftotext -layout of the stored PDF.
- `goncalves-1cir-2025.pdf` sha256: `e3f3c9f8f4481f0cdfe5463f80531d4eadff587a6c588b0a755d14f929d30f36`.
- `goncalves-1cir-2025.txt` sha256: `afb6886fefb1881e56a5a9edc211d8b50c73150a2fb1cd85e9ca69c0ea192368`.
- Manifest id `goncalves-1cir-2025`.

### `rulings/circuitronix-11cir-2021.pdf`, `rulings/circuitronix-11cir-2021.txt`

- Circuitronix, LLC v. Shenzen Kinwong Electronic Co., Ltd., 993 F.3d 1299 (11th Cir. 2021-04-08), 19-12547.
- Source: <https://storage.courtlistener.com/pdf/2021/04/08/circuitronix_llc_v._shenzen_kinwong_electronic_co._ltd..pdf>; opinion page <https://www.courtlistener.com/opinion/4871962/circuitronix-llc-v-shenzen-kinwong-electronic-co-ltd/>.
- Retrieved 2026-10-23 (2026-10-23T08:46:18Z). Downloaded as served (CourtListener storage copy of the court-issued PDF); `circuitronix-11cir-2021.txt`: pdftotext -layout of the stored PDF.
- `circuitronix-11cir-2021.pdf` sha256: `445251f138e6ff79fdc1a75f8d73b46e67e34f32df18b7c4f9c56b3a2eab47ac`.
- `circuitronix-11cir-2021.txt` sha256: `e64c819b40c9a588f17461eb690651be005f431eb0c22497f65cd9ca7ece58bd`.
- Manifest id `circuitronix-11cir-2021`.

### `rulings/blackman-9cir-2024.pdf`, `rulings/blackman-9cir-2024.txt`

- Chance Blackman v. Theresa Cisneros, 122 F.4th 377 (9th Cir. 2024-11-29), 23-55340.
- Source: <https://storage.courtlistener.com/pdf/2024/11/29/chance_blackman_v._theresa_cisneros.pdf>; opinion page <https://www.courtlistener.com/opinion/10286062/chance-blackman-v-theresa-cisneros/>.
- Retrieved 2026-10-23 (2026-10-23T08:44:55Z). Downloaded as served (CourtListener storage copy of the court-issued PDF); `blackman-9cir-2024.txt`: pdftotext -layout of the stored PDF.
- `blackman-9cir-2024.pdf` sha256: `30a5a84dc79bc25639d9d6000aaf7b0b9b2a3af2269f0fb357b58d74cc0600b2`.
- `blackman-9cir-2024.txt` sha256: `c650cd4a5c5298e21b0dd344faf84d061940908b919ce666140b3120d2362a53`.
- Manifest id `blackman-9cir-2024`.

### `rulings/organic-cannabis-9cir-2020.pdf`, `rulings/organic-cannabis-9cir-2020.txt`

- Organic Cannabis Foundation v. Cir, 962 F.3d 1082 (9th Cir. 2020-06-18), 17-72874.
- Source: <https://storage.courtlistener.com/pdf/2020/06/18/organic_cannabis_foundation_v._cir.pdf>; opinion page <https://www.courtlistener.com/opinion/4762110/organic-cannabis-foundation-v-cir/>.
- Retrieved 2026-10-23 (2026-10-23T08:45:27Z). Downloaded as served (CourtListener storage copy of the court-issued PDF); `organic-cannabis-9cir-2020.txt`: pdftotext -layout of the stored PDF.
- `organic-cannabis-9cir-2020.pdf` sha256: `834f574715123f09a8503e2a4a23ffc868250bebbb1a08b3c746531d716657c7`.
- `organic-cannabis-9cir-2020.txt` sha256: `25c05bb5430a688347e8045d6baf88b21209d627dba3099dc12b4aa13ad17327`.
- Manifest id `organic-cannabis-9cir-2020`.

### `rulings/malek-2cir-2024.pdf`, `rulings/malek-2cir-2024.txt`

- Malek v. Feigenbaum, 116 F.4th 118 (2d Cir. 2024-09-11), 23-0992-cv.
- Source: <https://storage.courtlistener.com/pdf/2024/09/11/malek_v._feigenbaum.pdf>; opinion page <https://www.courtlistener.com/opinion/10114043/malek-v-feigenbaum/>.
- Retrieved 2026-10-23 (2026-10-23T08:44:53Z). Downloaded as served (CourtListener storage copy of the court-issued PDF); `malek-2cir-2024.txt`: pdftotext -layout of the stored PDF.
- `malek-2cir-2024.pdf` sha256: `88b837ea13da1a76d533162b46835aea985f1872156e4b3e3a9e9e61840a2e77`.
- `malek-2cir-2024.txt` sha256: `f22550982999b90cc8f8e0be2852817e4c47d7d23daac14371f1c1c9818e440b`.
- Manifest id `malek-2cir-2024`.

## Rulings: Supreme Court decisions of the expect corpus (`rulings/`)

Stored by lg-wvlf.4 and lg-wvlf.6 on 2026-10-28 the way the CAP entries above are; `rulings/MANIFEST.json` has an entry for each (`rulings/README.md`).

### `rulings/arizona-v-washington-scotus-1978.html`, `rulings/arizona-v-washington-scotus-1978.json`, `rulings/arizona-v-washington-scotus-1978.txt`

- Arizona v. Washington, 434 U.S. 497 (U.S. 1978-02-21), No. 76-1168.
- Source: <https://static.case.law/us/434/html/0497-01.html>; metadata <https://static.case.law/us/434/cases/0497-01.json>.
- Retrieved 2026-10-28 (2026-10-28T14:06:10Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `arizona-v-washington-scotus-1978.txt`: tags stripped from the .html; page labels rendered as [*N].
- `arizona-v-washington-scotus-1978.html` sha256: `62e91b97af21253a49af40d5422d1c04b7b41371faab729980a72b2730166402`.
- `arizona-v-washington-scotus-1978.json` sha256: `65a447057a64f99d1c512c3a2a2f5463e2e96f87674cfb6689330aa244af1f06`.
- `arizona-v-washington-scotus-1978.txt` sha256: `9498c625ebe007a1b1d93b2fccee168729322daa4e3a2e167ca5b820d64b7d6d`.
- Manifest id `arizona-v-washington-scotus-1978`.

### `rulings/blockburger-scotus-1932.html`, `rulings/blockburger-scotus-1932.json`, `rulings/blockburger-scotus-1932.txt`

- Blockburger v. United States, 284 U.S. 299 (U.S. 1932-01-04), No. 374.
- Source: <https://static.case.law/us/284/html/0299-01.html>; metadata <https://static.case.law/us/284/cases/0299-01.json>.
- Retrieved 2026-10-28 (2026-10-28T14:06:10Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `blockburger-scotus-1932.txt`: tags stripped from the .html; page labels rendered as [*N].
- `blockburger-scotus-1932.html` sha256: `330ecff0850cedc9be6b3352cf46f2b069c6826c19259cda4ae5b9559c979309`.
- `blockburger-scotus-1932.json` sha256: `173fca35016d98fd445d87decb515c142a315837ec403d4ac7b1e8e2392bba9b`.
- `blockburger-scotus-1932.txt` sha256: `4807c967318bf71686a7dac7037bca7439bd374a3e458df07673f8ee7a20d045`.
- Manifest id `blockburger-scotus-1932`.

### `rulings/brady-scotus-1963.html`, `rulings/brady-scotus-1963.json`, `rulings/brady-scotus-1963.txt`

- Brady v. Maryland, 373 U.S. 83 (U.S. 1963-05-13), No. 490.
- Source: <https://static.case.law/us/373/html/0083-01.html>; metadata <https://static.case.law/us/373/cases/0083-01.json>.
- Retrieved 2026-10-28 (2026-10-28T14:06:11Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `brady-scotus-1963.txt`: tags stripped from the .html; page labels rendered as [*N].
- `brady-scotus-1963.html` sha256: `3a907e498e32719a604db2690d884be26fc20b47fad1934b6b6bd827201ec0d4`.
- `brady-scotus-1963.json` sha256: `7b6ba79157ce0259aa239bdfab15a78bb2ad59aced5750f961ca6946df5342af`.
- `brady-scotus-1963.txt` sha256: `9c63c62c1a7b8e3994df516dadde09bf3fb73f54256b75c2d6d9b462ac0c08cf`.
- Manifest id `brady-scotus-1963`.

### `rulings/burks-scotus-1978.html`, `rulings/burks-scotus-1978.json`, `rulings/burks-scotus-1978.txt`

- Burks v. United States, 437 U.S. 1 (U.S. 1978-06-14), No. 76-6528.
- Source: <https://static.case.law/us/437/html/0001-01.html>; metadata <https://static.case.law/us/437/cases/0001-01.json>.
- Retrieved 2026-10-28 (2026-10-28T14:06:11Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `burks-scotus-1978.txt`: tags stripped from the .html; page labels rendered as [*N].
- `burks-scotus-1978.html` sha256: `29745877ee837814f40f067f24eac01df9383b0581892df37656f705498464ef`.
- `burks-scotus-1978.json` sha256: `a26d6415e108072f6e339f46da00276966194e7b99aaa16671186b515f286eda`.
- `burks-scotus-1978.txt` sha256: `89e9734859b304604da3ccb6d63e327a89f06a8b975eb53dac436b94707e776f`.
- Manifest id `burks-scotus-1978`.

### `rulings/davis-scotus-1973.html`, `rulings/davis-scotus-1973.json`, `rulings/davis-scotus-1973.txt`

- Davis v. United States, 411 U.S. 233 (U.S. 1973-04-17), No. 71-6481.
- Source: <https://static.case.law/us/411/html/0233-01.html>; metadata <https://static.case.law/us/411/cases/0233-01.json>.
- Retrieved 2026-10-28 (2026-10-28T14:06:13Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `davis-scotus-1973.txt`: tags stripped from the .html; page labels rendered as [*N].
- `davis-scotus-1973.html` sha256: `e62ef48f1515dbf6ba1a554252ac09f79fcfcf86b1a32030b0ab14edd459c6aa`.
- `davis-scotus-1973.json` sha256: `d0a63567f80b05d465f0e78ce0d8c7515339096cf5f2811a07eee56f4f7c1a6c`.
- `davis-scotus-1973.txt` sha256: `c5e96baeef249e2b9a33b1d4780bae212792244d665af7697fe9603cb8c544bb`.
- Manifest id `davis-scotus-1973`.

### `rulings/giglio-scotus-1972.html`, `rulings/giglio-scotus-1972.json`, `rulings/giglio-scotus-1972.txt`

- Giglio v. United States, 405 U.S. 150 (U.S. 1972-02-24), No. 70-29.
- Source: <https://static.case.law/us/405/html/0150-01.html>; metadata <https://static.case.law/us/405/cases/0150-01.json>.
- Retrieved 2026-10-28 (2026-10-28T14:06:14Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `giglio-scotus-1972.txt`: tags stripped from the .html; page labels rendered as [*N].
- `giglio-scotus-1972.html` sha256: `1b7bbeb633af75db5fdd6cee8b684c911648581f808210264ff121b270f23266`.
- `giglio-scotus-1972.json` sha256: `270d2d573572dbc99ec50e4ef1ef447901b17fd2dd2d59e49facdd256b55715d`.
- `giglio-scotus-1972.txt` sha256: `43c35fdc0a3255b0f5648dcdbb6439a1c14e1ba953a03ca224adf0a4bb686fa6`.
- Manifest id `giglio-scotus-1972`.

### `rulings/jackson-v-virginia-scotus-1979.html`, `rulings/jackson-v-virginia-scotus-1979.json`, `rulings/jackson-v-virginia-scotus-1979.txt`

- Jackson v. Virginia, 443 U.S. 307 (U.S. 1979-06-28), No. 78-5283.
- Source: <https://static.case.law/us/443/html/0307-01.html>; metadata <https://static.case.law/us/443/cases/0307-01.json>.
- Retrieved 2026-10-28 (2026-10-28T14:06:14Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `jackson-v-virginia-scotus-1979.txt`: tags stripped from the .html; page labels rendered as [*N].
- `jackson-v-virginia-scotus-1979.html` sha256: `82c2a9ec599ea1fa3fa374842321927d97ef695f65bd9790420256d451061478`.
- `jackson-v-virginia-scotus-1979.json` sha256: `2981587f3eb477a6869ea64ec3f0675194ba578ec1d634c8afd9864a11fbb2ed`.
- `jackson-v-virginia-scotus-1979.txt` sha256: `9b5e22fc935a13bb8ddc99386bb690d4ff7c9dd588d69354429c38733fd36566`.
- Manifest id `jackson-v-virginia-scotus-1979`.

### `rulings/luce-scotus-1984.html`, `rulings/luce-scotus-1984.json`, `rulings/luce-scotus-1984.txt`

- Luce v. United States, 469 U.S. 38 (U.S. 1984-12-10), No. 83-912.
- Source: <https://static.case.law/us/469/html/0038-01.html>; metadata <https://static.case.law/us/469/cases/0038-01.json>.
- Retrieved 2026-10-28 (2026-10-28T14:06:15Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `luce-scotus-1984.txt`: tags stripped from the .html; page labels rendered as [*N].
- `luce-scotus-1984.html` sha256: `4c9f34481cf0b29ab82ebddb419b9dc4369127f5eb310e3eae3a6c0c6a36b2ed`.
- `luce-scotus-1984.json` sha256: `725d56fe3e1207db4149436a0554dd31c71c9d6f7bfe8e7aef85cf3fe7834f1e`.
- `luce-scotus-1984.txt` sha256: `df86b5db7ab6affbf981084dc96eb642ab3b6878383ba927c28042dfe6fd9d55`.
- Manifest id `luce-scotus-1984`.

### `rulings/mapp-scotus-1961.html`, `rulings/mapp-scotus-1961.json`, `rulings/mapp-scotus-1961.txt`

- Mapp v. Ohio, 367 U.S. 643 (U.S. 1961-06-19), No. 236.
- Source: <https://static.case.law/us/367/html/0643-01.html>; metadata <https://static.case.law/us/367/cases/0643-01.json>.
- Retrieved 2026-10-28 (2026-10-28T14:06:15Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `mapp-scotus-1961.txt`: tags stripped from the .html; page labels rendered as [*N].
- `mapp-scotus-1961.html` sha256: `3253654902b470cc221a7c856041b3791ed6c6a8143a48756b10ec455d5b34b3`.
- `mapp-scotus-1961.json` sha256: `583b80d79a6cdb6912cd5a943160c069adfcdac815be7ab3085a08e32cee2bab`.
- `mapp-scotus-1961.txt` sha256: `611c744d24b25dd3686f0a3f2d5a3effe48b78ce0fb062110d2a043abd2b6565`.
- Manifest id `mapp-scotus-1961`.

### `rulings/booker-scotus-2005.html`, `rulings/booker-scotus-2005.json`, `rulings/booker-scotus-2005.txt`

- United States v. Booker, 543 U.S. 220 (U.S. 2005-01-12), No. 04-104.
- Source: <https://static.case.law/us/543/html/0220-01.html>; metadata <https://static.case.law/us/543/cases/0220-01.json>.
- Retrieved 2026-10-28 (2026-10-28T14:06:16Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `booker-scotus-2005.txt`: tags stripped from the .html; page labels rendered as [*N].
- `booker-scotus-2005.html` sha256: `1fdd940ceaa15c4446f9e41c5d45f64ee036b272e2f69447bc5d0dec0ca06b10`.
- `booker-scotus-2005.json` sha256: `98692bdb2651163b3055b79ae3e8fcb54129b12c1fe81c1fd61fc19b08fdd0bf`.
- `booker-scotus-2005.txt` sha256: `d6e67fab19caa4c3a0d53c12d7bcde5ad1d6cd550e5f0715571bde922f2b4465`.
- Manifest id `booker-scotus-2005`.

### `rulings/booker-scotus-2005.docket.html`

- Supreme Court docket No. 04-104, United States v. Booker: the docket page as served.
- Source: <https://www.supremecourt.gov/search.aspx?filename=/docketfiles/04-104.htm>.
- Retrieved 2026-10-28 (2026-10-28T14:07:59Z). Downloaded as served (fetched with curl); stored unedited.
- sha256: `8e09158bed73280455fb265a8808e5714025710b5b7c27061a7562b9f4379a5b`.
- Manifest id `booker-scotus-2005` (`docket_file`).

### `rulings/smith-v-barry-scotus-1992.html`, `rulings/smith-v-barry-scotus-1992.json`, `rulings/smith-v-barry-scotus-1992.txt`

- Smith v. Barry, 502 U.S. 244 (U.S. 1992-01-14), No. 90-7477.
- Source: <https://static.case.law/us/502/html/0244-01.html>; metadata <https://static.case.law/us/502/cases/0244-01.json>.
- Retrieved 2026-10-28 (2026-10-28T14:04:31Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `smith-v-barry-scotus-1992.txt`: tags stripped from the .html; page labels rendered as [*N].
- `smith-v-barry-scotus-1992.html` sha256: `05805beb65772fb7e36c9a104d2324cf0d3994b474029a8d83f950401df5f79a`.
- `smith-v-barry-scotus-1992.json` sha256: `df34b88f3fcd0ab76970d6e9dc07e1991b886d36a33908792e579366cfc677fd`.
- `smith-v-barry-scotus-1992.txt` sha256: `02e8a58e1ba3ada08840ac56cb54a77ea5244835dadc07a12391260d3dd1b3ef`.
- Manifest id `smith-v-barry-scotus-1992`.

### `rulings/torres-scotus-1988.html`, `rulings/torres-scotus-1988.json`, `rulings/torres-scotus-1988.txt`

- Torres v. Oakland Scavenger Co., 487 U.S. 312 (U.S. 1988-06-24), No. 86-1845.
- Source: <https://static.case.law/us/487/html/0312-01.html>; metadata <https://static.case.law/us/487/cases/0312-01.json>.
- Retrieved 2026-10-28 (2026-10-28T14:04:32Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `torres-scotus-1988.txt`: tags stripped from the .html; page labels rendered as [*N].
- `torres-scotus-1988.html` sha256: `f00d5a85ddadf34a2db62d0c01077fd440588cc274cb8a95c84834c1f12e883f`.
- `torres-scotus-1988.json` sha256: `cb4174c18740cf513ce85ebff0d1f58a2bd695fc7ca1f351be0c128767de7ef3`.
- `torres-scotus-1988.txt` sha256: `c7746cce94f058affcc0ac7797c3facad7335d9abfc7be25316e28cd8cb036c5`.
- Manifest id `torres-scotus-1988`.

### `rulings/foman-scotus-1962.html`, `rulings/foman-scotus-1962.json`, `rulings/foman-scotus-1962.txt`

- Foman v. Davis, 371 U.S. 178 (U.S. 1962-12-03), No. 41.
- Source: <https://static.case.law/us/371/html/0178-01.html>; metadata <https://static.case.law/us/371/cases/0178-01.json>.
- Retrieved 2026-10-28 (2026-10-28T14:04:33Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `foman-scotus-1962.txt`: tags stripped from the .html; page labels rendered as [*N].
- `foman-scotus-1962.html` sha256: `f3a16d7000b0f6dc2579d5a97651e59732959b365ad3f24fdb75d333af3179fa`.
- `foman-scotus-1962.json` sha256: `8caa0f0bf23b3907b7b7b69a1d965588abb0561de7d1912970645f161b4ba999`.
- `foman-scotus-1962.txt` sha256: `044761ffa6e99778620eccdb990c47e6eff8017a3df32a988439cbdda6624c44`.
- Manifest id `foman-scotus-1962`.

### `rulings/gonzalez-v-crosby-scotus-2005.html`, `rulings/gonzalez-v-crosby-scotus-2005.json`, `rulings/gonzalez-v-crosby-scotus-2005.txt`

- Gonzalez v. Crosby, 545 U.S. 524 (U.S. 2005-06-23), No. 04-6432.
- Source: <https://static.case.law/us/545/html/0524-01.html>; metadata <https://static.case.law/us/545/cases/0524-01.json>.
- Retrieved 2026-10-28 (2026-10-28T14:04:34Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `gonzalez-v-crosby-scotus-2005.txt`: tags stripped from the .html; page labels rendered as [*N].
- `gonzalez-v-crosby-scotus-2005.html` sha256: `cb61f181b1a948a333b405d9667cc4d557093a32af5ee2ebec1e82fb403f5ae8`.
- `gonzalez-v-crosby-scotus-2005.json` sha256: `d9b80e2898af0a93f4a43772d4e62b964e58278a672a90cb9fd50bc694354842`.
- `gonzalez-v-crosby-scotus-2005.txt` sha256: `eb8e5f552e29f587ae22c7289a3295f2a5fee2c12de7a3b75d2828ef20fc3abd`.
- Manifest id `gonzalez-v-crosby-scotus-2005`.

### `rulings/anderson-bessemer-scotus-1985.html`, `rulings/anderson-bessemer-scotus-1985.json`, `rulings/anderson-bessemer-scotus-1985.txt`

- Anderson v. City of Bessemer City, 470 U.S. 564 (U.S. 1985-03-19), No. 83-1623.
- Source: <https://static.case.law/us/470/html/0564-01.html>; metadata <https://static.case.law/us/470/cases/0564-01.json>.
- Retrieved 2026-10-28 (2026-10-28T14:04:35Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `anderson-bessemer-scotus-1985.txt`: tags stripped from the .html; page labels rendered as [*N].
- `anderson-bessemer-scotus-1985.html` sha256: `e8f2abcf2f09d1a651d2b1222e768fa15b58556df46c39768cd705a0c64e33bf`.
- `anderson-bessemer-scotus-1985.json` sha256: `c19fe6fba3b3df2870823b2d8a07f9c2e4ee0b44e0aba1c1917612d3969fdbc4`.
- `anderson-bessemer-scotus-1985.txt` sha256: `883472f1d83aa7f3b9e2b7823fc539a27cebc379e4554715cc7a8177fd7bf3b1`.
- Manifest id `anderson-bessemer-scotus-1985`.

### `rulings/cooter-gell-scotus-1990.html`, `rulings/cooter-gell-scotus-1990.json`, `rulings/cooter-gell-scotus-1990.txt`

- Cooter & Gell v. Hartmarx Corp., 496 U.S. 384 (U.S. 1990-06-11), No. 89-275.
- Source: <https://static.case.law/us/496/html/0384-01.html>; metadata <https://static.case.law/us/496/cases/0384-01.json>.
- Retrieved 2026-10-28 (2026-10-28T14:04:36Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `cooter-gell-scotus-1990.txt`: tags stripped from the .html; page labels rendered as [*N].
- `cooter-gell-scotus-1990.html` sha256: `f4949577b8f9370295da86c0448e4cb3ed35b1e8443901740a4ba2f6ed908ad6`.
- `cooter-gell-scotus-1990.json` sha256: `70068a3e424b6d2d3f693b95e95fd62afb233441abe01d0dc0f057c37b05ef46`.
- `cooter-gell-scotus-1990.txt` sha256: `87e298b0517b80e7783511f872a3659d5ab0b0a46588c50e2992571e11afa60e`.
- Manifest id `cooter-gell-scotus-1990`.

### `rulings/cohen-scotus-1949.html`, `rulings/cohen-scotus-1949.json`, `rulings/cohen-scotus-1949.txt`

- Cohen v. Beneficial Industrial Loan Corp., 337 U.S. 541 (U.S. 1949-06-20), NO. 442.
- Source: <https://static.case.law/us/337/html/0541-01.html>; metadata <https://static.case.law/us/337/cases/0541-01.json>.
- Retrieved 2026-10-28 (2026-10-28T14:04:37Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `cohen-scotus-1949.txt`: tags stripped from the .html; page labels rendered as [*N].
- `cohen-scotus-1949.html` sha256: `0ca388a60f8754e00393236432e3bce8d25372fa8d8d3ae68196cf8108872d67`.
- `cohen-scotus-1949.json` sha256: `c6cdf95328353116526b1f5c94f59148890c40470a319e882837bb6c1b51867c`.
- `cohen-scotus-1949.txt` sha256: `bd1b7785bb68431cc44a373188fad78c30a0ac82cc4ec9024de0de57f5e6627e`.
- Manifest id `cohen-scotus-1949`.

### `rulings/ornelas-scotus-1996.html`, `rulings/ornelas-scotus-1996.json`, `rulings/ornelas-scotus-1996.txt`

- Ornelas v. United States, 517 U.S. 690 (U.S. 1996-05-28), No. 95-5257.
- Source: <https://static.case.law/us/517/html/0690-01.html>; metadata <https://static.case.law/us/517/cases/0690-01.json>.
- Retrieved 2026-10-28 (2026-10-28T14:04:38Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `ornelas-scotus-1996.txt`: tags stripped from the .html; page labels rendered as [*N].
- `ornelas-scotus-1996.html` sha256: `a110dff1a7b4a098f8447f39bad4f4ac51a249aa0cc6bc12a0d2669b7c7f48fb`.
- `ornelas-scotus-1996.json` sha256: `3ca9a50dbdc3029c3e17f1872009f503f3d4378ef076f313eb306eded872e898`.
- `ornelas-scotus-1996.txt` sha256: `701aaff90586c07f91b7f78484b83ba5a7e8c5bc25771b33cd22019b6c9a7b4c`.
- Manifest id `ornelas-scotus-1996`.

### `rulings/american-railway-express-scotus-1924.html`, `rulings/american-railway-express-scotus-1924.json`, `rulings/american-railway-express-scotus-1924.txt`

- United States & Interstate Commerce Commission v. American Railway Express Co., 265 U.S. 425 (U.S. 1924-06-02), Nos. 666-668.
- Source: <https://static.case.law/us/265/html/0425-01.html>; metadata <https://static.case.law/us/265/cases/0425-01.json>.
- Retrieved 2026-10-28 (2026-10-28T14:04:39Z). Downloaded as served (Caselaw Access Project static copy of the official reporter text (Harvard Law School Library); star page markers [*N] are reporter pages); `american-railway-express-scotus-1924.txt`: tags stripped from the .html; page labels rendered as [*N].
- `american-railway-express-scotus-1924.html` sha256: `8072f2934f2671ff6501da72c7a31e9099670a52636c64059726ff7e6c8efdde`.
- `american-railway-express-scotus-1924.json` sha256: `03b0565a7897447486df24a187c5d3f8753d5a40f8c6b700c83f052a29e54682`.
- `american-railway-express-scotus-1924.txt` sha256: `3721d50e3d23553ea2408f8b993480b58bf89f55a51df51677f54801b81464fb`.
- Manifest id `american-railway-express-scotus-1924`.
## CourtListener copies (`rulings/*.cl.pdf`)

The court-issued slip opinions of five rulings whose reporter text above
comes from the Caselaw Access Project, kept beside it (`rulings/README.md`).
`rulings/MANIFEST.json` has no entry for them: the storage URL each was
downloaded from was not recorded. The source given is the CourtListener
opinion page that the citation resolves to
(`https://www.courtlistener.com/c/<reporter>/<vol>/<page>/`, checked
2026-10-28), which serves the same PDF. Retrieved 2026-10-23: they were
downloaded with the lg-fq2.3 corpus and committed in 2b8a3307 that day; the
exact download time was not recorded.

### `rulings/justice-7cir-2012.cl.pdf`

- Justice v. Town of Cicero, 682 F.3d 662 (7th Cir. 2012), slip opinion PDF as issued by the court.
- Source: <https://www.courtlistener.com/opinion/801567/john-justice-v-town-of-cicero/>.
- Retrieved 2026-10-23. Downloaded as served from CourtListener storage; stored unedited.
- sha256: `2cb509fdac296486c2a112df950db888de7a96de6700216af0ba38154d4eb7b5`.

### `rulings/lorenzo-4cir-2015.cl.pdf`

- Lorenzo v. Prime Communications, L.P., 806 F.3d 777 (4th Cir. 2015), slip opinion PDF as issued by the court.
- Source: <https://www.courtlistener.com/opinion/3157627/rose-lorenzo-v-prime-communications-lp/>.
- Retrieved 2026-10-23. Downloaded as served from CourtListener storage; stored unedited.
- sha256: `3f970fca49938e5fe04499156a6682d1efe8e75debb9fcef770833e4013813e8`.

### `rulings/ramos-lopez-5cir-2016.cl.pdf`

- Ramos-Lopez v. Lynch, 823 F.3d 1024 (5th Cir. 2016), slip opinion PDF as issued by the court.
- Source: <https://www.courtlistener.com/opinion/3206584/sonia-ramos-lopez-v-loretta-lynch/>.
- Retrieved 2026-10-23. Downloaded as served from CourtListener storage; stored unedited.
- sha256: `b85b31fe802238a47dddfb7eb8b4ae0ae98d71ea32502cea14e195b77b9951b1`.

### `rulings/serna-5cir-2013.cl.pdf`

- Serna v. Law Office of Joseph Onwuteaka, P.C., 732 F.3d 440 (5th Cir. 2013), slip opinion PDF as issued by the court.
- Source: <https://www.courtlistener.com/opinion/1048497/rolando-serna-v-law-office-of-joseph-onwuteaka-e/>.
- Retrieved 2026-10-23. Downloaded as served from CourtListener storage; stored unedited.
- sha256: `13550dc67556a94c9dd4ae3b51b3ebd7d3c6c385cc5e6f28857374234b0f1888`.

### `rulings/ultimate-appliance-6cir-2010.cl.pdf`

- Ultimate Appliance CC v. Kirby Co., 601 F.3d 414 (6th Cir. 2010), slip opinion PDF as issued by the court.
- Source: <https://www.courtlistener.com/opinion/1098/ultimate-appliance-cc-v-kirby-co/>.
- Retrieved 2026-10-23. Downloaded as served from CourtListener storage; stored unedited.
- sha256: `0f2e0a7cfeec30aff0e048c6c30a03de6dd113ac86489e0e4dfdc25624f3ec3c`.

## Dockets: Supreme Court of the United States (`dockets/`)

The Court's docket pages of the five certiorari cases of the lg-wvlf.5 expect batch and of Booker's (lg-94rl) (`tests/expect/`), the record their petition dates come from. Each `.txt` is the page's case header and "Proceedings and Orders" table, one line per field or cell, extracted from the as-served HTML with a script (tags dropped, entities decoded, whitespace runs collapsed, the counsel table left out); no wording changed. The HTML as served is beside it under `dockets/original/`.

### `dockets/scotus-04-104.txt`

- Supreme Court docket No. 04-104, United States v. Booker.
- Source: <https://www.supremecourt.gov/search.aspx?filename=/docketfiles/04-104.htm>.
- Retrieved 2026-10-28 (2026-10-28T14:07:59Z, the `docket_retrieved_at` of `booker-scotus-2005` in `rulings/MANIFEST.json`); extracted 2026-10-29 by script as above from the HTML stored in lg-bdvf as `rulings/booker-scotus-2005.docket.html`, which `dockets/original/scotus-04-104.html` copies byte for byte.
- `dockets/scotus-04-104.txt` sha256: `c602955a92cb75ef40237674845bdecfdf28ce54335041164148baa04380f868`.
- Original: `dockets/original/scotus-04-104.html` as served (sha256 8e09158bed73280455fb265a8808e5714025710b5b7c27061a7562b9f4379a5b); extraction sha256 c602955a92cb75ef40237674845bdecfdf28ce54335041164148baa04380f868. Served at https://www.supremecourt.gov/search.aspx?filename=/docketfiles/04-104.htm; fetched with curl (lg-bdvf).

### `dockets/scotus-06-1164.txt`

- Supreme Court docket No. 06-1164, John R. Sand & Gravel Co. v. United States.
- Source: <https://www.supremecourt.gov/search.aspx?filename=/docketfiles/06-1164.htm>.
- Retrieved 2026-10-28; fetched with curl, extracted from the HTML by script as above.
- `dockets/scotus-06-1164.txt` sha256: `0e8d535e1f33af1f84f907baaba14c456640922968b61e5a56277ebceeb6b044`.
- Original: `dockets/original/scotus-06-1164.html` as served (sha256 6b665a50bf6240527edf7db462552e3328c61408ab6f0070da5912afae889a91); extraction sha256 0e8d535e1f33af1f84f907baaba14c456640922968b61e5a56277ebceeb6b044. Served at https://www.supremecourt.gov/search.aspx?filename=/docketfiles/06-1164.htm; fetched with curl, retrieved 2026-10-28.

### `dockets/scotus-07-219.txt`

- Supreme Court docket No. 07-219, Exxon Shipping Co. v. Baker.
- Source: <https://www.supremecourt.gov/search.aspx?filename=/docketfiles/07-219.htm>.
- Retrieved 2026-10-28; fetched with curl, extracted from the HTML by script as above.
- `dockets/scotus-07-219.txt` sha256: `49b22e89b0f4c1fee67e0cb5cf9d66c39aea028d13b0a6ed95819811adc9d633`.
- Original: `dockets/original/scotus-07-219.html` as served (sha256 0fb884a0cc0af46af76328f2ff2cfeb7d41ebf15435dcb639a5b0138bd07eee4); extraction sha256 49b22e89b0f4c1fee67e0cb5cf9d66c39aea028d13b0a6ed95819811adc9d633. Served at https://www.supremecourt.gov/search.aspx?filename=/docketfiles/07-219.htm; fetched with curl, retrieved 2026-10-28.

### `dockets/scotus-09-337.txt`

- Supreme Court docket No. 09-337, Krupski v. Costa Crociere S.p.A..
- Source: <https://www.supremecourt.gov/search.aspx?filename=/docketfiles/09-337.htm>.
- Retrieved 2026-10-28; fetched with curl, extracted from the HTML by script as above.
- `dockets/scotus-09-337.txt` sha256: `372597171e111dd553ece4058294254a2394af4089109276fd4a2e99f8bd2cae`.
- Original: `dockets/original/scotus-09-337.html` as served (sha256 4dd29867e5e0d3131e4ffc186f213e7cc9b94e1155c3be214530e57867ab444c); extraction sha256 372597171e111dd553ece4058294254a2394af4089109276fd4a2e99f8bd2cae. Served at https://www.supremecourt.gov/search.aspx?filename=/docketfiles/09-337.htm; fetched with curl, retrieved 2026-10-28.

### `dockets/scotus-09-846.txt`

- Supreme Court docket No. 09-846, United States v. Tohono O'odham Nation.
- Source: <https://www.supremecourt.gov/search.aspx?filename=/docketfiles/09-846.htm>.
- Retrieved 2026-10-28; fetched with curl, extracted from the HTML by script as above.
- `dockets/scotus-09-846.txt` sha256: `59f56051ad37510327a1ae0efe7b37de3dabe9b3b3bc0cd3e71a92a2e41f23c7`.
- Original: `dockets/original/scotus-09-846.html` as served (sha256 2d5326045ec1d3dd36796fcb764a3b2ad502cc231d54d524d2de0d027e01129b); extraction sha256 59f56051ad37510327a1ae0efe7b37de3dabe9b3b3bc0cd3e71a92a2e41f23c7. Served at https://www.supremecourt.gov/search.aspx?filename=/docketfiles/09-846.htm; fetched with curl, retrieved 2026-10-28.

### `dockets/scotus-13-352.txt`

- Supreme Court docket No. 13-352, B&B Hardware, Inc. v. Hargis Industries, Inc.
- Source: <https://www.supremecourt.gov/search.aspx?filename=/docketfiles/13-352.htm>.
- Retrieved 2026-10-28; fetched with curl, extracted from the HTML by script as above.
- `dockets/scotus-13-352.txt` sha256: `90e3b2814fab292013f952145bccca85640fc71a14e94ce72a509c8064ceb40a`.
- Original: `dockets/original/scotus-13-352.html` as served (sha256 09a2afdc9954b6151a0c1dc3b2cfe982c92e1cf0ff0c1ed885a16f9cdcf4ac5f); extraction sha256 90e3b2814fab292013f952145bccca85640fc71a14e94ce72a509c8064ceb40a. Served at https://www.supremecourt.gov/search.aspx?filename=/docketfiles/13-352.htm; fetched with curl, retrieved 2026-10-28.
