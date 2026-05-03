import { tOptimizer } from "../../../../packages/ui/src/localization";
import { desktopCommandCenterState } from "../adapters/desktopState";
import { PlanActionBar, ScanWorkflowView } from "../components/OptimizationWorkflow";

const scanActions = [
  {
    id: "start-scan",
    label: tOptimizer("actions.startScan"),
    variant: "primary" as const
  },
  {
    id: "cancel-scan",
    label: tOptimizer("actions.cancelScan"),
    variant: "ghost" as const
  }
];

export function ScanRoute() {
  return (
    <ScanWorkflowView
      actions={<PlanActionBar actions={scanActions} />}
      data={desktopCommandCenterState.routes.scan}
    />
  );
}
