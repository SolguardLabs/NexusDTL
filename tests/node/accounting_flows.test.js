const assert = require("node:assert/strict");
const test = require("node:test");

const { assertConserved, runScenario } = require("../helpers/runScenario");

test("routed reparte importes entre trader, receiver, solver y vault", () => {
    const report = runScenario("routed");

    assertConserved(report);
    assert.equal(report.vault.reserve_balance, 88_010_000_000);
    assert.equal(report.balances.lp_reserve, 30_000_000_000);
    assert.equal(report.balances.trader_reserve, 16_000_000_000);
    assert.equal(report.balances.receiver_reserve, 1_984_030_000);
    assert.equal(report.balances.solver_reserve, 2_005_970_000);
    assert.equal(report.balances.borrower_reserve, 0);
});

test("credit reduce reserva de vault y acredita al borrower", () => {
    const report = runScenario("credit");

    assertConserved(report);
    assert.equal(report.transactions.length, 2);
    assert.equal(report.vault.reserve_balance, 88_500_000_000);
    assert.equal(report.balances.borrower_reserve, 1_500_000_000);
});

test("exit bloquea participaciones sin mover reserva líquida", () => {
    const report = runScenario("exit");

    assertConserved(report);
    assert.equal(report.transactions.length, 1);
    assert.equal(report.vault.reserve_balance, 90_000_000_000);
    assert.equal(report.vault.total_shares, 90_000_000_000);
    assert.equal(report.vault.locked_shares, 5_000_000_000);
});
