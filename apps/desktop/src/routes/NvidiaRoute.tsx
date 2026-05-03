import { desktopCommandCenterState } from "../adapters/desktopState";
import { NvidiaWorkflowView } from "../components/OptimizationWorkflow";

export function NvidiaRoute() {
  return <NvidiaWorkflowView data={desktopCommandCenterState.routes.nvidia} />;
}
