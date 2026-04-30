import { optimizationWorkflow } from "../../../../packages/ui/src/optimizationWorkflow.js";
import { OptimizeWorkflowView } from "../components/OptimizationWorkflow";

export function OptimizeRoute() {
  return <OptimizeWorkflowView data={optimizationWorkflow.optimize} />;
}
