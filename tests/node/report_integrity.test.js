const assert = require("node:assert/strict");
const test = require("node:test");

const { assertDigest, runScenario } = require("../helpers/runScenario");

test("los assets se serializan con metadata estable", () => {
    const report = runScenario("snapshot");

    assert.equal(report.reserve_asset.symbol, "NXUSD");
    assert.equal(report.reserve_asset.decimals, 6);
    assert.equal(report.collateral_asset.symbol, "NXBOND");
    assert.equal(report.collateral_asset.decimals, 6);
    assertDigest(report.reserve_asset.id);
    assertDigest(report.collateral_asset.id);
    assertDigest(report.vault.vault_id);
});

test("cada transición principal cambia el digest de estado", () => {
    const snapshot = runScenario("snapshot");
    const routed = runScenario("routed");
    const credit = runScenario("credit");
    const exit = runScenario("exit");

    assert.notEqual(snapshot.state_digest, routed.state_digest);
    assert.notEqual(snapshot.state_digest, credit.state_digest);
    assert.notEqual(snapshot.state_digest, exit.state_digest);
    assert.notEqual(routed.state_digest, credit.state_digest);
});

test("los contadores de superficie se mantienen en escenarios no routed", () => {
    for (const name of ["snapshot", "credit", "exit"]) {
        const report = runScenario(name);

        assert.equal(report.surface.operators, 4);
        assert.equal(report.surface.oracle_publishers, 1);
        assert.equal(report.surface.oracle_markets, 1);
        assert.equal(report.surface.auction_lanes, 1);
        assert.equal(report.surface.solver_bids, 1);
    }
});
