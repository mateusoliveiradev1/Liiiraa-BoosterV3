import { useMemo, useState } from "react";
import logoMark from "./assets/logo-mark.svg";
import { navigationItems, statusStripItems, type NavigationItem, type StatusItem } from "./commandCenter";
import { defaultOptimizationRouteId, optimizationRoutes } from "./routes";

export function App() {
  const [activeView, setActiveView] = useState(defaultOptimizationRouteId);

  const activeRoute = useMemo(
    () =>
      optimizationRoutes.find((route) => route.id === activeView) ??
      optimizationRoutes.find((route) => route.id === defaultOptimizationRouteId) ??
      optimizationRoutes[0],
    [activeView]
  );

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
        <main className="command-center" id={activeRoute.id}>
          {activeRoute.element}
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
