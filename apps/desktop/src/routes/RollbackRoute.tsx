import { optimizationWorkflow } from "../../../../packages/ui/src/optimizationWorkflow.js";
import { RollbackWorkflowView } from "../components/OptimizationWorkflow";

export function RollbackRoute() {
  return <RollbackWorkflowView data={optimizationWorkflow.rollback} />;
}
