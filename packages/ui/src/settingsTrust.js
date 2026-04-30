/**
 * @typedef {"active" | "danger" | "lab" | "neutral" | "success" | "warning"} SettingsTrustTone
 * @typedef {"danger" | "ghost" | "secondary"} SettingsTrustActionVariant
 * @typedef {{ id: string; label: string; value: string; detail: string; tone: SettingsTrustTone }} SettingsTrustMetric
 * @typedef {{ id: string; label: string; value: string; detail: string; enabled: boolean; tone: SettingsTrustTone }} SettingsTrustToggle
 * @typedef {{ id: "beta" | "dev" | "stable"; label: string; state: string; detail: string; selected: boolean; tone: SettingsTrustTone }} SettingsTrustChannel
 * @typedef {{ id: string; label: string; value: string; detail: string; tone: SettingsTrustTone }} SettingsTrustChainItem
 * @typedef {{ id: string; label: string; detail: string; variant: SettingsTrustActionVariant }} SettingsTrustLocalDataAction
 * @typedef {{ id: string; label: string; state: string; detail: string; enabled: boolean; tone: SettingsTrustTone }} SettingsTrustGate
 * @typedef {{ id: string; time: string; label: string; detail: string; tone: SettingsTrustTone }} SettingsTrustAuditEvent
 * @typedef {{ id: string; label: string; detail: string; tone: SettingsTrustTone; value?: string; state?: string }} SettingsTrustStatusItem
 * @typedef {{ id: string; tone: SettingsTrustTone }} SettingsTrustToneItem
 * @typedef {{ signature: "Signed by Liiiraa"; statusMetrics: SettingsTrustMetric[]; privacyControls: SettingsTrustToggle[]; updateChannels: SettingsTrustChannel[]; trustChain: SettingsTrustChainItem[]; localDataActions: SettingsTrustLocalDataAction[]; advancedGates: SettingsTrustGate[]; updateMetadata: Array<[string, string]>; auditTrail: SettingsTrustAuditEvent[] }} SettingsTrustData
 */

/** @type {SettingsTrustData} */
export const settingsTrust = {
  signature: "Signed by Liiiraa",
  statusMetrics: [
    {
      id: "privacy",
      label: "Privacy",
      value: "Local first",
      detail: "Telemetry and crash reports are opt-in",
      tone: "success"
    },
    {
      id: "updates",
      label: "Updates",
      value: "Stable",
      detail: "Signed artifacts only",
      tone: "active"
    },
    {
      id: "catalog",
      label: "Catalog",
      value: "Verified",
      detail: "Last-known-good rollback cached",
      tone: "success"
    },
    {
      id: "lab",
      label: "Lab gates",
      value: "Locked",
      detail: "Advanced tweaks need explicit opt-in",
      tone: "warning"
    }
  ],
  privacyControls: [
    {
      id: "telemetry",
      label: "Performance telemetry",
      value: "Off",
      detail: "Benchmark and scan summaries stay on this PC until consent is enabled.",
      enabled: false,
      tone: "success"
    },
    {
      id: "crash-reports",
      label: "Crash reports",
      value: "Off",
      detail: "Reports exclude secrets, personal files, raw registry dumps, and cloud credentials.",
      enabled: false,
      tone: "success"
    },
    {
      id: "benchmark-sync",
      label: "Benchmark cloud sync",
      value: "Local only",
      detail: "Before and after captures remain available offline and can be exported manually.",
      enabled: false,
      tone: "neutral"
    }
  ],
  updateChannels: [
    {
      id: "stable",
      label: "Stable",
      state: "Selected",
      detail: "Receives signed releases after beta soak or explicit approval.",
      selected: true,
      tone: "success"
    },
    {
      id: "beta",
      label: "Beta",
      state: "Available",
      detail: "Receives updater, privileged-agent, and Lab-tweak changes before stable.",
      selected: false,
      tone: "warning"
    },
    {
      id: "dev",
      label: "Dev",
      state: "Internal",
      detail: "May break and is reserved for signed internal validation builds.",
      selected: false,
      tone: "lab"
    }
  ],
  trustChain: [
    {
      id: "identity",
      label: "Publisher identity",
      value: "Liiiraa Booster",
      detail: "Public screens identify the product as Signed by Liiiraa.",
      tone: "success"
    },
    {
      id: "app-artifact",
      label: "App artifacts",
      value: "Signature required",
      detail: "Stable distribution is blocked unless Windows and updater artifacts are signed.",
      tone: "success"
    },
    {
      id: "updater",
      label: "Updater metadata",
      value: "Signature verified",
      detail: "Version, platform, URL, release notes, channel, and signature are checked before install.",
      tone: "active"
    },
    {
      id: "catalog",
      label: "Tweak catalog",
      value: "Code-free",
      detail: "Remote catalogs cannot add arbitrary scripts or new privileged command IDs.",
      tone: "success"
    },
    {
      id: "private-keys",
      label: "Private keys",
      value: "Not bundled",
      detail: "Updater and release signing keys belong only in protected release environments.",
      tone: "warning"
    }
  ],
  localDataActions: [
    {
      id: "export-history",
      label: "Export local history",
      detail: "Snapshots, audit events, and benchmark metadata export without cloud sync.",
      variant: "secondary"
    },
    {
      id: "delete-telemetry-queue",
      label: "Delete pending sync queue",
      detail: "Clears unsent telemetry and benchmark sync payloads from this device.",
      variant: "danger"
    },
    {
      id: "open-data-folder",
      label: "Open local data folder",
      detail: "Inspect backups and last-known-good catalog cache.",
      variant: "ghost"
    }
  ],
  advancedGates: [
    {
      id: "competitive",
      label: "Competitive tradeoffs",
      state: "Review required",
      detail: "Security and reboot tradeoffs must be disclosed before apply.",
      enabled: false,
      tone: "warning"
    },
    {
      id: "lab",
      label: "Lab features",
      state: "Locked",
      detail: "Experimental changes require explicit opt-in, benchmark context, and rollback.",
      enabled: false,
      tone: "lab"
    },
    {
      id: "privileged-agent",
      label: "Privileged-agent changes",
      state: "Beta first",
      detail: "New elevated commands ship through beta before stable.",
      enabled: false,
      tone: "active"
    }
  ],
  updateMetadata: [
    ["Version", "0.0.0"],
    ["Channel", "stable"],
    ["Platform", "Windows x64"],
    ["Transport", "HTTPS only"],
    ["Signature", "Required"],
    ["Minimum previous version", "Set when needed"],
    ["Rollback", "Previous app plus last-known-good catalog"]
  ],
  auditTrail: [
    {
      id: "signature-check",
      time: "09:12",
      label: "Updater signature policy loaded",
      detail: "Unsigned metadata is rejected before install.",
      tone: "success"
    },
    {
      id: "catalog-cache",
      time: "09:14",
      label: "Catalog fallback ready",
      detail: "Last-known-good cache remains available offline.",
      tone: "active"
    },
    {
      id: "telemetry-state",
      time: "09:16",
      label: "Telemetry consent checked",
      detail: "Future uploads blocked while consent is off.",
      tone: "success"
    }
  ]
};

/** @param {unknown} value */
const escapeHtml = (value) =>
  String(value)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");

const requiredTones = new Set(["active", "danger", "lab", "neutral", "success", "warning"]);

/**
 * @param {string} label
 * @param {SettingsTrustToneItem[]} items
 */
const assertToneList = (label, items) => {
  for (const item of items) {
    if (!requiredTones.has(item.tone)) {
      throw new Error(`${label}.${item.id} has an unsupported tone: ${item.tone}`);
    }
  }
};

/** @param {SettingsTrustData} [data] */
export function assertSettingsTrustSmoke(data = settingsTrust) {
  if (data.signature !== "Signed by Liiiraa") {
    throw new Error('Settings trust surface must include "Signed by Liiiraa".');
  }

  const selectedChannels = data.updateChannels.filter((channel) => channel.selected);
  if (selectedChannels.length !== 1) {
    throw new Error("Exactly one update channel must be selected.");
  }

  for (const channel of ["dev", "beta", "stable"]) {
    if (!data.updateChannels.some((item) => item.id === channel)) {
      throw new Error(`Missing update channel: ${channel}`);
    }
  }

  if (!data.privacyControls.some((control) => control.id === "telemetry" && control.enabled === false)) {
    throw new Error("Telemetry must default to off on the trust settings surface.");
  }

  if (!data.trustChain.some((item) => item.id === "updater" && /signature/i.test(item.value))) {
    throw new Error("Updater trust chain must show signature verification.");
  }

  if (!data.trustChain.some((item) => item.id === "catalog" && /script/i.test(item.detail))) {
    throw new Error("Catalog trust chain must disclose that remote catalogs are not code execution.");
  }

  if (!data.advancedGates.some((gate) => gate.id === "lab" && gate.enabled === false)) {
    throw new Error("Lab gates must default to locked.");
  }

  for (const requiredMetadata of ["Channel", "Signature", "Transport"]) {
    if (!data.updateMetadata.some(([label]) => label === requiredMetadata)) {
      throw new Error(`Missing update metadata field: ${requiredMetadata}`);
    }
  }

  assertToneList("statusMetrics", data.statusMetrics);
  assertToneList("privacyControls", data.privacyControls);
  assertToneList("updateChannels", data.updateChannels);
  assertToneList("trustChain", data.trustChain);
  assertToneList("advancedGates", data.advancedGates);
  assertToneList("auditTrail", data.auditTrail);
}

/** @param {SettingsTrustMetric} metric */
const renderMetric = (metric) => `
  <article class="tile" data-tone="${escapeHtml(metric.tone)}">
    <span>${escapeHtml(metric.label)}</span>
    <strong>${escapeHtml(metric.value)}</strong>
    <small>${escapeHtml(metric.detail)}</small>
  </article>`;

/** @param {SettingsTrustStatusItem[]} items */
const renderStatusRows = (items) =>
  items
    .map(
      (item) => `
        <div class="row" data-tone="${escapeHtml(item.tone)}">
          <b>${escapeHtml(item.label)}</b>
          <span>${escapeHtml(item.value ?? item.state)}</span>
          <span>${escapeHtml(item.detail)}</span>
        </div>`
    )
    .join("");

export function renderSettingsTrustSmokeHtml(data = settingsTrust) {
  assertSettingsTrustSmoke(data);

  return `<!doctype html>
  <html lang="en">
    <head>
      <meta charset="utf-8" />
      <meta name="viewport" content="width=device-width, initial-scale=1" />
      <title>Liiiraa Booster Settings Trust Smoke</title>
      <style>
        :root {
          color-scheme: dark;
          font-family: Inter, "Segoe UI", system-ui, sans-serif;
          background: #0b0f14;
          color: #f5f8fb;
        }
        * { box-sizing: border-box; }
        body { margin: 0; background: #111820; }
        main { display: grid; gap: 18px; max-width: 1160px; margin: 0 auto; padding: 20px; }
        header { display: flex; justify-content: space-between; gap: 14px; align-items: center; }
        h1, h2, p { margin: 0; }
        h1 { font-size: 30px; letter-spacing: 0; }
        h2 { font-size: 17px; letter-spacing: 0; }
        .eyebrow { color: #7d8a99; font-size: 12px; font-weight: 800; text-transform: uppercase; }
        .grid { display: grid; grid-template-columns: repeat(4, minmax(140px, 1fr)); gap: 12px; }
        .views { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; }
        .panel, .tile {
          border: 1px solid #2a3541;
          border-radius: 8px;
          background: #151d26;
          box-shadow: 0 18px 52px rgba(0, 0, 0, 0.28);
        }
        .panel { display: grid; gap: 12px; padding: 16px; }
        .tile { display: grid; gap: 7px; min-height: 112px; padding: 14px; border-top: 3px solid var(--tone, #9aa8b8); }
        [data-tone="active"] { --tone: #27d7ff; }
        [data-tone="success"] { --tone: #3af28f; }
        [data-tone="warning"] { --tone: #ffbd5a; }
        [data-tone="danger"] { --tone: #ff5a67; }
        [data-tone="lab"] { --tone: #9b7cff; }
        [data-tone="neutral"] { --tone: #9aa8b8; }
        .tile span, .panel span, .panel small, .panel p { color: #9aa8b8; }
        strong, b { color: #f5f8fb; }
        .tile strong { font-family: Consolas, monospace; font-size: 22px; }
        .row {
          display: grid;
          grid-template-columns: minmax(150px, 0.75fr) minmax(110px, 0.45fr) minmax(0, 1fr);
          gap: 10px;
          padding-top: 10px;
          border-top: 1px solid #2a3541;
        }
        .row:first-of-type { border-top: 0; padding-top: 0; }
        .row b { color: var(--tone, #f5f8fb); }
        .controls { display: flex; flex-wrap: wrap; gap: 8px; }
        button {
          min-height: 36px;
          border: 1px solid #344252;
          border-radius: 8px;
          background: #202b37;
          color: #f5f8fb;
          font: inherit;
          font-weight: 800;
        }
        button.primary { background: #27d7ff; color: #071015; border-color: #27d7ff; }
        button.danger { color: #ff5a67; border-color: #ff5a67; background: #331c23; }
        .signature {
          display: inline-flex;
          align-items: center;
          min-height: 34px;
          padding: 6px 10px;
          border: 1px solid rgba(58, 242, 143, 0.55);
          border-radius: 8px;
          color: #3af28f;
          font-weight: 900;
        }
        @media (max-width: 860px) {
          header { align-items: flex-start; flex-direction: column; }
          .grid, .views { grid-template-columns: 1fr; }
          .row { grid-template-columns: 1fr; }
        }
      </style>
    </head>
    <body>
      <main>
        <header>
          <div>
            <p class="eyebrow">Settings</p>
            <h1>Privacy, updates, and trust</h1>
          </div>
          <div class="signature">${escapeHtml(data.signature)}</div>
        </header>
        <section class="grid" aria-label="Settings trust metrics">
          ${data.statusMetrics.map(renderMetric).join("")}
        </section>
        <section class="views">
          <section class="panel">
            <h2>Privacy and telemetry</h2>
            ${renderStatusRows(data.privacyControls)}
          </section>
          <section class="panel">
            <h2>Update channel</h2>
            ${renderStatusRows(data.updateChannels)}
          </section>
          <section class="panel">
            <h2>Signing and update trust</h2>
            ${renderStatusRows(data.trustChain)}
          </section>
          <section class="panel">
            <h2>Local data</h2>
            ${data.localDataActions
              .map((action) => `<div class="row"><b>${escapeHtml(action.label)}</b><button class="${escapeHtml(action.variant)}">${escapeHtml(action.label)}</button><span>${escapeHtml(action.detail)}</span></div>`)
              .join("")}
          </section>
          <section class="panel">
            <h2>Advanced gates</h2>
            ${renderStatusRows(data.advancedGates)}
          </section>
          <section class="panel">
            <h2>Update metadata</h2>
            ${data.updateMetadata
              .map(([label, value]) => `<div class="row"><b>${escapeHtml(label)}</b><span>${escapeHtml(value)}</span><span>Verified before install</span></div>`)
              .join("")}
          </section>
          <section class="panel">
            <h2>Audit trail</h2>
            ${data.auditTrail
              .map((event) => `<div class="row" data-tone="${escapeHtml(event.tone)}"><b>${escapeHtml(event.time)}</b><span>${escapeHtml(event.label)}</span><span>${escapeHtml(event.detail)}</span></div>`)
              .join("")}
          </section>
        </section>
      </main>
    </body>
  </html>`;
}
