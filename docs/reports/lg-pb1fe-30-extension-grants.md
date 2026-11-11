# Extension grants blocked by the current typed form (lg-pb1fe.30)

## Finding

`extend` does not encode the MPEP § 1.136(a) grant. The source grammar in
`docs/litigate/SPEC.md` §5.3 accepts a bounded grant with `max N.days` or a
date expression (`to B`), or an unbounded discretionary grant. The typed
HIR (`crates/litigate-sema/src/hir/temporal.rs`, `Extension`) and lowered
`Deadline.extensions[]` preserve those same day/date bounds and
discretionary grants; neither has a month-duration bound nor a representation
for fee payment as the condition that permits a party's as-of-right
extension. The ordinary period itself can be month-counted (`1.months`),
as `lit/mpep/deadlines.lit`'s `extension-month` demonstrates. That does not
make the distinct `extend` grant month-capable.

The three grants already typed in `lit/mpep/deadlines.lit` are *derived
periods* (`ssp-reply-extended-1/2/3`): they author alternate due dates
four, five, and six months from the Office Action, based on the three-month
SSP plus one, two, or three months. They are not `extend { ... }` grants.
The affected graph edges instead use the separate one-month
`extension-month` period for their inline deadlines. A typed declaration
there would wrongly imply that its one-month duration is the extension's
legal bound and miss the cumulative six-month ceiling. Thus none of the five
MPEP edges below can truthfully acquire an extension grant using the current
form.

The two inline deadlines have a separate shape problem. The PSR disclosure
deadline is an inline `deadline 35.days` on an edge to sentencing, with no
named deadline declaration to carry its § 32(b)(2) good-cause extension.
The ITC summary-determination motion similarly has an inline 60-day backward
window. Its rule permits good-cause exceptions, but the window is anchored
to the hearing schedule rather than a named event/deadline, and the inline
deadline form has no `extend` member. Moving it to an ordinary named deadline
would require inventing a schedule event and would lose § 210.18(a)'s
backward counting with a forward business-day roll. The links deadline is
named, but the links package has no authority level of its own from which
the lower-authority delegation form can be typed; the cited rules give
the 45-day trigger and do not state a typed grant/standard for the summary.

## Smallest language change for a later ruling

Add a month bound and fee condition to the bounded extension form, then carry
them as typed HIR fields and lowered JSON fields (with refinement checks for
the applicable fee and cumulative cap). The concrete shape needs a legal
decision on whether the cap is relative to the original prescribed due date
or a named statutory ceiling; do not approximate it as `N.days` or an
unbounded discretion.

For inline edge deadlines, the minimal form would allow a typed extension
grant on `deadline ... { ... }` and lower it into that edge's deadline
extensions and summary fields. The ITC case additionally needs a typed
schedule anchor/counting mode that represents a hearing-relative backward
window rolling forward on a nonbusiness day. For links grants, a separate
language decision is needed on the authority level and standard for a grant
authored in `lit/links`; simply borrowing `CaseOrder` from a forum would
claim an authority the links package does not have. These require a language
and authority-design ruling; this bead makes no syntax, HIR, or lowering
change.

## Lowered edge diff

No source declarations were changed, so the listed extension summary fields
remain as before. `null` means the lowered value is absent/null; the typed
extension list remains empty for all rows. The bead's three required package
groups are shown, with links added because its rehearing period is also in
the launch notes.

| Package | Edge | Field | Old | New |
|---|---|---|---|---|
| MPEP | `edge-applicant-response-window-extension-taken` | `deadline.extendable` | null | null |
| MPEP | `edge-applicant-response-window-extension-taken` | `deadline.extensionAuthority` | null | null |
| MPEP | `edge-extension-taken-amendment-filed` | `deadline.extendable` | null | null |
| MPEP | `edge-extension-taken-amendment-filed` | `deadline.extensionAuthority` | null | null |
| MPEP | `edge-extension-taken-extension-taken` | `deadline.extendable` | null | null |
| MPEP | `edge-extension-taken-extension-taken` | `deadline.extensionAuthority` | null | null |
| MPEP | `edge-notice-of-appeal-appeal-brief-filed` | `deadline.extendable` | null | null |
| MPEP | `edge-notice-of-appeal-appeal-brief-filed` | `deadline.extensionAuthority` | null | null |
| MPEP | `edge-notice-of-appeal-appeal-dismissed` | `deadline.extendable` | null | null |
| MPEP | `edge-notice-of-appeal-appeal-dismissed` | `deadline.extensionAuthority` | null | null |
| FRCrimP | `edge-presentence-report-sentencing-hearing` | `deadline.extendable` | null | null |
| FRCrimP | `edge-presentence-report-sentencing-hearing` | `deadline.extensionAuthority` | null | null |
| ITC | `edge-prehearing-summary-determination-motion-filed` | `deadline.extendable` | null | null |
| ITC | `edge-prehearing-summary-determination-motion-filed` | `deadline.extensionAuthority` | null | null |
| Links | each `rehearing-period-united-states-party` replacement edge | `deadline.extendable` | null | null |
| Links | each `rehearing-period-united-states-party` replacement edge | `deadline.extensionAuthority` | null | null |

The corresponding typed provisions are 37 C.F.R. §§ 1.136(a), 1.17(a),
41.37(e), 35 U.S.C. § 133; Fed. R. Crim. P. 32(b)(2); 19 C.F.R.
§§ 210.6(a), 210.18(a); and Fed. Cir. R. 40(f)(1), FRAP 40(d)(1). No
provision is being silently converted into a different extension mechanism.

DECIDED: leave all rows unchanged and document the missing typed forms -- the existing syntax cannot state their legal bounds, inline deadlines cannot carry grants, and inventing a month cap, schedule trigger, or links authority would misstate law.
