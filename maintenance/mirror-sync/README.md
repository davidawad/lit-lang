# GitHub main mirrors GitLab main

GitLab main is the source of truth; GitHub main is a mirror of it
(lg-litigate-epic-tuq.29.1). Carbon keeps them equal by plain-pushing over
its existing SSH key: no GitHub token is stored in GitLab and no deploy
credential is reused.

`scripts/mirror-sync once` fetches both mains and decides:

| verdict | exit | meaning |
|---|---|---|
| `IN_SYNC <sha>` | 0 | both mains at the same commit |
| `MIRRORED <old>..<new>` | 0 | GitHub main was an ancestor of GitLab main and was fast-forwarded |
| `MIRROR_DIVERGED <gh> <gl>` | 3 | GitHub main is not an ancestor of GitLab main; nothing pushed — tell the PM |
| `MIRROR_FAILED <reason>` | 1 | a fetch or the push failed; the next pass retries |

It never forces: a push the remote refuses (GitHub main moved after the
fetch) is `MIRROR_FAILED`, and the next pass decides again from fresh refs.
Remote names default to `gitlab` and `github` (`MIRROR_FROM`, `MIRROR_TO`);
`MIRROR_REPO` picks the clone. Only `refs/mirror-sync/*` in that clone are
written.

- **Interim:** in a session, `scripts/mirror-sync loop` runs a pass every
  120 s and stops at the first `MIRROR_DIVERGED`.
- **Durable:** on carbon, the launchd agent
  [`com.litgraph.mirror-sync.plist`](com.litgraph.mirror-sync.plist) (setup
  in its header) or the equivalent cron line
  `*/2 * * * * MIRROR_REPO=/PATH/TO/litgraph /PATH/TO/litgraph/scripts/mirror-sync once >> /tmp/litgraph-mirror-sync.log 2>&1`.
  Watch the log for `MIRROR_DIVERGED`.

`just mirror-sync-test` (in `just ci`) runs `scripts/mirror-sync self-test`:
every verdict against throwaway bare repositories, including the
divergence refusal (GitHub main unchanged) and a GitHub main that moves
between fetch and push (refused, not forced).

Rejoin baseline: both mains at `c64cc2c0`.
