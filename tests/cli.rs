use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn unattended_execution_exits_without_prompting() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tz-convert"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);

    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("Unattended execution did not terminate");
        }
        thread::sleep(Duration::from_millis(10));
    }

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert!(!output.stdout.is_empty());
}
