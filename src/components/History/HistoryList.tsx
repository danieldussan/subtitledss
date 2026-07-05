import { useState, useEffect, useRef, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Search,
  Trash2,
  MessageSquare,
  Clock,
  X,
  Loader2,
  Download,
  ChevronDown,
  ChevronRight,
} from "lucide-react";
import { ExportDialog } from "../Export/ExportDialog";

interface HistoryEntry {
  id: number;
  timestamp: string;
  language: string;
  original_text: string;
  translation: string | null;
  source_app: string | null;
}

function getDateGroup(timestamp: string): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const days = Math.floor(diff / (1000 * 60 * 60 * 24));

  if (days === 0) return "Today";
  if (days === 1) return "Yesterday";
  if (days < 7) return "This Week";
  if (days < 30) return "This Month";
  return "Older";
}

function groupEntriesByDate(entries: HistoryEntry[]): Map<string, HistoryEntry[]> {
  const groups = new Map<string, HistoryEntry[]>();
  for (const entry of entries) {
    const group = getDateGroup(entry.timestamp);
    if (!groups.has(group)) {
      groups.set(group, []);
    }
    groups.get(group)!.push(entry);
  }
  return groups;
}

export function HistoryList() {
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<HistoryEntry[]>([]);
  const [showExportDialog, setShowExportDialog] = useState(false);
  const [expandedGroups, setExpandedGroups] = useState<Set<string>>(
    new Set(["Today", "Yesterday"]),
  );
  const [expandedEntries, setExpandedEntries] = useState<Set<number>>(new Set());
  const searchInputRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    loadHistory();
  }, []);

  const loadHistory = async () => {
    try {
      setLoading(true);
      const result = await invoke<HistoryEntry[]>("get_history", { limit: 100 });
      setEntries(result);
      setError(null);
    } catch (err) {
      const msg = typeof err === "string" ? err : "Failed to load history";
      setError(msg);
      console.error("Failed to load history:", err);
    } finally {
      setLoading(false);
    }
  };

  const handleSearch = async (query: string) => {
    if (!query.trim()) {
      setSearchResults([]);
      return;
    }
    try {
      const results = await invoke<HistoryEntry[]>("search_history", {
        query,
        limit: 50,
      });
      setSearchResults(results);
      setError(null);
    } catch (err) {
      const msg = typeof err === "string" ? err : "Search failed";
      setError(msg);
      console.error("Search failed:", err);
    }
  };

  const handleSearchInputChange = (value: string) => {
    setSearchQuery(value);
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }
    debounceRef.current = setTimeout(() => {
      handleSearch(value);
    }, 300);
  };

  const handleClear = async () => {
    if (confirm("Are you sure you want to clear all history?")) {
      try {
        await invoke("clear_history");
        setEntries([]);
        setSearchResults([]);
        setError(null);
      } catch (err) {
        const msg = typeof err === "string" ? err : "Failed to clear history";
        setError(msg);
        console.error("Failed to clear history:", err);
      }
    }
  };

  const handleDeleteEntry = async (id: number) => {
    try {
      await invoke("delete_history_entry", { id });
      setEntries(entries.filter((e) => e.id !== id));
      setSearchResults(searchResults.filter((e) => e.id !== id));
    } catch (err) {
      console.error("Failed to delete entry:", err);
    }
  };

  const clearSearch = () => {
    setSearchQuery("");
    setSearchResults([]);
    setError(null);
    searchInputRef.current?.focus();
  };

  const toggleGroup = (group: string) => {
    const newExpanded = new Set(expandedGroups);
    if (newExpanded.has(group)) {
      newExpanded.delete(group);
    } else {
      newExpanded.add(group);
    }
    setExpandedGroups(newExpanded);
  };

  const toggleEntry = (id: number) => {
    const newExpanded = new Set(expandedEntries);
    if (newExpanded.has(id)) {
      newExpanded.delete(id);
    } else {
      newExpanded.add(id);
    }
    setExpandedEntries(newExpanded);
  };

  const displayEntries = searchResults.length > 0 ? searchResults : entries;
  const groupedEntries = useMemo(() => groupEntriesByDate(displayEntries), [displayEntries]);

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="px-5 pt-4 pb-3 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <h2 className="text-[15px] font-semibold text-text-primary">History</h2>
          {entries.length > 0 && (
            <span className="text-[11px] text-text-muted bg-bg-surface px-2 py-0.5 rounded-full">
              {entries.length}
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          {entries.length > 0 && (
            <>
              <button
                onClick={() => setShowExportDialog(true)}
                className="btn btn-ghost btn-sm gap-1.5"
              >
                <Download size={12} />
                Export
              </button>
              <button onClick={handleClear} className="btn btn-danger btn-sm gap-1.5">
                <Trash2 size={12} />
                Clear
              </button>
            </>
          )}
        </div>
      </div>

      {/* Search */}
      <div className="px-5 pb-3">
        <div className="relative">
          <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted" />
          <input
            ref={searchInputRef}
            type="text"
            value={searchQuery}
            onChange={(e) => handleSearchInputChange(e.target.value)}
            placeholder="Search transcriptions..."
            className="input pl-9 pr-8"
          />
          {searchQuery && (
            <button
              onClick={clearSearch}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-text-muted hover:text-text-primary"
            >
              <X size={14} />
            </button>
          )}
        </div>
      </div>

      {/* Error */}
      {error && (
        <div className="mx-5 mb-3 px-3 py-2 bg-danger-subtle border border-danger/20 rounded-lg text-[12px] text-danger">
          {error}
        </div>
      )}

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-5 pb-4">
        {loading ? (
          <div className="empty-state">
            <Loader2 size={24} className="animate-spin mb-3" />
            <span>Loading history...</span>
          </div>
        ) : displayEntries.length === 0 ? (
          <div className="empty-state">
            <MessageSquare size={32} className="mb-3 opacity-30" />
            <span className="text-[13px]">
              {searchQuery ? "No results found" : "No transcriptions yet"}
            </span>
            <span className="text-[12px] mt-1">
              {searchQuery
                ? "Try a different search term"
                : "Start capturing to begin transcription"}
            </span>
          </div>
        ) : (
          <div className="space-y-3">
            {Array.from(groupedEntries.entries()).map(([group, groupEntries]) => (
              <div key={group}>
                <button
                  onClick={() => toggleGroup(group)}
                  className="flex items-center gap-2 w-full text-left mb-2"
                >
                  {expandedGroups.has(group) ? (
                    <ChevronDown size={14} className="text-text-muted" />
                  ) : (
                    <ChevronRight size={14} className="text-text-muted" />
                  )}
                  <span className="text-[12px] font-medium text-text-secondary uppercase tracking-wide">
                    {group}
                  </span>
                  <span className="text-[10px] text-text-muted">({groupEntries.length})</span>
                </button>

                {expandedGroups.has(group) && (
                  <div className="space-y-2 ml-6">
                    {groupEntries.map((entry) => (
                      <div key={entry.id} className="card p-3">
                        <div
                          className="flex items-center gap-2 mb-2 cursor-pointer"
                          onClick={() => toggleEntry(entry.id)}
                        >
                          <Clock size={12} className="text-text-muted" />
                          <span className="text-[11px] text-text-muted font-mono">
                            {new Date(entry.timestamp).toLocaleString()}
                          </span>
                          <span className="text-[10px] text-text-muted bg-bg-surface px-1.5 py-0.5 rounded font-mono">
                            {entry.language}
                          </span>
                          <div className="flex-1" />
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              handleDeleteEntry(entry.id);
                            }}
                            className="btn btn-ghost btn-sm p-1 opacity-0 group-hover:opacity-100 hover:text-danger"
                          >
                            <Trash2 size={12} />
                          </button>
                        </div>
                        <p
                          className={`text-[13px] text-text-primary leading-relaxed ${
                            expandedEntries.has(entry.id) ? "" : "line-clamp-2"
                          }`}
                        >
                          {entry.original_text}
                        </p>
                        {entry.translation && expandedEntries.has(entry.id) && (
                          <p className="text-[12px] text-text-secondary mt-1.5 italic">
                            {entry.translation}
                          </p>
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Export Dialog */}
      {showExportDialog && (
        <ExportDialog entries={displayEntries} onClose={() => setShowExportDialog(false)} />
      )}
    </div>
  );
}
