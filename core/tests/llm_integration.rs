use std::process::Command;

#[test]
#[ignore] // Nur manuell: cargo test -- --ignored
fn test_claude_categorization() {
    let output = Command::new(env!("CARGO_BIN_EXE_nexus-core"))
        .args(["set-key", "claude", "test"])
        .output();
    // Dieser Test ist ein Platzhalter.
    // Echte Integration erfordert einen gültigen API-Key:
    //
    //   nexus set-key claude sk-ant-...
    //   cargo test -- --ignored
    //
    // Dann wird hier gegen die echte API getestet.
    assert!(output.is_ok(), "Binary muss ausführbar sein");
}
