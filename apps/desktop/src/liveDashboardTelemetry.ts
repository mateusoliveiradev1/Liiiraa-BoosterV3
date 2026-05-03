import { invoke, isTauri } from "@tauri-apps/api/core";

export const LIVE_DASHBOARD_TELEMETRY_INTERVAL_MS = 1000;

const liveDashboardTelemetryPreferenceKey = "liiiraa.booster.liveDashboardTelemetry.enabled";
const liveDashboardTelemetryPreferenceEvent = "liiiraa:live-dashboard-telemetry-preference";

export type LiveResourceSnapshot = {
  commandId: string;
  requester: string;
  collectedAtUtc: string;
  source: string;
  cpu: {
    usagePercent: number | null;
    logicalProcessors: number | null;
    maxClockMhz: number | null;
    name: string | null;
  };
  memory: {
    totalBytes: number | null;
    freeBytes: number | null;
    usedBytes: number | null;
    usedPercent: number | null;
  };
  disk: {
    bytesPerSecond: number | null;
    totalBytes: number | null;
    usedPercent: number | null;
    primaryVolume: string | null;
    health: string | null;
  };
  network: {
    bytesPerSecond: number | null;
    totalBytes: number | null;
    linkSpeedBitsPerSecond: number | null;
    activeAdapters: number;
    adapterName: string | null;
  };
};

export type LiveDashboardTelemetryPreferenceListener = (enabled: boolean) => void;

export function canUseLiveDashboardTelemetry() {
  try {
    return isTauri();
  } catch {
    return typeof window !== "undefined" && Object.prototype.hasOwnProperty.call(window, "__TAURI_INTERNALS__");
  }
}

export function getLiveDashboardTelemetryPreference() {
  if (typeof window === "undefined") {
    return true;
  }

  try {
    return window.localStorage.getItem(liveDashboardTelemetryPreferenceKey) !== "false";
  } catch {
    return true;
  }
}

export function setLiveDashboardTelemetryPreference(enabled: boolean) {
  if (typeof window === "undefined") {
    return;
  }

  try {
    window.localStorage.setItem(liveDashboardTelemetryPreferenceKey, enabled ? "true" : "false");
  } catch {
    // The in-memory event still keeps open views in sync when storage is unavailable.
  }

  window.dispatchEvent(
    new CustomEvent(liveDashboardTelemetryPreferenceEvent, {
      detail: { enabled }
    })
  );
}

export function subscribeLiveDashboardTelemetryPreference(listener: LiveDashboardTelemetryPreferenceListener) {
  if (typeof window === "undefined") {
    return () => undefined;
  }

  const handlePreferenceChange = (event: Event) => {
    listener(Boolean((event as CustomEvent<{ enabled: boolean }>).detail.enabled));
  };
  const handleStorageChange = (event: StorageEvent) => {
    if (event.key === liveDashboardTelemetryPreferenceKey) {
      listener(event.newValue !== "false");
    }
  };

  window.addEventListener(liveDashboardTelemetryPreferenceEvent, handlePreferenceChange);
  window.addEventListener("storage", handleStorageChange);

  return () => {
    window.removeEventListener(liveDashboardTelemetryPreferenceEvent, handlePreferenceChange);
    window.removeEventListener("storage", handleStorageChange);
  };
}

export async function collectLiveDashboardTelemetry() {
  return invoke<LiveResourceSnapshot>("get_live_resource_snapshot", {
    payload: {
      requester: "main-window"
    }
  });
}
