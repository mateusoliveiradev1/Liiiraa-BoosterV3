import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  assertTelemetryRedactionCoverage,
  createPrivacySafeCrashReport,
  redactTelemetryString,
  redactTelemetryValue
} from "../src/index.js";

describe("telemetry redaction", () => {
  it("redacts common personal identifiers and secrets from strings", () => {
    const redacted = redactTelemetryString(
      "email=liiiraa@example.com path=C:\\Users\\Liiiraa\\AppData\\Local token=abc123 ip=192.168.0.25"
    );

    assert.equal(redacted.includes("liiiraa@example.com"), false);
    assert.equal(redacted.includes("C:\\Users\\Liiiraa"), false);
    assert.equal(redacted.includes("abc123"), false);
    assert.equal(redacted.includes("192.168.0.25"), false);
    assert.equal(redacted.includes("[redacted-email]"), true);
    assert.equal(redacted.includes("[redacted-local-path]"), true);
  });

  it("redacts nested telemetry values and denylisted keys", () => {
    const redacted = redactTelemetryValue({
      cookie: "sid=secret",
      databaseUrl: "postgres://user:pass@example.invalid/db",
      message: "open /home/liiiraa/.config/liiiraa/state.json",
      nested: {
        authorization: "Bearer raw-secret-value"
      }
    });
    const serialized = JSON.stringify(redacted);

    assert.equal(serialized.includes("sid=secret"), false);
    assert.equal(serialized.includes("postgres://"), false);
    assert.equal(serialized.includes("/home/liiiraa"), false);
    assert.equal(serialized.includes("raw-secret-value"), false);
    assert.equal(redacted.cookie, "[redacted]");
    assert.equal(redacted.databaseUrl, "[redacted]");
    assert.equal(redacted.nested.authorization, "[redacted]");
  });

  it("builds privacy-safe crash reports with redacted error context", () => {
    const report = createPrivacySafeCrashReport(
      {
        appVersion: "0.1.0",
        capturedAtUtc: "2026-05-02T12:00:00Z",
        channel: "stable",
        context: {
          localDataPath: "C:\\Users\\Liiiraa\\AppData\\Local\\Liiiraa",
          route: "/settings?email=liiiraa@example.com"
        },
        error: {
          message: "Could not load liiiraa@example.com",
          name: "CrashError",
          stack:
            "CrashError: Could not load liiiraa@example.com\n    at boot (C:\\Users\\Liiiraa\\AppData\\Local\\app.ts:1:1)"
        },
        id: "crash:test:001",
        severity: "fatal"
      },
      {
        capturedAtUtc: "2026-05-02T12:00:00Z"
      }
    );
    const serialized = JSON.stringify(report);

    assert.equal(report.kind, "crash-report");
    assert.equal(report.report.severity, "fatal");
    assert.equal(serialized.includes("liiiraa@example.com"), false);
    assert.equal(serialized.includes("C:\\Users\\Liiiraa"), false);
    assert.equal(serialized.includes("[redacted-email]"), true);
    assert.equal(serialized.includes("[redacted-local-path]"), true);
  });

  it("covers required telemetry redaction guardrails", () => {
    assert.equal(assertTelemetryRedactionCoverage(), true);
  });
});
