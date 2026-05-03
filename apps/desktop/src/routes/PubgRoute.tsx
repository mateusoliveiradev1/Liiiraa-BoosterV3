import { desktopCommandCenterState } from "../adapters/desktopState";
import { PubgWorkflowView } from "../components/OptimizationWorkflow";

export function PubgRoute() {
  return <PubgWorkflowView data={desktopCommandCenterState.routes.pubg} />;
}
