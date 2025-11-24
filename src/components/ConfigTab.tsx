import { useState, useEffect } from "react";
import { useAppStore } from "../store";
import { getConfig, updateConfig } from "../lib/tauri";
import { Save, RefreshCw } from "lucide-react";
import type { UserConfiguration } from "../types";

export default function ConfigTab() {
  const { config, setConfig } = useAppStore();
  const [localConfig, setLocalConfig] = useState<UserConfiguration | null>(null);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);

  useEffect(() => {
    if (config) {
      setLocalConfig(config);
    }
  }, [config]);

  const handleSave = async () => {
    if (!localConfig) return;

    setSaving(true);
    setMessage(null);

    try {
      await updateConfig(localConfig);
      setConfig(localConfig);
      setMessage({ type: "success", text: "Configuration saved successfully!" });

      // Clear message after 3 seconds
      setTimeout(() => setMessage(null), 3000);
    } catch (error) {
      console.error("Failed to save config:", error);
      setMessage({ type: "error", text: `Error: ${error}` });
    } finally {
      setSaving(false);
    }
  };

  const handleReload = async () => {
    try {
      const freshConfig = await getConfig();
      setConfig(freshConfig);
      setLocalConfig(freshConfig);
      setMessage({ type: "success", text: "Configuration reloaded!" });
      setTimeout(() => setMessage(null), 3000);
    } catch (error) {
      console.error("Failed to reload config:", error);
      setMessage({ type: "error", text: `Error: ${error}` });
    }
  };

  if (!localConfig) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      <div className="p-6 border-b border-border flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Configuration</h2>
          <p className="text-sm text-muted-foreground mt-1">
            Manage your cmdai settings
          </p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={handleReload}
            className="px-4 py-2 bg-secondary text-secondary-foreground rounded-lg hover:bg-secondary/90 flex items-center gap-2"
          >
            <RefreshCw className="w-4 h-4" />
            Reload
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            className="px-6 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50 flex items-center gap-2"
          >
            <Save className="w-4 h-4" />
            {saving ? "Saving..." : "Save Changes"}
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-2xl space-y-6">
          {message && (
            <div
              className={`p-4 rounded-lg ${
                message.type === "success"
                  ? "bg-green-50 text-green-800 border border-green-200"
                  : "bg-red-50 text-red-800 border border-red-200"
              }`}
            >
              {message.text}
            </div>
          )}

          {/* Safety Level */}
          <div className="space-y-2">
            <label className="block text-sm font-medium">Safety Level</label>
            <select
              value={localConfig.safety_level}
              onChange={(e) =>
                setLocalConfig({ ...localConfig, safety_level: e.target.value })
              }
              className="w-full px-4 py-2 rounded-lg border border-input bg-background focus:outline-none focus:ring-2 focus:ring-ring"
            >
              <option value="strict">Strict - Block all potentially dangerous commands</option>
              <option value="moderate">Moderate - Warn about dangerous commands</option>
              <option value="permissive">Permissive - Allow all commands with warnings</option>
            </select>
            <p className="text-sm text-muted-foreground">
              Controls how cmdai handles potentially dangerous commands
            </p>
          </div>

          {/* Default Shell */}
          <div className="space-y-2">
            <label className="block text-sm font-medium">Default Shell</label>
            <select
              value={localConfig.default_shell || "bash"}
              onChange={(e) =>
                setLocalConfig({ ...localConfig, default_shell: e.target.value || null })
              }
              className="w-full px-4 py-2 rounded-lg border border-input bg-background focus:outline-none focus:ring-2 focus:ring-ring"
            >
              <option value="bash">Bash</option>
              <option value="zsh">Zsh</option>
              <option value="fish">Fish</option>
              <option value="sh">POSIX sh</option>
              <option value="powershell">PowerShell</option>
            </select>
            <p className="text-sm text-muted-foreground">
              The shell type to use by default for command generation
            </p>
          </div>

          {/* Log Level */}
          <div className="space-y-2">
            <label className="block text-sm font-medium">Log Level</label>
            <select
              value={localConfig.log_level}
              onChange={(e) =>
                setLocalConfig({ ...localConfig, log_level: e.target.value })
              }
              className="w-full px-4 py-2 rounded-lg border border-input bg-background focus:outline-none focus:ring-2 focus:ring-ring"
            >
              <option value="error">Error</option>
              <option value="warn">Warning</option>
              <option value="info">Info</option>
              <option value="debug">Debug</option>
              <option value="trace">Trace</option>
            </select>
            <p className="text-sm text-muted-foreground">
              Verbosity level for application logging
            </p>
          </div>

          {/* Default Model */}
          <div className="space-y-2">
            <label className="block text-sm font-medium">Default Model</label>
            <input
              type="text"
              value={localConfig.default_model || ""}
              onChange={(e) =>
                setLocalConfig({ ...localConfig, default_model: e.target.value || null })
              }
              placeholder="e.g., codellama:7b"
              className="w-full px-4 py-2 rounded-lg border border-input bg-background focus:outline-none focus:ring-2 focus:ring-ring"
            />
            <p className="text-sm text-muted-foreground">
              Model to use for command generation (leave empty for default)
            </p>
          </div>

          {/* Cache Max Size */}
          <div className="space-y-2">
            <label className="block text-sm font-medium">
              Cache Max Size (GB)
            </label>
            <input
              type="number"
              value={localConfig.cache_max_size_gb}
              onChange={(e) =>
                setLocalConfig({
                  ...localConfig,
                  cache_max_size_gb: parseInt(e.target.value) || 10,
                })
              }
              min="1"
              max="100"
              className="w-full px-4 py-2 rounded-lg border border-input bg-background focus:outline-none focus:ring-2 focus:ring-ring"
            />
            <p className="text-sm text-muted-foreground">
              Maximum disk space to use for model cache
            </p>
          </div>

          {/* Log Rotation Days */}
          <div className="space-y-2">
            <label className="block text-sm font-medium">
              Log Rotation (Days)
            </label>
            <input
              type="number"
              value={localConfig.log_rotation_days}
              onChange={(e) =>
                setLocalConfig({
                  ...localConfig,
                  log_rotation_days: parseInt(e.target.value) || 7,
                })
              }
              min="1"
              max="365"
              className="w-full px-4 py-2 rounded-lg border border-input bg-background focus:outline-none focus:ring-2 focus:ring-ring"
            />
            <p className="text-sm text-muted-foreground">
              Number of days to keep log files before rotation
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
