const assert = require("node:assert/strict");
const test = require("node:test");

const { assertConserved, assertDigest, runScenario } = require("../helpers/runScenario");

test("snapshot expone la superficie operativa inicial", () => {
    const report = runScenario("snapshot");

    assert.equal(report.scenario, "snapshot");
    assert.equal(report.network_id, 72_900);
    assertConserved(report);
    assertDigest(report.state_digest);
    assert.deepEqual(report.surface, {
        operators: 4,
        oracle_publishers: 1,
        oracle_markets: 1,
        auction_lanes: 1,
        solver_bids: 1,
        netting_reports: 0,
        treasury_assets: 0,
    });
});

test("routed ejecuta una liquidación con netting y tesorería", () => {
    const report = runScenario("routed");

    assert.equal(report.scenario, "routed");
    assertConserved(report);
    assert.equal(report.transactions.length, 1);
    assert.equal(report.surface.netting_reports, 1);
    assert.equal(report.surface.treasury_assets, 1);
    assertDigest(report.transactions[0]);
});

test("un nombre desconocido usa el flujo routed por defecto", () => {
    const report = runScenario("no-registrado");

    assert.equal(report.scenario, "routed");
    assertConserved(report);
    assert.equal(report.transactions.length, 1);
});
