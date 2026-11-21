use std::process::{Command, Output, Stdio};
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
    let text = String::from_utf8(output.stdout).unwrap();
    let names: Vec<_> = text
        .lines()
        .filter(|line| line.starts_with('|'))
        .map(|line| line.split('|').nth(1).unwrap().trim())
        .collect();
    assert_eq!(names, ["Timezone", "Local", "UTC"]);
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tz-convert"))
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn success(args: &[&str]) -> String {
    let output = run(args);
    assert!(output.status.success(), "{:?}", output);
    assert!(output.stderr.is_empty());
    String::from_utf8(output.stdout).unwrap()
}

fn timezone_names() -> Vec<String> {
    let mut names: Vec<_> = jiff::tz::db()
        .available()
        .map(|name| name.to_string())
        .collect();
    names.sort_unstable();
    names
}

#[test]
fn lists_the_complete_timezone_database() {
    let output = success(&["--list-timezones"]);
    let names: Vec<_> = output.lines().collect();
    assert_eq!(names, timezone_names());
    for name in ["UTC", "Europe/London", "US/Eastern"] {
        assert!(names.contains(&name), "Missing {name}");
    }
}

#[test]
fn converts_to_every_timezone() {
    let output = success(&["--time", "2026-07-15 12:00", "--from", "UTC", "--all"]);
    let rows: Vec<_> = output
        .lines()
        .filter(|line| line.starts_with('|'))
        .map(|line| line.split('|').nth(1).unwrap().trim())
        .skip(1)
        .collect();
    let names = timezone_names();
    assert_eq!(rows.len(), names.len());
    for name in &names {
        assert!(rows.contains(&name.as_str()), "Missing {name}");
    }
}

#[test]
fn converts_multiple_destinations_with_current_rules() {
    let output = success(&[
        "--time",
        "2026-07-15 12:00",
        "--from",
        "UTC",
        "--to",
        "Africa/Cairo,Asia/Kathmandu",
        "--to",
        "America/Coyhaique",
    ]);
    assert!(output.contains("source: UTC"));
    assert!(output.contains("2026-07-15 15:00:00 EEST"));
    assert!(output.contains("2026-07-15 17:45:00 +0545"));
    assert!(output.contains("2026-07-15 09:00:00 -03"));
    assert!(output.contains("+05:45:00"));
}

#[test]
fn accepts_previously_unlisted_source_zones_and_rolls_over_dates() {
    let output = success(&[
        "--time",
        "2024-02-29 00:15",
        "--from",
        "Asia/Kathmandu",
        "--to",
        "UTC,Pacific/Chatham",
    ]);
    assert!(output.contains("2024-02-28 18:30:00 UTC"));
    assert!(output.contains("2024-02-29 08:15:00 +1345"));
}

#[test]
fn reports_invalid_dates_and_zones_as_failures() {
    for args in [
        vec!["--time", "tomorrow"],
        vec!["--time", "2026-02-30 12:00"],
        vec!["--time", "2100-07-15 12:00"],
        vec!["--from", "Europe/Unknown"],
        vec!["--to", "Asia/Unknown"],
        vec!["--all", "--to", "UTC"],
        vec!["--list-timezones", "--time", "2026-01-15 12:00"],
    ] {
        let output = run(&args);
        assert!(!output.status.success(), "{args:?}");
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn daylight_saving_errors_do_not_panic() {
    for (value, message) in [
        ("2026-03-29 01:30", "does not exist"),
        ("2026-10-25 01:30", "occurs twice"),
    ] {
        let output = run(&["--time", value, "--from", "Europe/London"]);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains(message));
        assert!(!error.contains("panicked"));
    }
}

#[test]
fn conversion_does_not_require_external_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_tz-convert"))
        .args([
            "--time",
            "2026-01-15 12:00",
            "--from",
            "UTC",
            "--to",
            "Europe/London",
        ])
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
}

#[test]
fn closed_output_pipe_exits_cleanly() {
    for argument in ["--list-timezones", "--all"] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_tz-convert"))
            .arg(argument)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        drop(child.stdout.take());
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn duplicate_destinations_are_only_displayed_once() {
    let output = success(&["--from", "UTC", "--to", "UTC,Asia/Tokyo,Asia/Tokyo"]);
    assert_eq!(output.matches("Asia/Tokyo").count(), 1);
    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with("| UTC "))
            .count(),
        1
    );
}

#[test]
fn aliases_are_displayed_separately() {
    let output = success(&["--from", "UTC", "--to", "US/Eastern,America/New_York"]);
    assert!(output.contains("| US/Eastern "));
    assert!(output.contains("| America/New_York "));
}

#[test]
fn timezone_names_are_case_insensitive() {
    let output = success(&[
        "--time",
        "2026-07-15 12:00",
        "--from",
        "europe/london",
        "--to",
        "ASIA/TOKYO",
    ]);
    assert!(output.contains("source: Europe/London"));
    assert!(output.contains("| Asia/Tokyo "));
    assert!(output.contains("2026-07-15 20:00:00 JST"));
}
