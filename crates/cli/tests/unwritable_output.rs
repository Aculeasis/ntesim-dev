use std::{
    fs::File,
    path::Path,
    process::{Command, Stdio},
};

#[test]
fn unwritable_output_returns_error_without_panicking() {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../presets/studio/s1.nte");
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    for valid_command in [true, false] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ntesim"));
        if valid_command {
            command.arg("inspect-script").arg(&script);
        }
        // Read-only handles deterministically reject writes, even if the child
        // finishes before its parent is scheduled again. No file is writable.
        let status = command
            .stdout(Stdio::from(File::open(&input).unwrap()))
            .stderr(Stdio::from(File::open(&input).unwrap()))
            .status()
            .expect("run ntesim CLI");
        assert_eq!(status.code(), Some(1), "valid command: {valid_command}");
    }
}
