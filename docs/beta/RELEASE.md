# Beta binary releases

Cut a release by pushing a version tag whose name starts with `v` followed by
a digit, such as `v0.4.0`. GitLab builds the `litigate` and `litigate-lsp`
binaries for `x86_64-unknown-linux-gnu` and `x86_64-unknown-linux-musl`, then
attaches both archives to the GitLab release for that tag.

There is no branch release process. macOS binaries require a GitLab macOS
runner; the project does not currently configure one.
