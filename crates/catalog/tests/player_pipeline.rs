use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "sandbox-player-sheets-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&path).unwrap();
        for entry in
            std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sheets")).unwrap()
        {
            let entry = entry.unwrap();
            if entry.path().extension().is_some_and(|e| e == "csv") {
                std::fs::copy(entry.path(), path.join(entry.file_name())).unwrap();
            }
        }
        Self(path)
    }
    fn run(&self, action: &str) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_sandbox-catalog"));
        command.arg(action).arg(&self.0);
        if action == "generate" {
            command.arg(self.0.join("generated.rs"));
        }
        command.output().unwrap()
    }
    fn replace(&self, sheet: &str, field: &str, value: &str) {
        let path = self.0.join(sheet);
        let mut reader = csv::Reader::from_path(&path).unwrap();
        let headers = reader.headers().unwrap().clone();
        let column = headers.iter().position(|h| h == field).unwrap();
        let rows: Vec<_> = reader.records().map(|r| r.unwrap()).collect();
        drop(reader);
        let mut writer = csv::Writer::from_path(path).unwrap();
        writer.write_record(&headers).unwrap();
        for row in rows {
            writer
                .write_record(
                    row.iter()
                        .enumerate()
                        .map(|(i, v)| if i == column { value } else { v }),
                )
                .unwrap();
        }
        writer.flush().unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn cli_validates_and_generates_player_and_play_sheets() {
    let fixture = Fixture::new();
    for action in ["validate", "generate"] {
        let output = fixture.run(action);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let code = std::fs::read_to_string(fixture.0.join("generated.rs")).unwrap();
    assert!(code.contains("compiled_player_config"));
    assert!(code.contains("compiled_play_config"));
    let before = code;
    assert!(
        !fixture.run("generate").status.success(),
        "must not overwrite output"
    );
    assert_eq!(
        before,
        std::fs::read_to_string(fixture.0.join("generated.rs")).unwrap()
    );
}

#[test]
fn cli_rejects_invalid_player_and_play_sheets_before_writing_output() {
    for (sheet, field, value) in [
        ("source_player.csv", "walk_speed", "NaN"),
        ("source_player.csv", "model", "models/../escape.mdl"),
        ("source_player.csv", "radius", "0.6"),
        ("source_play.csv", "max_props", "0"),
        ("source_play.csv", "gravity", "NaN"),
    ] {
        let fixture = Fixture::new();
        fixture.replace(sheet, field, value);
        if field == "radius" {
            fixture.replace(sheet, "height", "0.5");
        }
        for action in ["validate", "generate"] {
            let output = fixture.run(action);
            assert!(
                !output.status.success(),
                "{action} accepted {sheet} {field}={value}"
            );
            assert!(String::from_utf8_lossy(&output.stderr).contains("ERROR:"));
        }
        assert!(!fixture.0.join("generated.rs").exists());
    }
}
