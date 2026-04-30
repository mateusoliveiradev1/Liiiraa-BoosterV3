import type { CSSProperties, ReactNode } from "react";
import type {
  SettingsTrustActionVariant,
  SettingsTrustChannel,
  SettingsTrustData,
  SettingsTrustGate,
  SettingsTrustMetric,
  SettingsTrustToggle,
  SettingsTrustTone
} from "../../../../../packages/ui/src/settingsTrust.js";

type ToneItem = {
  id: string;
  label: string;
  value?: string;
  state?: string;
  detail: string;
  tone: SettingsTrustTone;
};

const viewGridStyle: CSSProperties = {
  display: "grid",
  gap: "1rem"
};

const twoColumnStyle: CSSProperties = {
  display: "grid",
  gridTemplateColumns: "repeat(auto-fit, minmax(min(100%, 24rem), 1fr))",
  gap: "0.9rem",
  alignItems: "start"
};

const compactRowStyle: CSSProperties = {
  display: "grid",
  gap: "0.65rem",
  minWidth: 0
};

const trustMarkStyle: CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  minHeight: "2.3rem",
  padding: "0.45rem 0.7rem",
  color: "var(--success)",
  border: "1px solid rgba(58, 242, 143, 0.55)",
  borderRadius: "8px",
  fontWeight: 900
};

const statusRowStyle: CSSProperties = {
  display: "grid",
  gridTemplateColumns: "0.75rem minmax(8rem, 0.65fr) minmax(0, 1fr)",
  gap: "0.65rem",
  alignItems: "start"
};

const definitionGridStyle: CSSProperties = {
  display: "grid",
  gap: "0.65rem",
  margin: 0
};

const definitionRowStyle: CSSProperties = {
  display: "grid",
  gridTemplateColumns: "minmax(8.5rem, 0.55fr) minmax(0, 1fr)",
  gap: "0.75rem",
  margin: 0
};

const toggleRowStyle: CSSProperties = {
  display: "grid",
  gridTemplateColumns: "1.25rem minmax(0, 1fr) auto",
  gap: "0.65rem",
  alignItems: "start"
};

export function SettingsTrustSurfaces({ data }: { data: SettingsTrustData }) {
  return (
    <div style={viewGridStyle} aria-label="Settings privacy update and trust surfaces">
      <header className="page-header">
        <div>
          <p className="eyebrow">Settings</p>
          <h1>Privacy, updates, and trust</h1>
        </div>
        <div className="header-actions">
          <span style={trustMarkStyle}>{data.signature}</span>
          <button className="button button--primary" type="button">
            Check updates
          </button>
          <button className="button button--secondary" type="button">
            Export local data
          </button>
        </div>
      </header>

      <MetricGrid metrics={data.statusMetrics} />

      <div style={twoColumnStyle}>
        <Surface title="Privacy and telemetry" eyebrow="Consent state">
          <div style={compactRowStyle}>
            {data.privacyControls.map((control) => (
              <ToggleRow key={control.id} item={control} />
            ))}
          </div>
        </Surface>

        <Surface title="Update channel" eyebrow="Signed release lanes">
          <div style={compactRowStyle} role="radiogroup" aria-label="Update channel">
            {data.updateChannels.map((channel) => (
              <ChannelRow key={channel.id} channel={channel} />
            ))}
          </div>
        </Surface>

        <Surface title="Signing and update trust" eyebrow="Release integrity">
          <ToneList items={data.trustChain} />
        </Surface>

        <Surface title="Local data" eyebrow="Export and deletion controls">
          <div style={compactRowStyle}>
            {data.localDataActions.map((action) => (
              <LocalDataAction key={action.id} action={action} />
            ))}
          </div>
        </Surface>

        <Surface title="Advanced gates" eyebrow="Competitive and Lab boundaries">
          <div style={compactRowStyle}>
            {data.advancedGates.map((gate) => (
              <GateRow key={gate.id} gate={gate} />
            ))}
          </div>
        </Surface>

        <Surface title="Update metadata" eyebrow="Verified before install">
          <DefinitionGrid items={data.updateMetadata} />
        </Surface>
      </div>

      <Surface title="Audit trail" eyebrow="Recent trust checks">
        <div style={compactRowStyle}>
          {data.auditTrail.map((event) => (
            <StatusRow
              key={event.id}
              item={{
                id: event.id,
                label: event.time,
                value: event.label,
                detail: event.detail,
                tone: event.tone
              }}
            />
          ))}
        </div>
      </Surface>
    </div>
  );
}

function Surface({
  title,
  eyebrow,
  children
}: {
  title: string;
  eyebrow: string;
  children: ReactNode;
}) {
  return (
    <section className="surface">
      <div className="section-heading">
        <div>
          <p className="eyebrow">{eyebrow}</p>
          <h2>{title}</h2>
        </div>
      </div>
      {children}
    </section>
  );
}

function MetricGrid({ metrics }: { metrics: SettingsTrustMetric[] }) {
  return (
    <section className="metric-grid" aria-label="Settings trust metrics">
      {metrics.map((metric) => (
        <article className="metric-tile" data-tone={metric.tone} key={metric.id}>
          <span>{metric.label}</span>
          <strong>{metric.value}</strong>
          <small>{metric.detail}</small>
        </article>
      ))}
    </section>
  );
}

function ToneList({ items }: { items: ToneItem[] }) {
  return (
    <div style={compactRowStyle}>
      {items.map((item) => (
        <StatusRow item={item} key={item.id} />
      ))}
    </div>
  );
}

function StatusRow({ item }: { item: ToneItem }) {
  return (
    <div style={statusRowStyle} data-tone={item.tone}>
      <span style={toneMarkerStyle} aria-hidden="true" />
      <div>
        <strong>{item.label}</strong>
        <small className="workflow-muted">{item.value ?? item.state}</small>
      </div>
      <small className="workflow-muted">{item.detail}</small>
    </div>
  );
}

function ToggleRow({ item }: { item: SettingsTrustToggle }) {
  return (
    <label style={toggleRowStyle} data-tone={item.tone}>
      <input checked={item.enabled} readOnly type="checkbox" />
      <span>
        <strong>{item.label}</strong>
        <small className="workflow-muted">{item.detail}</small>
      </span>
      <span className="pill pill--active">{item.value}</span>
    </label>
  );
}

function ChannelRow({ channel }: { channel: SettingsTrustChannel }) {
  return (
    <label style={toggleRowStyle} data-tone={channel.tone}>
      <input checked={channel.selected} readOnly type="radio" name="settings-channel" />
      <span>
        <strong>{channel.label}</strong>
        <small className="workflow-muted">{channel.detail}</small>
      </span>
      <span className="pill pill--active">{channel.state}</span>
    </label>
  );
}

function GateRow({ gate }: { gate: SettingsTrustGate }) {
  return (
    <label style={toggleRowStyle} data-tone={gate.tone}>
      <input checked={gate.enabled} readOnly type="checkbox" />
      <span>
        <strong>{gate.label}</strong>
        <small className="workflow-muted">{gate.detail}</small>
      </span>
      <span className="pill pill--active">{gate.state}</span>
    </label>
  );
}

function LocalDataAction({
  action
}: {
  action: {
    id: string;
    label: string;
    detail: string;
    variant: SettingsTrustActionVariant;
  };
}) {
  const className =
    action.variant === "danger"
      ? "button button--ghost"
      : action.variant === "secondary"
        ? "button button--secondary"
        : "button button--ghost";
  const buttonStyle =
    action.variant === "danger"
      ? {
          color: "var(--danger)",
          borderColor: "rgba(255, 90, 103, 0.7)",
          background: "#331c23"
        }
      : undefined;

  return (
    <div style={definitionRowStyle}>
      <button className={className} style={buttonStyle} type="button">
        {action.label}
      </button>
      <small className="workflow-muted">{action.detail}</small>
    </div>
  );
}

function DefinitionGrid({ items }: { items: Array<[string, string]> }) {
  return (
    <dl style={definitionGridStyle}>
      {items.map(([label, value]) => (
        <div key={label} style={definitionRowStyle}>
          <dt>{label}</dt>
          <dd>{value}</dd>
        </div>
      ))}
    </dl>
  );
}

const toneMarkerStyle: CSSProperties = {
  width: "0.75rem",
  height: "0.75rem",
  marginTop: "0.25rem",
  background: "var(--tone, var(--neutral))",
  borderRadius: "50%"
};
