import { useEffect, useState } from "react";
import { useAppStore } from "../store";
import { getAnalytics, exportHistory } from "../lib/tauri";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import {
  BarChart3,
  TrendingUp,
  Clock,
  Terminal as TerminalIcon,
  Download,
  CheckCircle2,
  XCircle,
  AlertCircle,
} from "lucide-react";

export default function AnalyticsTab() {
  const { analytics, setAnalytics } = useAppStore();
  const [exporting, setExporting] = useState(false);

  useEffect(() => {
    loadAnalytics();
  }, []);

  const loadAnalytics = async () => {
    try {
      const data = await getAnalytics();
      setAnalytics(data);
    } catch (error) {
      console.error("Failed to load analytics:", error);
    }
  };

  const handleExport = async (format: "json" | "csv") => {
    setExporting(true);
    try {
      const data = await exportHistory(format);

      const filePath = await save({
        defaultPath: `cmdai-history.${format}`,
        filters: [{
          name: format.toUpperCase(),
          extensions: [format]
        }]
      });

      if (filePath) {
        await writeTextFile(filePath, data);
        alert("History exported successfully!");
      }
    } catch (error) {
      console.error("Failed to export history:", error);
      alert(`Error: ${error}`);
    } finally {
      setExporting(false);
    }
  };

  if (!analytics) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  const successRate = analytics.total_executions > 0
    ? (analytics.successful_executions / analytics.total_executions) * 100
    : 0;

  const blockRate = analytics.total_executions > 0
    ? (analytics.blocked_executions / analytics.total_executions) * 100
    : 0;

  return (
    <div className="h-full flex flex-col">
      <div className="p-6 border-b border-border flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Analytics</h2>
          <p className="text-sm text-muted-foreground mt-1">
            Usage statistics and insights
          </p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => handleExport("json")}
            disabled={exporting}
            className="px-4 py-2 bg-secondary text-secondary-foreground rounded-lg hover:bg-secondary/90 disabled:opacity-50 flex items-center gap-2"
          >
            <Download className="w-4 h-4" />
            Export JSON
          </button>
          <button
            onClick={() => handleExport("csv")}
            disabled={exporting}
            className="px-4 py-2 bg-secondary text-secondary-foreground rounded-lg hover:bg-secondary/90 disabled:opacity-50 flex items-center gap-2"
          >
            <Download className="w-4 h-4" />
            Export CSV
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-8">
          {/* Total Executions */}
          <div className="bg-card border border-border rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-muted-foreground">
                Total Executions
              </span>
              <BarChart3 className="w-5 h-5 text-primary" />
            </div>
            <p className="text-3xl font-bold">{analytics.total_executions}</p>
          </div>

          {/* Successful */}
          <div className="bg-card border border-border rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-muted-foreground">
                Successful
              </span>
              <CheckCircle2 className="w-5 h-5 text-green-600" />
            </div>
            <p className="text-3xl font-bold">{analytics.successful_executions}</p>
            <p className="text-sm text-muted-foreground mt-1">
              {successRate.toFixed(1)}% success rate
            </p>
          </div>

          {/* Blocked */}
          <div className="bg-card border border-border rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-muted-foreground">
                Blocked
              </span>
              <XCircle className="w-5 h-5 text-red-600" />
            </div>
            <p className="text-3xl font-bold">{analytics.blocked_executions}</p>
            <p className="text-sm text-muted-foreground mt-1">
              {blockRate.toFixed(1)}% blocked
            </p>
          </div>

          {/* Avg Generation Time */}
          <div className="bg-card border border-border rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-muted-foreground">
                Avg Generation Time
              </span>
              <Clock className="w-5 h-5 text-primary" />
            </div>
            <p className="text-3xl font-bold">
              {analytics.average_generation_time_ms.toFixed(0)}
              <span className="text-lg text-muted-foreground ml-1">ms</span>
            </p>
          </div>

          {/* Most Used Shell */}
          <div className="bg-card border border-border rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-muted-foreground">
                Most Used Shell
              </span>
              <TerminalIcon className="w-5 h-5 text-primary" />
            </div>
            <p className="text-3xl font-bold capitalize">
              {analytics.most_used_shell || "N/A"}
            </p>
          </div>

          {/* Performance Trend */}
          <div className="bg-card border border-border rounded-lg p-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-muted-foreground">
                Performance
              </span>
              <TrendingUp className="w-5 h-5 text-green-600" />
            </div>
            <p className="text-3xl font-bold text-green-600">Good</p>
            <p className="text-sm text-muted-foreground mt-1">
              Based on generation time
            </p>
          </div>
        </div>

        {/* Risk Level Distribution */}
        <div className="bg-card border border-border rounded-lg p-6 mb-6">
          <div className="flex items-center gap-2 mb-4">
            <AlertCircle className="w-5 h-5 text-primary" />
            <h3 className="text-lg font-semibold">Risk Level Distribution</h3>
          </div>
          <div className="space-y-3">
            {Object.entries(analytics.risk_level_distribution).map(([level, count]) => {
              const percentage = analytics.total_executions > 0
                ? (count / analytics.total_executions) * 100
                : 0;
              return (
                <div key={level}>
                  <div className="flex items-center justify-between text-sm mb-1">
                    <span className="font-medium capitalize">{level}</span>
                    <span className="text-muted-foreground">
                      {count} ({percentage.toFixed(1)}%)
                    </span>
                  </div>
                  <div className="w-full h-2 bg-muted rounded-full overflow-hidden">
                    <div
                      className={`h-full ${
                        level.toLowerCase() === "safe"
                          ? "bg-green-500"
                          : level.toLowerCase() === "moderate"
                          ? "bg-yellow-500"
                          : level.toLowerCase() === "high"
                          ? "bg-orange-500"
                          : "bg-red-500"
                      }`}
                      style={{ width: `${percentage}%` }}
                    />
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Backend Usage */}
        <div className="bg-card border border-border rounded-lg p-6">
          <div className="flex items-center gap-2 mb-4">
            <BarChart3 className="w-5 h-5 text-primary" />
            <h3 className="text-lg font-semibold">Backend Usage</h3>
          </div>
          <div className="space-y-3">
            {Object.entries(analytics.backend_usage).map(([backend, count]) => {
              const percentage = analytics.total_executions > 0
                ? (count / analytics.total_executions) * 100
                : 0;
              return (
                <div key={backend}>
                  <div className="flex items-center justify-between text-sm mb-1">
                    <span className="font-medium capitalize">{backend}</span>
                    <span className="text-muted-foreground">
                      {count} ({percentage.toFixed(1)}%)
                    </span>
                  </div>
                  <div className="w-full h-2 bg-muted rounded-full overflow-hidden">
                    <div
                      className="h-full bg-primary"
                      style={{ width: `${percentage}%` }}
                    />
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}
