import { optimizationWorkflow } from "../../../../packages/ui/src/optimizationWorkflow.js";
import { NvidiaWorkflowView } from "../components/OptimizationWorkflow";

export function NvidiaRoute() {
  return <NvidiaWorkflowView data={optimizationWorkflow.gaming.nvidia} />;
}
