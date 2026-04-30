export type SettingsTrustTone = "active" | "danger" | "lab" | "neutral" | "success" | "warning";

export type SettingsTrustActionVariant = "danger" | "ghost" | "secondary";

export interface SettingsTrustMetric {
  id: string;
  label: string;
  value: string;
  detail: string;
  tone: SettingsTrustTone;
}

export interface SettingsTrustToggle {
  id: string;
  label: string;
  value: string;
  detail: string;
  enabled: boolean;
  tone: SettingsTrustTone;
}

export interface SettingsTrustChannel {
  id: "beta" | "dev" | "stable";
  label: string;
  state: string;
  detail: string;
  selected: boolean;
  tone: SettingsTrustTone;
}

export interface SettingsTrustChainItem {
  id: string;
  label: string;
  value: string;
  detail: string;
  tone: SettingsTrustTone;
}

export interface SettingsTrustLocalDataAction {
  id: string;
  label: string;
  detail: string;
  variant: SettingsTrustActionVariant;
}

export interface SettingsTrustGate {
  id: string;
  label: string;
  state: string;
  detail: string;
  enabled: boolean;
  tone: SettingsTrustTone;
}

export interface SettingsTrustAuditEvent {
  id: string;
  time: string;
  label: string;
  detail: string;
  tone: SettingsTrustTone;
}

export interface SettingsTrustData {
  signature: "Signed by Liiiraa";
  statusMetrics: SettingsTrustMetric[];
  privacyControls: SettingsTrustToggle[];
  updateChannels: SettingsTrustChannel[];
  trustChain: SettingsTrustChainItem[];
  localDataActions: SettingsTrustLocalDataAction[];
  advancedGates: SettingsTrustGate[];
  updateMetadata: Array<[string, string]>;
  auditTrail: SettingsTrustAuditEvent[];
}

export const settingsTrust: SettingsTrustData;

export function assertSettingsTrustSmoke(data?: SettingsTrustData): void;
export function renderSettingsTrustSmokeHtml(data?: SettingsTrustData): string;
