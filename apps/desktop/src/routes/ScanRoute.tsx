import { desktopCommandCenterState } from "../adapters/desktopState";
import { ScanWorkflowView } from "../components/OptimizationWorkflow";

export function ScanRoute() {
  return (
    <ScanWorkflowView
      data={desktopCommandCenterState.routes.scan}
      optimizeData={desktopCommandCenterState.routes.optimize}
    />
  );
}
