import { useState } from "react";
import { useAppStore } from "../store";
import { generateCommand } from "../lib/tauri";
import { Shell } from "@tauri-apps/plugin-shell";
import {
  AlertCircle,
  CheckCircle2,
  Copy,
  PlayCircle,
  Search,
  Loader2,
} from "lucide-react";
import { cn, getRiskLevelColor, formatDuration } from "../lib/utils";
import type { CommandGenerationResponse } from "../types";

export default function CommandTab() {
  const { addToHistory } = useAppStore();
  const [prompt, setPrompt] = useState("");
  const [shell, setShell] = useState("bash");
  const [safetyLevel, setSafetyLevel] = useState("moderate");
  const [dryRun, setDryRun] = useState(true);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<CommandGenerationResponse | null>(null);
  const [output, setOutput] = useState<string>("");
  const [executing, setExecuting] = useState(false);

  const handleGenerate = async () => {
    if (!prompt.trim()) return;

    setLoading(true);
    setResult(null);
    setOutput("");

    try {
      const response = await generateCommand({
        prompt,
        shell,
        safety_level: safetyLevel,
        dry_run: dryRun,
      });
      setResult(response);
    } catch (error) {
      console.error("Failed to generate command:", error);
      alert(`Error: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const handleExecute = async () => {
    if (!result) return;

    setExecuting(true);
    setOutput("");

    try {
      const command = await Shell.create("sh", ["-c", result.generated_command]);
      command.stdout.on("data", (line) => {
        setOutput((prev) => prev + line);
      });
      command.stderr.on("data", (line) => {
        setOutput((prev) => prev + `[ERROR] ${line}`);
      });

      await command.execute();
    } catch (error) {
      setOutput((prev) => prev + `\n[ERROR] ${error}`);
    } finally {
      setExecuting(false);
    }
  };

  const handleCopy = () => {
    if (result) {
      navigator.clipboard.writeText(result.generated_command);
    }
  };

  return (
    <div className="h-full flex flex-col">
      <div className="p-6 border-b border-border">
        <h2 className="text-2xl font-bold">Generate Command</h2>
        <p className="text-sm text-muted-foreground mt-1">
          Convert natural language to shell commands
        </p>
      </div>

      <div className="flex-1 overflow-y-auto p-6 space-y-6">
        {/* Input Section */}
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-2">
              Describe what you want to do
            </label>
            <textarea
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              placeholder="e.g., list all files in current directory"
              className="w-full px-4 py-3 rounded-lg border border-input bg-background resize-none focus:outline-none focus:ring-2 focus:ring-ring"
              rows={3}
            />
          </div>

          <div className="grid grid-cols-3 gap-4">
            <div>
              <label className="block text-sm font-medium mb-2">Shell</label>
              <select
                value={shell}
                onChange={(e) => setShell(e.target.value)}
                className="w-full px-4 py-2 rounded-lg border border-input bg-background focus:outline-none focus:ring-2 focus:ring-ring"
              >
                <option value="bash">Bash</option>
                <option value="zsh">Zsh</option>
                <option value="fish">Fish</option>
                <option value="sh">POSIX sh</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">
                Safety Level
              </label>
              <select
                value={safetyLevel}
                onChange={(e) => setSafetyLevel(e.target.value)}
                className="w-full px-4 py-2 rounded-lg border border-input bg-background focus:outline-none focus:ring-2 focus:ring-ring"
              >
                <option value="strict">Strict</option>
                <option value="moderate">Moderate</option>
                <option value="permissive">Permissive</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">Mode</label>
              <label className="flex items-center gap-2 px-4 py-2 rounded-lg border border-input bg-background cursor-pointer">
                <input
                  type="checkbox"
                  checked={dryRun}
                  onChange={(e) => setDryRun(e.target.checked)}
                  className="w-4 h-4"
                />
                <span className="text-sm">Dry Run (Inspect Only)</span>
              </label>
            </div>
          </div>

          <button
            onClick={handleGenerate}
            disabled={loading || !prompt.trim()}
            className="w-full px-6 py-3 bg-primary text-primary-foreground rounded-lg font-medium hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
          >
            {loading ? (
              <>
                <Loader2 className="w-5 h-5 animate-spin" />
                Generating...
              </>
            ) : (
              <>
                <Search className="w-5 h-5" />
                Generate Command
              </>
            )}
          </button>
        </div>

        {/* Result Section */}
        {result && (
          <div className="space-y-4">
            {/* Status */}
            <div
              className={cn(
                "p-4 rounded-lg border",
                result.blocked_reason
                  ? "bg-destructive/10 border-destructive"
                  : "bg-card border-border"
              )}
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  {result.blocked_reason ? (
                    <AlertCircle className="w-5 h-5 text-destructive" />
                  ) : (
                    <CheckCircle2 className="w-5 h-5 text-green-600" />
                  )}
                  <span className="font-medium">
                    {result.blocked_reason ? "Command Blocked" : "Command Generated"}
                  </span>
                </div>
                <div className="flex items-center gap-4">
                  <span
                    className={cn(
                      "px-3 py-1 rounded-full text-xs font-medium",
                      getRiskLevelColor(result.risk_level)
                    )}
                  >
                    {result.risk_level}
                  </span>
                  <span className="text-sm text-muted-foreground">
                    {formatDuration(result.generation_time_ms)}
                  </span>
                </div>
              </div>
              {result.blocked_reason && (
                <p className="mt-2 text-sm text-destructive">
                  {result.blocked_reason}
                </p>
              )}
            </div>

            {/* Command */}
            {!result.blocked_reason && (
              <div className="space-y-2">
                <label className="block text-sm font-medium">Generated Command</label>
                <div className="relative">
                  <pre className="p-4 bg-muted rounded-lg overflow-x-auto font-mono text-sm">
                    {result.generated_command}
                  </pre>
                  <div className="absolute top-2 right-2 flex gap-2">
                    <button
                      onClick={handleCopy}
                      className="p-2 bg-background rounded hover:bg-accent"
                      title="Copy command"
                    >
                      <Copy className="w-4 h-4" />
                    </button>
                    {!dryRun && (
                      <button
                        onClick={handleExecute}
                        disabled={executing}
                        className="p-2 bg-primary text-primary-foreground rounded hover:bg-primary/90 disabled:opacity-50"
                        title="Execute command"
                      >
                        <PlayCircle className="w-4 h-4" />
                      </button>
                    )}
                  </div>
                </div>
              </div>
            )}

            {/* Explanation */}
            {result.explanation && (
              <div className="space-y-2">
                <label className="block text-sm font-medium">Explanation</label>
                <p className="p-4 bg-muted rounded-lg text-sm">
                  {result.explanation}
                </p>
              </div>
            )}

            {/* Warnings */}
            {result.warnings.length > 0 && (
              <div className="space-y-2">
                <label className="block text-sm font-medium">Warnings</label>
                <div className="space-y-2">
                  {result.warnings.map((warning, i) => (
                    <div
                      key={i}
                      className="p-3 bg-yellow-50 border border-yellow-200 rounded-lg text-sm flex items-start gap-2"
                    >
                      <AlertCircle className="w-4 h-4 text-yellow-600 mt-0.5 flex-shrink-0" />
                      <span>{warning}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Alternatives */}
            {result.alternatives.length > 0 && (
              <div className="space-y-2">
                <label className="block text-sm font-medium">Alternative Commands</label>
                <div className="space-y-2">
                  {result.alternatives.map((alt, i) => (
                    <div key={i} className="p-3 bg-muted rounded-lg font-mono text-sm">
                      {alt}
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Output */}
            {output && (
              <div className="space-y-2">
                <label className="block text-sm font-medium">Command Output</label>
                <pre className="p-4 bg-muted rounded-lg overflow-x-auto text-sm max-h-96">
                  {output}
                </pre>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
