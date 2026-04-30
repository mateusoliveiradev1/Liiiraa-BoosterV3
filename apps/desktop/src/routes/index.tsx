import { DashboardRoute } from "./DashboardRoute";
import { OptimizeRoute } from "./OptimizeRoute";
import { RollbackRoute } from "./RollbackRoute";
import { ScanRoute } from "./ScanRoute";

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
    id: "rollback",
    label: "Rollback",
    element: <RollbackRoute />
  }
];

export const defaultOptimizationRouteId = "dashboard";

export { DashboardRoute } from "./DashboardRoute";
export { OptimizeRoute } from "./OptimizeRoute";
export { RollbackRoute } from "./RollbackRoute";
export { ScanRoute } from "./ScanRoute";
