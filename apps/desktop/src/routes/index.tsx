import { BenchmarksRoute } from "./BenchmarksRoute";
import { DashboardRoute } from "./DashboardRoute";
import { NvidiaRoute } from "./NvidiaRoute";
import { OptimizeRoute } from "./OptimizeRoute";
import { PowerRoute } from "./PowerRoute";
import { PubgRoute } from "./PubgRoute";
import { RollbackRoute } from "./RollbackRoute";
import { ScanRoute } from "./ScanRoute";
import { SettingsRoute } from "./settingsRoute";

export const optimizationRoutes = [
  {
    id: "dashboard",
    label: "Dashboard",
    element: <DashboardRoute />
  },
  {
    id: "scan",
    label: "Scan",
    element: <ScanRoute />
  },
  {
    id: "optimize",
    label: "Optimize",
    element: <OptimizeRoute />
  },
  {
    id: "power",
    label: "Power",
    element: <PowerRoute />
  },
  {
    id: "nvidia",
    label: "NVIDIA",
    element: <NvidiaRoute />
  },
  {
    id: "pubg",
    label: "PUBG",
    element: <PubgRoute />
  },
  {
    id: "benchmarks",
    label: "Benchmarks",
    element: <BenchmarksRoute />
  },
  {
    id: "rollback",
    label: "Rollback",
    element: <RollbackRoute />
  },
  {
    id: "settings",
    label: "Settings",
    element: <SettingsRoute />
  }
];

export const defaultOptimizationRouteId = "dashboard";

export { DashboardRoute } from "./DashboardRoute";
export { BenchmarksRoute } from "./BenchmarksRoute";
export { NvidiaRoute } from "./NvidiaRoute";
export { OptimizeRoute } from "./OptimizeRoute";
export { PowerRoute } from "./PowerRoute";
export { PubgRoute } from "./PubgRoute";
export { RollbackRoute } from "./RollbackRoute";
export { ScanRoute } from "./ScanRoute";
export { SettingsRoute } from "./settingsRoute";
