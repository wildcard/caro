import { useEffect } from "react";
import { useAppStore } from "./store";
import {
  getConfig,
  getExecutionHistory,
  getAnalytics,
} from "./lib/tauri";
import { Terminal, History, Settings, BarChart3 } from "lucide-react";
import CommandTab from "./components/CommandTab";
import HistoryTab from "./components/HistoryTab";
import ConfigTab from "./components/ConfigTab";
import AnalyticsTab from "./components/AnalyticsTab";

function App() {
  const { currentTab, setCurrentTab, setConfig, setHistory, setAnalytics, historyFilter } =
    useAppStore();

  useEffect(() => {
    // Load initial data
    const loadData = async () => {
      try {
        const [config, history, analytics] = await Promise.all([
          getConfig(),
          getExecutionHistory(historyFilter),
          getAnalytics(),
        ]);
        setConfig(config);
        setHistory(history);
        setAnalytics(analytics);
      } catch (error) {
        console.error("Failed to load initial data:", error);
      }
    };

    loadData();
  }, []);

  const tabs = [
    { id: "command" as const, label: "Command", icon: Terminal },
    { id: "history" as const, label: "History", icon: History },
    { id: "config" as const, label: "Config", icon: Settings },
    { id: "analytics" as const, label: "Analytics", icon: BarChart3 },
  ];

  return (
    <div className="flex h-screen bg-background">
      {/* Sidebar */}
      <div className="w-64 bg-card border-r border-border flex flex-col">
        <div className="p-6 border-b border-border">
          <h1 className="text-2xl font-bold text-primary flex items-center gap-2">
            <Terminal className="w-6 h-6" />
            cmdai
          </h1>
          <p className="text-sm text-muted-foreground mt-1">
            Natural Language to Shell Commands
          </p>
        </div>

        <nav className="flex-1 p-4 space-y-2">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setCurrentTab(tab.id)}
              className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg transition-colors ${
                currentTab === tab.id
                  ? "bg-primary text-primary-foreground"
                  : "hover:bg-accent text-muted-foreground hover:text-foreground"
              }`}
            >
              <tab.icon className="w-5 h-5" />
              <span className="font-medium">{tab.label}</span>
            </button>
          ))}
        </nav>

        <div className="p-4 border-t border-border">
          <p className="text-xs text-muted-foreground text-center">
            cmdai GUI v0.1.0
          </p>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 overflow-hidden">
        {currentTab === "command" && <CommandTab />}
        {currentTab === "history" && <HistoryTab />}
        {currentTab === "config" && <ConfigTab />}
        {currentTab === "analytics" && <AnalyticsTab />}
      </div>
    </div>
  );
}

export default App;
