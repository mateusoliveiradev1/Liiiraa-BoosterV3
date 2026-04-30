import { optimizationWorkflow } from "../../../../packages/ui/src/optimizationWorkflow.js";
import { BenchmarkWorkflowView } from "../components/OptimizationWorkflow";

export function BenchmarksRoute() {
  return <BenchmarkWorkflowView data={optimizationWorkflow.gaming.benchmarks} />;
}
