//! Helpers for passing optional target tools to `CMake`.

pub(crate) fn cmake_tool_args(
    ar: Option<std::ffi::OsString>,
    ranlib: Option<std::ffi::OsString>,
) -> Vec<std::ffi::OsString> {
    [("-DCMAKE_AR=", ar), ("-DCMAKE_RANLIB=", ranlib)]
        .into_iter()
        .filter_map(|(prefix, value)| {
            value.map(|value| {
                let mut arg = std::ffi::OsString::from(prefix);
                arg.push(value);
                arg
            })
        })
        .collect()
}

pub(crate) fn stderr_tail(stderr: &[u8], max_lines: usize) -> String {
    let text = String::from_utf8_lossy(stderr);
    text.lines()
        .rev()
        .take(max_lines)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::{cmake_tool_args, stderr_tail};
    use std::ffi::OsString;

    #[test]
    fn cmake_tool_arguments_are_only_emitted_for_configured_tools() {
        assert!(cmake_tool_args(None, None).is_empty());
        assert_eq!(
            cmake_tool_args(Some("/usr/bin/musl-ar".into()), None),
            vec![OsString::from("-DCMAKE_AR=/usr/bin/musl-ar")]
        );
        assert_eq!(
            cmake_tool_args(None, Some("/usr/bin/musl-ranlib".into())),
            vec![OsString::from("-DCMAKE_RANLIB=/usr/bin/musl-ranlib")]
        );
        assert_eq!(
            cmake_tool_args(
                Some("/usr/bin/musl-ar".into()),
                Some("/usr/bin/musl-ranlib".into())
            ),
            vec![
                OsString::from("-DCMAKE_AR=/usr/bin/musl-ar"),
                OsString::from("-DCMAKE_RANLIB=/usr/bin/musl-ranlib")
            ]
        );
    }

    #[test]
    fn stderr_tail_keeps_only_the_requested_last_lines() {
        assert_eq!(stderr_tail(b"one\ntwo\nthree\nfour\n", 2), "three\nfour");
        assert_eq!(stderr_tail(b"one\ntwo\n", 0), "");
    }
}
