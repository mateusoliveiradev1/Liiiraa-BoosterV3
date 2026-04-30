import { optimizationWorkflow } from "../../../../packages/ui/src/optimizationWorkflow.js";
import { PowerWorkflowView } from "../components/OptimizationWorkflow";

export function PowerRoute() {
  return <PowerWorkflowView data={optimizationWorkflow.gaming.power} />;
}
