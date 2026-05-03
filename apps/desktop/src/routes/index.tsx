import type { ReactElement } from "react";
import { tOptimizer } from "../../../../packages/ui/src/localization";
import type { DesktopRouteId } from "../adapters/desktopState";
import { BenchmarksRoute } from "./BenchmarksRoute";
import { DashboardRoute } from "./DashboardRoute";
import { NvidiaRoute } from "./NvidiaRoute";
import { OptimizeRoute } from "./OptimizeRoute";
import { PowerRoute } from "./PowerRoute";
import { PubgRoute } from "./PubgRoute";
import { RollbackRoute } from "./RollbackRoute";
import { ScanRoute } from "./ScanRoute";
import { SettingsRoute } from "./settingsRoute";

export type DesktopRouteDefinition = {
  id: DesktopRouteId;
  label: string;
  element: ReactElement;
};

export const optimizationRoutes: DesktopRouteDefinition[] = [
  {
    id: "dashboard",
    label: tOptimizer("routes.dashboard.label"),
    element: <DashboardRoute />
  },
  {
    id: "scan",
    label: tOptimizer("routes.scan.label"),
    element: <ScanRoute />
  },
  {
    id: "optimize",
    label: tOptimizer("routes.optimize.label"),
    element: <OptimizeRoute />
  },
  {
    id: "power",
    label: tOptimizer("routes.power.label"),
    element: <PowerRoute />
  },
  {
    id: "nvidia",
    label: tOptimizer("routes.nvidia.label"),
    element: <NvidiaRoute />
  },
  {
    id: "pubg",
    label: tOptimizer("routes.pubg.label"),
    element: <PubgRoute />
  },
  {
    id: "benchmarks",
    label: tOptimizer("routes.benchmarks.label"),
    element: <BenchmarksRoute />
  },
  {
    id: "rollback",
    label: tOptimizer("routes.rollback.label"),
    element: <RollbackRoute />
  },
  {
    id: "settings",
    label: tOptimizer("routes.settings.label"),
    element: <SettingsRoute />
  }
];

export const defaultOptimizationRouteId: DesktopRouteId = "dashboard";

export { DashboardRoute } from "./DashboardRoute";
export { BenchmarksRoute } from "./BenchmarksRoute";
export { NvidiaRoute } from "./NvidiaRoute";
export { OptimizeRoute } from "./OptimizeRoute";
export { PowerRoute } from "./PowerRoute";
export { PubgRoute } from "./PubgRoute";
export { RollbackRoute } from "./RollbackRoute";
export { ScanRoute } from "./ScanRoute";
export { SettingsRoute } from "./settingsRoute";
