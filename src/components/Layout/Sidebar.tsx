import { SECTION_ITEMS, type Section } from "./types";

interface SidebarProps {
  activeSection: Section;
  onNavigate: (section: Section) => void;
  isCollapsed?: boolean;
}

export function Sidebar({ activeSection, onNavigate, isCollapsed = false }: SidebarProps) {
  const mainItems = SECTION_ITEMS.filter((item) => item.group === "main");
  const configItems = SECTION_ITEMS.filter((item) => item.group === "configure");

  return (
    <nav className={`sidebar ${isCollapsed ? "collapsed" : ""}`}>
      <div className="sidebar-brand">
        <img src="/icon.svg" alt="subtitledss" className="sidebar-brand-icon" />
        <div>
          <div className="sidebar-brand-text">subtitledss</div>
          <div className="sidebar-brand-sub">real-time subtitles</div>
        </div>
      </div>

      <div className="sidebar-section">
        <div className="sidebar-section-label">Main</div>
        {mainItems.map((item) => (
          <button
            key={item.id}
            onClick={() => onNavigate(item.id)}
            className={`sidebar-item ${activeSection === item.id ? "active" : ""}`}
          >
            <item.icon size={18} />
            <span className="sidebar-item-label">{item.label}</span>
          </button>
        ))}
      </div>

      <div className="sidebar-section">
        <div className="sidebar-section-label">Configure</div>
        {configItems.map((item) => (
          <button
            key={item.id}
            onClick={() => onNavigate(item.id)}
            className={`sidebar-item ${activeSection === item.id ? "active" : ""}`}
          >
            <item.icon size={18} />
            <span className="sidebar-item-label">{item.label}</span>
          </button>
        ))}
      </div>

      <div className="sidebar-footer">
        <div className="sidebar-version">v1.0.0 · MIT</div>
      </div>
    </nav>
  );
}
