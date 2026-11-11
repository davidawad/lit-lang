//! The opt-in phase diagnostics must not change the check response bytes.

use std::process::Command;

#[test]
fn phase_timings_leave_check_stdout_unchanged() {
    let binary = env!("CARGO_BIN_EXE_litigate");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let run = |timings: bool| {
        let mut command = Command::new(binary);
        command
            .current_dir(&root)
            .args(["--format", "json", "check", "lit/core"]);
        if timings {
            command.env("LITIGATE_PHASE_TIMINGS", "1");
        } else {
            command.env_remove("LITIGATE_PHASE_TIMINGS");
        }
        command.output().expect("run litigate check")
    };
    let plain = run(false);
    let timed = run(true);
    assert_eq!(plain.status.code(), timed.status.code());
    let normalize = |stdout: &[u8]| {
        let mut value: serde_json::Value = serde_json::from_slice(stdout).unwrap();
        value.as_object_mut().unwrap().remove("elapsed_ms");
        value
    };
    assert_eq!(normalize(&plain.stdout), normalize(&timed.stdout));
    assert!(timed.stderr.windows(14).any(|w| w == b"LITIGATE_PHASE"));
}
