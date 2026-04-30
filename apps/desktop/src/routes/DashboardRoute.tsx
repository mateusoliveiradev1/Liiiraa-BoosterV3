import { optimizationWorkflow } from "../../../../packages/ui/src/optimizationWorkflow.js";
import { DashboardWorkflowView, PlanActionBar } from "../components/OptimizationWorkflow";

export function DashboardRoute() {
  return (
    <DashboardWorkflowView
      actions={<PlanActionBar actions={optimizationWorkflow.optimize.actions.slice(0, 2)} />}
      data={optimizationWorkflow.dashboard}
    />
  );
}
