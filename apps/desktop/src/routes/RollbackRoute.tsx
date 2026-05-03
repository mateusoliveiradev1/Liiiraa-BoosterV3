import { desktopCommandCenterState } from "../adapters/desktopState";
import { RollbackWorkflowView } from "../components/OptimizationWorkflow";

export function RollbackRoute() {
  return <RollbackWorkflowView data={desktopCommandCenterState.routes.rollback} />;
}
