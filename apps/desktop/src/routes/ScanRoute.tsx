import { optimizationWorkflow } from "../../../../packages/ui/src/optimizationWorkflow.js";
import { PlanActionBar, ScanWorkflowView } from "../components/OptimizationWorkflow";

const scanActions = [
  {
    id: "start-scan",
    label: "Start scan",
    variant: "primary" as const
  },
  {
    id: "cancel-scan",
    label: "Cancel scan",
    variant: "ghost" as const
  }
];

export function ScanRoute() {
  return <ScanWorkflowView actions={<PlanActionBar actions={scanActions} />} data={optimizationWorkflow.scan} />;
}
