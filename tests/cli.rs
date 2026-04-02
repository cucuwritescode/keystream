use std::process::Command;

fn keystream() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_keystream"));
    // prevent interactive prompts in tests
    cmd.stdin(std::process::Stdio::null());
    cmd
}

#[test]
fn help_flag_shows_usage() {
    let output = keystream().arg("--help").output().expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("KEYSTREAM"), "should show header");
    assert!(stdout.contains("usage:"), "should show usage line");
    assert!(stdout.contains("start"), "should list start command");
    assert!(stdout.contains("stop"), "should list stop command");
    assert!(stdout.contains("status"), "should list status command");
    assert!(stdout.contains("run"), "should list run command");
}

#[test]
fn h_flag_shows_usage() {
    let output = keystream().arg("-h").output().expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("usage:"), "should show usage line");
}

#[test]
fn no_args_shows_usage() {
    let output = keystream().output().expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("usage:"), "should show usage line");
}

#[test]
fn unknown_command_exits_nonzero() {
    let output = keystream().arg("nonsense").output().expect("failed to run");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unknown command"),
        "should report unknown command on stderr"
    );
    assert!(!output.status.success(), "should exit non-zero");
}

#[test]
fn status_when_not_running() {
    let output = keystream().arg("status").output().expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("offline"),
        "status should show offline when no daemon is running"
    );
}

#[test]
fn stop_when_not_running() {
    let output = keystream().arg("stop").output().expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("not running"),
        "stop should indicate daemon is not running"
    );
}

#[test]
fn start_with_invalid_mode_exits_nonzero() {
    let output = keystream()
        .args(["start", "invalid_mode"])
        .output()
        .expect("failed to run");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unknown mode"),
        "should report unknown mode on stderr"
    );
    assert!(!output.status.success(), "should exit non-zero");
}

#[test]
fn version_flag_shows_version() {
    let output = keystream()
        .arg("--version")
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("keystream"), "should contain binary name");
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "should contain version number"
    );
}

#[test]
fn v_flag_shows_version() {
    let output = keystream().arg("-v").output().expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "should contain version number"
    );
}

#[test]
fn help_lists_modes() {
    let output = keystream().arg("--help").output().expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pentatonic"), "should list pentatonic mode");
    assert!(stdout.contains("lydian"), "should list lydian mode");
}
