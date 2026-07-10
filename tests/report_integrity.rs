use std::process::Command;

use serde_json::Value;

fn run_scenario(name: &str) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_nexus_dtl"))
        .arg(name)
        .output()
        .expect("scenario command should start");
    assert!(
        output.status.success(),
        "scenario command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("scenario output should be valid json")
}

fn assert_hex_digest(value: &Value) {
    let text = value
        .as_str()
        .expect("digest should be serialized as a string");
    assert_eq!(text.len(), 64, "digest should have 32 bytes encoded as hex");
    assert!(
        text.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "digest should use hexadecimal characters"
    );
}

#[test]
fn reports_include_stable_asset_metadata() {
    let report = run_scenario("snapshot");

    assert_eq!(report["reserve_asset"]["symbol"], "NXUSD");
    assert_eq!(report["reserve_asset"]["decimals"], 6);
    assert_eq!(report["collateral_asset"]["symbol"], "NXBOND");
    assert_eq!(report["collateral_asset"]["decimals"], 6);
    assert_hex_digest(&report["reserve_asset"]["id"]);
    assert_hex_digest(&report["collateral_asset"]["id"]);
    assert_hex_digest(&report["vault"]["vault_id"]);
    assert_hex_digest(&report["state_digest"]);
}

#[test]
fn unknown_scenario_uses_default_routed_flow() {
    let report = run_scenario("unknown-scenario");

    assert_eq!(report["scenario"], "routed");
    assert_eq!(report["conservation_ok"], true);
    assert_eq!(
        report["transactions"]
            .as_array()
            .expect("transactions should be an array")
            .len(),
        1
    );
}

#[test]
fn state_digests_change_across_state_transitions() {
    let snapshot = run_scenario("snapshot");
    let routed = run_scenario("routed");
    let credit = run_scenario("credit");
    let exit = run_scenario("exit");

    assert_ne!(snapshot["state_digest"], routed["state_digest"]);
    assert_ne!(snapshot["state_digest"], credit["state_digest"]);
    assert_ne!(snapshot["state_digest"], exit["state_digest"]);
    assert_ne!(routed["state_digest"], credit["state_digest"]);
}
