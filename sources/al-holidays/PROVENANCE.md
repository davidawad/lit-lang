# Legal-holiday statute provenance (al-holidays)

### `al-holidays/al-legal-holidays.txt` — Ala. Code § 1-3-8 (Alabama official state holidays enumerated)

- Source: https://alison.legislature.state.al.us/code-of-alabama?section=1-3-8 (official Alabama Legislature ALISON site, Code of Alabama, Title 1, Chapter 3). The site is a script application whose statute text arrives as GraphQL data, so the original here is the rendered statute pane (outerHTML of the one element holding the section) read from the live page, not a server-sent HTML document.
- Edition: the page shows 'Not in Session / 2026 Regular Session adjourned on April 9th, 2026' and the section history ends Act 2025-303, § 1; no pending amendment shown. Subsection (b): a holiday on Sunday is observed on the following day, on Saturday on the preceding day; Thanksgiving is 'as designated by the Governor'; Mardi Gras is a holiday in Baldwin and Mobile Counties.
- Retrieved 2026-10-29 on carbon (lg-br7q); BrowserOS neo (the page needs script; no forged headers or user agent): the section URL was loaded twice (the first load was used to find the statute element and a click test, the second to read it), plus the site's own GraphQL calls the page makes; there is no robots.txt (the path serves the application page, no directives); terms not separately fetched; the element's text was extracted with html2txt2.py (`## Ala. Code § 1-3-8`); the tab was closed.
- Extract sha256 445e763e4106d60c6d748b5a5f1cd1a148e6a600c52aeaf6d98246dd504c67e0.
- Original: `al-holidays/original/al-1-3-8-rendered.html` as served (sha256 a66da195d02f25c31d31c8bee686bf17215aa6684d205b69eae4f7c4b40dbd11). Served at https://alison.legislature.state.al.us/code-of-alabama?section=1-3-8.
