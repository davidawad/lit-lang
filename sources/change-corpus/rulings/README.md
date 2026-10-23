# Timeliness rulings corpus (lg-fq2.3) — PARTIAL

Primary-source opinions deciding timeliness under FRCP 6 / FRAP 26 (and
related periods), for later validation: the deadline computed under the rules
in force on the relevant dates must match the court. Machine-readable record:
`MANIFEST.json` (case, court, docket, decided, citation, source_url,
retrieved_at UTC, sha256 of the stored PDF and of the derived `.txt`, rule and
version, trigger, service method, filing date, court-computed deadline,
holding, pin cites). Pin cites to slip opinions use PDF page numbers of the
stored file. Sources: CourtListener storage copies of court PDFs, and the
GPO U.S. Reports volume (Bowles).

## Status: 8 rulings of the 20 target; 0 confirmed circuit splits

| id | court | decided | events | holding |
| --- | --- | --- | --- | --- |
| rl-bowles-scotus-2007 | U.S. | 2007-06-14 | 2004 (pre-2009) | untimely |
| rl-ultimate-appliance-6cir-2010 | 6th Cir. | 2010-04-01 | 2008 (pre-2009 rules) | untimely |
| rl-serna-5cir-2013 | 5th Cir. | 2013-10-07 | 2010-11, 6(a)+6(d) | timely |
| rl-justice-7cir-2012 | 7th Cir. | 2012-06-05 | 2011, midnight rule | untimely |
| rl-vazquez-rivera-1cir-2014 | 1st Cir. | 2014-07-15 | 2010, admin deadline | untimely |
| rl-lorenzo-4cir-2015 | 4th Cir. | 2015-11-24 | 2014, 23(f) | untimely |
| rl-legras-9cir-2015 | 9th Cir. | 2015-05-28 | 2011, ERISA, 6(a) by analogy | timely |
| rl-ramos-lopez-5cir-2016 | 5th Cir. | 2016-05-24 | 2015, FRAP 26 | untimely |

Balance: before the Dec. 1, 2009 amendments, 2 (Bowles, Ultimate Appliance by
event dates); after, 6. Around the Dec. 1, 2016 removal of electronic-service
days: none decided after it; Serna, Lorenzo and Ramos-Lopez apply the old
"+3 days for electronic service" regime.

## Gaps and why

- Short of 20, no pre-2009 circuit rulings, no post-2016 rulings, no
  confirmed circuit split. Cause: the unauthenticated CourtListener REST API
  returned HTTP 429 after roughly twenty requests with `Retry-After` near
  67,000 s (about 18.6 hours), and its web UI and opinion pages sit behind a
  human-verification challenge for automated clients. Neither was bypassed.
  A CourtListener API token (authenticated v4 access) would remove the limit.
- `MANIFEST.json` `unconfirmed_leads` lists candidate split topics found by
  search (FRAP 4(a)(6) "notice" trigger; Rule 6(e) three days and Rule 59(b))
  whose opinion texts were NOT retrieved or read. They are leads, not splits.
- Lorenzo (4th Cir.) and Eastman (3d Cir.) agree that 6(d)/26(c) do not
  apply to periods running from entry of an order: no split there.
- Weak cases flagged in `notes`: Vazquez-Rivera (administrative deadline) and
  LeGras (ERISA contract period) are not pure FRCP 6 applications.
