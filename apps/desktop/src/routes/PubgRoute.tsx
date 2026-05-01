import { optimizationWorkflow } from "../../../../packages/ui/src/optimizationWorkflow.js";
import { PubgWorkflowView } from "../components/OptimizationWorkflow";

const pubgLaunchOptionCleanup: Array<{
  id: string;
  token: string;
  reason: string;
  recommendation: string;
  backup: string;
  tone: "active" | "danger" | "lab" | "neutral" | "success" | "warning";
}> = [
  {
    id: "use-all-cores",
    token: "-USEALLAVAILABLECORES",
    reason: "Windows already schedules PUBG across available cores.",
    recommendation: "Remove without adding a replacement flag.",
    backup: "Steam value captured before cleanup.",
    tone: "warning"
  },
  {
    id: "malloc-system",
    token: "-malloc=system",
    reason: "Allocator forcing is legacy and unsupported for current PUBG.",
    recommendation: "Remove without adding a replacement flag.",
    backup: "Steam value captured before cleanup.",
    tone: "warning"
  },
  {
    id: "priority-high",
    token: "-high",
    reason: "Priority forcing can starve system work and stays blocked by policy.",
    recommendation: "Remove and keep priority changes out of default plans.",
    backup: "Steam value captured before cleanup.",
    tone: "danger"
  },
  {
    id: "dx11-force",
    token: "-dx11",
    reason: "Renderer forcing should be benchmarked per machine.",
    recommendation: "Remove and use the DirectX benchmark flow instead.",
    backup: "Steam value captured before cleanup.",
    tone: "warning"
  }
];

const pubgWorkflowData = {
  ...optimizationWorkflow.gaming.pubg,
  launchOptions: pubgLaunchOptionCleanup
};

export function PubgRoute() {
  return <PubgWorkflowView data={pubgWorkflowData} />;
}
