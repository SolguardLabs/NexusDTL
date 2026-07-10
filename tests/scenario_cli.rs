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

fn field<'a>(report: &'a Value, path: &[&str]) -> &'a Value {
    let mut cursor = report;
    for segment in path {
        cursor = cursor
            .get(segment)
            .unwrap_or_else(|| panic!("missing field: {}", path.join(".")));
    }
    cursor
}

fn number(report: &Value, path: &[&str]) -> u64 {
    field(report, path)
        .as_u64()
        .unwrap_or_else(|| panic!("field is not an unsigned number: {}", path.join(".")))
}

#[test]
fn snapshot_report_initializes_protocol_surface() {
    let report = run_scenario("snapshot");

    assert_eq!(field(&report, &["scenario"]), "snapshot");
    assert_eq!(number(&report, &["network_id"]), 72_900);
    assert_eq!(field(&report, &["conservation_ok"]), true);
    assert_eq!(number(&report, &["journal_entries"]), 17);
    assert_eq!(number(&report, &["surface", "operators"]), 4);
    assert_eq!(number(&report, &["surface", "oracle_publishers"]), 1);
    assert_eq!(number(&report, &["surface", "oracle_markets"]), 1);
    assert_eq!(number(&report, &["surface", "auction_lanes"]), 1);
    assert_eq!(number(&report, &["surface", "solver_bids"]), 1);
    assert_eq!(number(&report, &["surface", "netting_reports"]), 0);
    assert_eq!(number(&report, &["surface", "treasury_assets"]), 0);
}

#[test]
fn routed_report_applies_settlement_accounting() {
    let report = run_scenario("routed");

    assert_eq!(field(&report, &["scenario"]), "routed");
    assert_eq!(field(&report, &["conservation_ok"]), true);
    assert_eq!(
        number(&report, &["vault", "reserve_balance"]),
        88_010_000_000
    );
    assert_eq!(
        number(&report, &["balances", "trader_reserve"]),
        16_000_000_000
    );
    assert_eq!(
        number(&report, &["balances", "receiver_reserve"]),
        1_984_030_000
    );
    assert_eq!(
        number(&report, &["balances", "solver_reserve"]),
        2_005_970_000
    );
    assert_eq!(
        field(&report, &["transactions"])
            .as_array()
            .expect("transactions should be an array")
            .len(),
        1
    );
    assert_eq!(number(&report, &["surface", "netting_reports"]), 1);
    assert_eq!(number(&report, &["surface", "treasury_assets"]), 1);
}

#[test]
fn credit_report_tracks_facility_draw() {
    let report = run_scenario("credit");

    assert_eq!(field(&report, &["scenario"]), "credit");
    assert_eq!(field(&report, &["conservation_ok"]), true);
    assert_eq!(
        number(&report, &["vault", "reserve_balance"]),
        88_500_000_000
    );
    assert_eq!(
        number(&report, &["balances", "borrower_reserve"]),
        1_500_000_000
    );
    assert_eq!(
        field(&report, &["transactions"])
            .as_array()
            .expect("transactions should be an array")
            .len(),
        2
    );
}

#[test]
fn exit_report_locks_requested_shares() {
    let report = run_scenario("exit");

    assert_eq!(field(&report, &["scenario"]), "exit");
    assert_eq!(field(&report, &["conservation_ok"]), true);
    assert_eq!(
        number(&report, &["vault", "reserve_balance"]),
        90_000_000_000
    );
    assert_eq!(number(&report, &["vault", "total_shares"]), 90_000_000_000);
    assert_eq!(number(&report, &["vault", "locked_shares"]), 5_000_000_000);
    assert_eq!(
        field(&report, &["transactions"])
            .as_array()
            .expect("transactions should be an array")
            .len(),
        1
    );
}
