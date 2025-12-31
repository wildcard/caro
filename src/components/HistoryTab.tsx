import { useState, useEffect } from "react";
import { useAppStore } from "../store";
import {
  getExecutionHistory,
  deleteExecution,
  rateExecution,
  voteExecution,
  getExecutionRatings,
  getExecutionVotes,
} from "../lib/tauri";
import {
  Search,
  Filter,
  Trash2,
  ThumbsUp,
  ThumbsDown,
  Star,
  Calendar,
  Clock,
  Terminal as TerminalIcon,
  AlertTriangle,
} from "lucide-react";
import { cn, formatTimestamp, formatDuration, getRiskLevelColor } from "../lib/utils";
import type { ExecutionRecord, ExecutionRating, ExecutionVote } from "../types";

export default function HistoryTab() {
  const { history, setHistory, historyFilter, setHistoryFilter, selectedExecution, setSelectedExecution } = useAppStore();
  const [searchQuery, setSearchQuery] = useState("");
  const [showFilters, setShowFilters] = useState(false);
  const [ratings, setRatings] = useState<ExecutionRating[]>([]);
  const [votes, setVotes] = useState<ExecutionVote[]>([]);
  const [ratingValue, setRatingValue] = useState(5);
  const [ratingFeedback, setRatingFeedback] = useState("");

  useEffect(() => {
    loadHistory();
  }, [historyFilter]);

  useEffect(() => {
    if (selectedExecution?.id) {
      loadRatingsAndVotes(selectedExecution.id);
    }
  }, [selectedExecution]);

  const loadHistory = async () => {
    try {
      const data = await getExecutionHistory(historyFilter);
      setHistory(data);
    } catch (error) {
      console.error("Failed to load history:", error);
    }
  };

  const loadRatingsAndVotes = async (executionId: number) => {
    try {
      const [ratingsData, votesData] = await Promise.all([
        getExecutionRatings(executionId),
        getExecutionVotes(executionId),
      ]);
      setRatings(ratingsData);
      setVotes(votesData);
    } catch (error) {
      console.error("Failed to load ratings/votes:", error);
    }
  };

  const handleSearch = () => {
    setHistoryFilter({
      ...historyFilter,
      search_query: searchQuery || null,
    });
  };

  const handleDelete = async (id: number) => {
    if (!confirm("Are you sure you want to delete this execution?")) return;

    try {
      await deleteExecution(id);
      loadHistory();
      if (selectedExecution?.id === id) {
        setSelectedExecution(null);
      }
    } catch (error) {
      console.error("Failed to delete execution:", error);
      alert(`Error: ${error}`);
    }
  };

  const handleVote = async (voteType: "up" | "down") => {
    if (!selectedExecution?.id) return;

    try {
      await voteExecution(selectedExecution.id, voteType);
      loadRatingsAndVotes(selectedExecution.id);
    } catch (error) {
      console.error("Failed to vote:", error);
      alert(`Error: ${error}`);
    }
  };

  const handleRate = async () => {
    if (!selectedExecution?.id) return;

    try {
      await rateExecution(selectedExecution.id, ratingValue, ratingFeedback || undefined);
      loadRatingsAndVotes(selectedExecution.id);
      setRatingFeedback("");
    } catch (error) {
      console.error("Failed to rate:", error);
      alert(`Error: ${error}`);
    }
  };

  const voteCount = {
    up: votes.filter((v) => v.vote_type === "up").length,
    down: votes.filter((v) => v.vote_type === "down").length,
  };

  const avgRating = ratings.length > 0
    ? ratings.reduce((sum, r) => sum + r.rating, 0) / ratings.length
    : 0;

  return (
    <div className="h-full flex">
      {/* History List */}
      <div className="w-96 border-r border-border flex flex-col">
        <div className="p-4 border-b border-border space-y-3">
          <h2 className="text-xl font-bold">Execution History</h2>

          <div className="flex gap-2">
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSearch()}
              placeholder="Search..."
              className="flex-1 px-3 py-2 rounded-lg border border-input bg-background text-sm focus:outline-none focus:ring-2 focus:ring-ring"
            />
            <button
              onClick={handleSearch}
              className="px-3 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90"
            >
              <Search className="w-4 h-4" />
            </button>
            <button
              onClick={() => setShowFilters(!showFilters)}
              className="px-3 py-2 bg-secondary text-secondary-foreground rounded-lg hover:bg-secondary/90"
            >
              <Filter className="w-4 h-4" />
            </button>
          </div>

          {showFilters && (
            <div className="space-y-2 p-3 bg-muted rounded-lg">
              <select
                value={historyFilter.shell_type || ""}
                onChange={(e) =>
                  setHistoryFilter({
                    ...historyFilter,
                    shell_type: e.target.value || null,
                  })
                }
                className="w-full px-3 py-2 rounded-lg border border-input bg-background text-sm"
              >
                <option value="">All Shells</option>
                <option value="bash">Bash</option>
                <option value="zsh">Zsh</option>
                <option value="fish">Fish</option>
              </select>

              <select
                value={
                  historyFilter.executed === null
                    ? ""
                    : historyFilter.executed
                    ? "executed"
                    : "not-executed"
                }
                onChange={(e) =>
                  setHistoryFilter({
                    ...historyFilter,
                    executed:
                      e.target.value === ""
                        ? null
                        : e.target.value === "executed",
                  })
                }
                className="w-full px-3 py-2 rounded-lg border border-input bg-background text-sm"
              >
                <option value="">All</option>
                <option value="executed">Executed</option>
                <option value="not-executed">Not Executed</option>
              </select>
            </div>
          )}
        </div>

        <div className="flex-1 overflow-y-auto">
          {history.length === 0 ? (
            <div className="p-8 text-center text-muted-foreground">
              <p>No execution history yet</p>
            </div>
          ) : (
            <div className="divide-y divide-border">
              {history.map((record) => (
                <div
                  key={record.id}
                  onClick={() => setSelectedExecution(record)}
                  className={cn(
                    "p-4 cursor-pointer hover:bg-accent transition-colors",
                    selectedExecution?.id === record.id && "bg-accent"
                  )}
                >
                  <div className="flex items-start justify-between gap-2 mb-2">
                    <p className="text-sm font-medium line-clamp-2">
                      {record.prompt}
                    </p>
                    <span
                      className={cn(
                        "px-2 py-0.5 rounded-full text-xs font-medium whitespace-nowrap",
                        getRiskLevelColor(record.risk_level)
                      )}
                    >
                      {record.risk_level}
                    </span>
                  </div>
                  <code className="text-xs bg-muted px-2 py-1 rounded block truncate mb-2">
                    {record.generated_command}
                  </code>
                  <div className="flex items-center gap-3 text-xs text-muted-foreground">
                    <span className="flex items-center gap-1">
                      <Calendar className="w-3 h-3" />
                      {new Date(record.timestamp).toLocaleDateString()}
                    </span>
                    <span className="flex items-center gap-1">
                      <TerminalIcon className="w-3 h-3" />
                      {record.shell_type}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Detail View */}
      <div className="flex-1 overflow-y-auto">
        {selectedExecution ? (
          <div className="p-6 space-y-6">
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <h3 className="text-xl font-bold mb-2">{selectedExecution.prompt}</h3>
                <div className="flex items-center gap-4 text-sm text-muted-foreground">
                  <span className="flex items-center gap-1">
                    <Calendar className="w-4 h-4" />
                    {formatTimestamp(selectedExecution.timestamp)}
                  </span>
                  <span className="flex items-center gap-1">
                    <Clock className="w-4 h-4" />
                    {formatDuration(selectedExecution.generation_time_ms)}
                  </span>
                  <span className="flex items-center gap-1">
                    <TerminalIcon className="w-4 h-4" />
                    {selectedExecution.shell_type}
                  </span>
                  <span
                    className={cn(
                      "px-3 py-1 rounded-full text-xs font-medium",
                      getRiskLevelColor(selectedExecution.risk_level)
                    )}
                  >
                    {selected Execution.risk_level}
                  </span>
                </div>
              </div>
              <button
                onClick={() => handleDelete(selectedExecution.id!)}
                className="p-2 text-destructive hover:bg-destructive/10 rounded-lg"
              >
                <Trash2 className="w-5 h-5" />
              </button>
            </div>

            {/* Command */}
            <div>
              <label className="block text-sm font-medium mb-2">Command</label>
              <pre className="p-4 bg-muted rounded-lg overflow-x-auto font-mono text-sm">
                {selectedExecution.generated_command}
              </pre>
            </div>

            {/* Explanation */}
            {selectedExecution.explanation && (
              <div>
                <label className="block text-sm font-medium mb-2">Explanation</label>
                <p className="p-4 bg-muted rounded-lg text-sm">
                  {selectedExecution.explanation}
                </p>
              </div>
            )}

            {/* Warnings */}
            {selectedExecution.warnings.length > 0 && (
              <div>
                <label className="block text-sm font-medium mb-2">Warnings</label>
                <div className="space-y-2">
                  {selectedExecution.warnings.map((warning, i) => (
                    <div
                      key={i}
                      className="p-3 bg-yellow-50 border border-yellow-200 rounded-lg text-sm flex items-start gap-2"
                    >
                      <AlertTriangle className="w-4 h-4 text-yellow-600 mt-0.5 flex-shrink-0" />
                      <span>{warning}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Voting */}
            <div className="border-t pt-6">
              <label className="block text-sm font-medium mb-3">Community Feedback</label>
              <div className="flex items-center gap-4">
                <button
                  onClick={() => handleVote("up")}
                  className="flex items-center gap-2 px-4 py-2 bg-green-50 text-green-700 rounded-lg hover:bg-green-100"
                >
                  <ThumbsUp className="w-5 h-5" />
                  <span className="font-medium">{voteCount.up}</span>
                </button>
                <button
                  onClick={() => handleVote("down")}
                  className="flex items-center gap-2 px-4 py-2 bg-red-50 text-red-700 rounded-lg hover:bg-red-100"
                >
                  <ThumbsDown className="w-5 h-5" />
                  <span className="font-medium">{voteCount.down}</span>
                </button>
                {avgRating > 0 && (
                  <div className="flex items-center gap-2 px-4 py-2 bg-yellow-50 text-yellow-700 rounded-lg">
                    <Star className="w-5 h-5 fill-current" />
                    <span className="font-medium">{avgRating.toFixed(1)}</span>
                    <span className="text-sm">({ratings.length})</span>
                  </div>
                )}
              </div>
            </div>

            {/* Rating Form */}
            <div className="border-t pt-6">
              <label className="block text-sm font-medium mb-3">Add Your Rating</label>
              <div className="space-y-3">
                <div className="flex items-center gap-2">
                  {[1, 2, 3, 4, 5].map((value) => (
                    <button
                      key={value}
                      onClick={() => setRatingValue(value)}
                      className={cn(
                        "p-2 rounded-lg transition-colors",
                        value <= ratingValue
                          ? "text-yellow-500"
                          : "text-gray-300 hover:text-gray-400"
                      )}
                    >
                      <Star className="w-6 h-6 fill-current" />
                    </button>
                  ))}
                  <span className="ml-2 text-sm font-medium">{ratingValue}/5</span>
                </div>
                <textarea
                  value={ratingFeedback}
                  onChange={(e) => setRatingFeedback(e.target.value)}
                  placeholder="Add feedback (optional)"
                  className="w-full px-4 py-3 rounded-lg border border-input bg-background resize-none focus:outline-none focus:ring-2 focus:ring-ring"
                  rows={3}
                />
                <button
                  onClick={handleRate}
                  className="px-6 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90"
                >
                  Submit Rating
                </button>
              </div>
            </div>

            {/* Previous Ratings */}
            {ratings.length > 0 && (
              <div className="border-t pt-6">
                <label className="block text-sm font-medium mb-3">Previous Ratings</label>
                <div className="space-y-3">
                  {ratings.map((rating) => (
                    <div key={rating.id} className="p-4 bg-muted rounded-lg">
                      <div className="flex items-center gap-2 mb-2">
                        {[1, 2, 3, 4, 5].map((value) => (
                          <Star
                            key={value}
                            className={cn(
                              "w-4 h-4 fill-current",
                              value <= rating.rating ? "text-yellow-500" : "text-gray-300"
                            )}
                          />
                        ))}
                        <span className="text-xs text-muted-foreground ml-2">
                          {formatTimestamp(rating.created_at)}
                        </span>
                      </div>
                      {rating.feedback && (
                        <p className="text-sm text-muted-foreground">{rating.feedback}</p>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        ) : (
          <div className="h-full flex items-center justify-center text-muted-foreground">
            <p>Select an execution to view details</p>
          </div>
        )}
      </div>
    </div>
  );
}
