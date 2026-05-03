import { desktopCommandCenterState } from "../adapters/desktopState";
import { PowerWorkflowView } from "../components/OptimizationWorkflow";

export function PowerRoute() {
  return <PowerWorkflowView data={desktopCommandCenterState.routes.power} />;
}
