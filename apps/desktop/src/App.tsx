import { useState } from "react";
import logoMark from "./assets/logo-mark.svg";
import {
  commandMetrics,
  flowSteps,
  modeOptions,
  navigationItems,
  planBuckets,
  sessionEvents,
  statusStripItems,
  type CommandMetric,
  type FlowStep,
  type ModeOption,
  type NavigationItem,
  type PlanBucket,
  type SessionEvent,
  type StatusItem
} from "./commandCenter";

export function App() {
  const [activeView, setActiveView] = useState("dashboard");
  const [selectedMode, setSelectedMode] = useState<ModeOption["id"]>("safe");

  const activeLabel = navigationItems.find((item) => item.id === activeView)?.label ?? "Dashboard";

  return (
    <div className="app-shell">
      <aside className="sidebar" aria-label="Primary">
        <a className="brand" href="#dashboard" aria-label="Liiiraa Booster command center">
          <img src={logoMark} alt="" className="brand__mark" />
          <span className="brand__text">Liiiraa Booster</span>
        </a>
        <nav className="nav-list" aria-label="Desktop sections">
          {navigationItems.map((item) => (
            <NavButton key={item.id} item={item} active={item.id === activeView} onSelect={setActiveView} />
          ))}
        </nav>
      </aside>

      <div className="workspace">
        <StatusStrip items={statusStripItems} />
        <main className="command-center" id="dashboard">
          <header className="page-header">
            <div>
              <p className="eyebrow">Command center</p>
              <h1>{activeLabel}</h1>
            </div>
            <div className="header-actions" aria-label="Primary actions">
              <button className="button button--ghost" type="button">
                Export plan
              </button>
              <button className="button button--primary" type="button">
                Start scan
              </button>
            </div>
          </header>

          <section className="metric-grid" aria-label="System summary">
            {commandMetrics.map((metric) => (
              <MetricTile key={metric.label} metric={metric} />
            ))}
          </section>

          <section className="main-grid" aria-label="Command workspace">
            <div className="surface surface--wide">
              <div className="section-heading">
                <div>
                  <p className="eyebrow">Optimization state</p>
                  <h2>Safe plan queued</h2>
                </div>
                <span className="pill pill--active">Rollback ready</span>
              </div>
              <ModeSelector modes={modeOptions} selectedMode={selectedMode} onSelect={setSelectedMode} />
              <FlowRail steps={flowSteps} />
            </div>

            <div className="surface">
              <div className="section-heading">
                <div>
                  <p className="eyebrow">Apply queue</p>
                  <h2>Plan buckets</h2>
                </div>
              </div>
              <PlanBucketTable buckets={planBuckets} />
            </div>

            <div className="surface">
              <div className="section-heading">
                <div>
                  <p className="eyebrow">Session</p>
                  <h2>Rollback timeline</h2>
                </div>
              </div>
              <Timeline events={sessionEvents} />
            </div>

            <div className="surface surface--trust">
              <div className="trust-block">
                <span className="trust-block__label">Signed by Liiiraa</span>
                <strong>Stable channel verified</strong>
                <span>Catalog signature, updater signature, and local backup store are ready.</span>
              </div>
              <div className="action-bar">
                <button className="button button--primary" type="button">
                  Apply safe only
                </button>
                <button className="button button--ghost" type="button">
                  Review rollback
                </button>
              </div>
            </div>
          </section>
        </main>
      </div>
    </div>
  );
}

function NavButton({
  item,
  active,
  onSelect
}: {
  item: NavigationItem;
  active: boolean;
  onSelect: (id: string) => void;
}) {
  return (
    <button
      aria-current={active ? "page" : undefined}
      className="nav-button"
      data-active={active}
      onClick={() => onSelect(item.id)}
      title={item.group ? `${item.group}: ${item.label}` : item.label}
      type="button"
    >
      {item.group ? <span className="nav-button__group">{item.group}</span> : null}
      <span>{item.label}</span>
    </button>
  );
}

function StatusStrip({ items }: { items: StatusItem[] }) {
  return (
    <section className="status-strip" aria-label="Runtime status">
      {items.map((item) => (
        <span className="status-item" data-tone={item.tone} key={item.label}>
          <span>{item.label}</span>
          <strong>{item.value}</strong>
        </span>
      ))}
    </section>
  );
}

function MetricTile({ metric }: { metric: CommandMetric }) {
  return (
    <article className="metric-tile" data-tone={metric.tone}>
      <span>{metric.label}</span>
      <strong>{metric.value}</strong>
      <small>{metric.detail}</small>
    </article>
  );
}

function ModeSelector({
  modes,
  selectedMode,
  onSelect
}: {
  modes: ModeOption[];
  selectedMode: ModeOption["id"];
  onSelect: (mode: ModeOption["id"]) => void;
}) {
  return (
    <div className="mode-selector" aria-label="Optimization mode">
      {modes.map((mode) => (
        <button
          aria-pressed={selectedMode === mode.id}
          className="mode-option"
          data-tone={mode.tone}
          key={mode.id}
          onClick={() => onSelect(mode.id)}
          type="button"
        >
          <strong>{mode.label}</strong>
          <span>{mode.summary}</span>
        </button>
      ))}
    </div>
  );
}

function FlowRail({ steps }: { steps: FlowStep[] }) {
  return (
    <ol className="flow-rail" aria-label="Optimization flow">
      {steps.map((step) => (
        <li className="flow-step" data-state={step.state} key={step.label}>
          <span className="flow-step__marker" aria-hidden="true" />
          <div>
            <strong>{step.label}</strong>
            <span>{step.detail}</span>
          </div>
        </li>
      ))}
    </ol>
  );
}

function PlanBucketTable({ buckets }: { buckets: PlanBucket[] }) {
  return (
    <div className="bucket-table" role="table" aria-label="Plan buckets">
      <div className="bucket-row bucket-row--head" role="row">
        <span role="columnheader">Group</span>
        <span role="columnheader">Risk</span>
        <span role="columnheader">Rollback</span>
        <span role="columnheader">Reboot</span>
      </div>
      {buckets.map((bucket) => (
        <div className="bucket-row" data-tone={bucket.tone} role="row" key={bucket.label}>
          <span role="cell">
            <strong>{bucket.label}</strong>
            <small>{bucket.count} items</small>
          </span>
          <span role="cell">{bucket.risk}</span>
          <span role="cell">{bucket.rollback}</span>
          <span role="cell">{bucket.reboot}</span>
        </div>
      ))}
    </div>
  );
}

function Timeline({ events }: { events: SessionEvent[] }) {
  return (
    <ol className="timeline" aria-label="Recent session events">
      {events.map((event) => (
        <li className="timeline-item" data-tone={event.tone} key={`${event.time}-${event.label}`}>
          <time>{event.time}</time>
          <div>
            <strong>{event.label}</strong>
            <span>{event.detail}</span>
          </div>
        </li>
      ))}
    </ol>
  );
}
