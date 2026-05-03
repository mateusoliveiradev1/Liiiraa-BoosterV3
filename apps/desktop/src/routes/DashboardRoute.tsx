import { desktopCommandCenterState } from "../adapters/desktopState";
import { DashboardWorkflowView } from "../components/OptimizationWorkflow";

export function DashboardRoute() {
  return (
    <DashboardWorkflowView
      data={desktopCommandCenterState.routes.dashboard}
      optimizeData={desktopCommandCenterState.routes.optimize}
      rollbackData={desktopCommandCenterState.routes.rollback}
      scanData={desktopCommandCenterState.routes.scan}
    />
  );
}
