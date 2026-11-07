# Beta binary releases

Cut a release by pushing a version tag whose name starts with `v` followed by
a digit, such as `v0.4.0`. GitLab builds the `litigate` and `litigate-lsp`
binaries for `x86_64-unknown-linux-gnu` and `x86_64-unknown-linux-musl`, then
attaches both archives to the GitLab release for that tag.

The GNU artifact is built in Debian Bookworm. Its current bundled Z3 archive
requires glibc 2.39 or newer (the Z3 archive is built against that floor).
The musl job runs the pinned Rust Alpine image, where `litigate-smt` builds its
pinned Z3 5.1.0 source with the native musl C/C++ toolchain because Z3's
official Linux archive is glibc-linked. It runs both `litigate --version` and
a solve in that Alpine image, which has no glibc.

There is no branch release process. macOS binaries require a GitLab macOS
runner; the project does not currently configure one.
