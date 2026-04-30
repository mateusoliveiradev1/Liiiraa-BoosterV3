import { optimizationWorkflow } from "../../../../packages/ui/src/optimizationWorkflow.js";
import { BenchmarkWorkflowView } from "../components/OptimizationWorkflow";
import { createDefaultPrivacyConsentState, evaluateDesktopPrivacyGate } from "../privacyConsent";

export function BenchmarksRoute() {
  const benchmarkSyncGate = evaluateDesktopPrivacyGate({
    consent: createDefaultPrivacyConsentState(),
    kind: "benchmark-sync"
  });

  return (
    <div style={{ display: "grid", gap: "1rem" }}>
      <BenchmarkWorkflowView data={optimizationWorkflow.gaming.benchmarks} />
      <section className="surface" data-tone={benchmarkSyncGate.tone}>
        <div className="section-heading">
          <div>
            <p className="eyebrow">Privacy gate</p>
            <h2>Benchmark cloud sync</h2>
          </div>
          <span className="pill pill--active">{benchmarkSyncGate.value}</span>
        </div>
        <p className="workflow-muted">{benchmarkSyncGate.message}</p>
      </section>
    </div>
  );
}
