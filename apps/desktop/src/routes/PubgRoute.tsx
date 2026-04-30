import { optimizationWorkflow } from "../../../../packages/ui/src/optimizationWorkflow.js";
import { PubgWorkflowView } from "../components/OptimizationWorkflow";

export function PubgRoute() {
  return <PubgWorkflowView data={optimizationWorkflow.gaming.pubg} />;
}
