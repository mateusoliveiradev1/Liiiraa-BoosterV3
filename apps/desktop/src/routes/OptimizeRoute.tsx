import { desktopCommandCenterState } from "../adapters/desktopState";
import { OptimizeWorkflowView } from "../components/OptimizationWorkflow";

export function OptimizeRoute() {
  return <OptimizeWorkflowView data={desktopCommandCenterState.routes.optimize} />;
}
