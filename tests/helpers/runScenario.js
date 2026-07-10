const assert = require("node:assert/strict");
const { execFileSync } = require("node:child_process");
const { resolve } = require("node:path");

const repoRoot = resolve(__dirname, "..", "..");

function runScenario(name) {
    const cargoArgs = ["run", "--quiet", "--", name];
    const stdout =
        process.platform === "win32"
            ? execFileSync("cmd.exe", ["/d", "/c", commandLine("cargo", cargoArgs)], {
                  cwd: repoRoot,
                  encoding: "utf8",
                  stdio: ["ignore", "pipe", "pipe"],
              })
            : execFileSync("cargo", cargoArgs, {
                  cwd: repoRoot,
                  encoding: "utf8",
                  stdio: ["ignore", "pipe", "pipe"],
              });
    return JSON.parse(stdout);
}

function commandLine(command, args) {
    return [command, ...args].map(quoteCmdArg).join(" ");
}

function quoteCmdArg(part) {
    const text = String(part);
    if (/^[A-Za-z0-9_.:=-]+$/.test(text)) {
        return text;
    }
    return `"${text.replaceAll('"', '""')}"`;
}

function assertDigest(value) {
    assert.equal(typeof value, "string");
    assert.match(value, /^[0-9a-f]{64}$/);
}

function assertConserved(report) {
    assert.equal(report.conservation_ok, true);
}

module.exports = {
    assertConserved,
    assertDigest,
    runScenario,
};
