import { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { SearchInput } from "./SearchInput";
import { ViewToggle } from "./ViewToggle";
import { StatusFilter } from "./StatusFilter";
import { TagSection } from "./TagSection";
import { WordGrid } from "./WordGrid";
import { WordList } from "./WordList";
import { BottomBanner } from "./BottomBanner";
import { DetailPanel } from "./DetailPanel";
import { useWords } from "../../hooks/useWords";
import { useWordStore, type FilterStatus } from "../../stores/wordStore";
import { Avatar, Button } from "@heroui/react";
import { Settings } from "lucide-react";
import { useNavigate } from "react-router-dom";
import type { Word } from "../../lib/api";

type ViewMode = "grid" | "list";

export function HomePage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [viewMode, setViewMode] = useState<ViewMode>("grid");
  const [searchQuery, setSearchQuery] = useState("");
  const [filterStatus, setFilterStatus] = useState<FilterStatus>("all");
  const [selectedTag, setSelectedTag] = useState<string | null>(null);

  const selectedWord = useWordStore((state) => state.selectedWord);
  const setSelectedWord = useWordStore((state) => state.setSelectedWord);

  const { data: words = [], isLoading } = useWords(searchQuery || undefined);

  const filteredWords = useMemo(() => {
    let result = words;

    if (filterStatus !== "all") {
      result = result.filter((w) => w.status === filterStatus);
    }

    if (selectedTag) {
      result = result.filter((w) => w.tags.includes(selectedTag));
    }

    return result;
  }, [words, filterStatus, selectedTag]);

  const dueCount = useMemo(() => {
    return words.filter((w) => {
      const nextReview = w.next_review_at;
      if (!nextReview) return false;
      return nextReview <= Date.now();
    }).length;
  }, [words]);

  const allTags = useMemo(() => {
    const tagSet = new Set<string>();
    words.forEach((w) => {
      if (w.tags) {
        w.tags.split(",").forEach((t) => {
          const trimmed = t.trim();
          if (trimmed) tagSet.add(trimmed);
        });
      }
    });
    return Array.from(tagSet).sort();
  }, [words]);

  const handleWordClick = (word: Word) => {
    setSelectedWord(word);
  };

  const handleCloseDetail = () => {
    setSelectedWord(null);
  };

  return (
    <div className="flex h-screen">
      {/* Sidebar */}
      <aside className="bg-background w-60 p-4 flex flex-col gap-4">
        <div className="flex items-center">
          <Avatar size="lg">
            <Avatar.Image
              alt="Small Avatar"
              src="https://heroui-assets.nyc3.cdn.digitaloceanspaces.com/avatars/blue.jpg"
            />
            <Avatar.Fallback>SM</Avatar.Fallback>
          </Avatar>
          <span className="ml-2 font-bold">{t("app.name")}</span>
        </div>
        <StatusFilter
          words={words}
          currentFilter={filterStatus}
          onFilterChange={setFilterStatus}
        />
        <TagSection
          tags={allTags}
          selectedTag={selectedTag}
          onTagSelect={setSelectedTag}
        />
        <div className="mt-auto">
          <Button variant="ghost" onClick={() => navigate("/settings")}>
            <Settings size={18} />
          </Button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex flex-col">
        {/* TopBar */}
        <header className="h-14 px-4 flex items-center gap-4">
          <SearchInput value={searchQuery} onChange={setSearchQuery} />
          <ViewToggle mode={viewMode} onModeChange={setViewMode} />
        </header>

        {/* Content */}
        <div className="flex-1 overflow-auto p-4">
          {isLoading ? (
            <div className="flex items-center justify-center h-full">
              <span className="text-foreground-400">{t("app.loading")}</span>
            </div>
          ) : filteredWords.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full gap-4">
              <span className="text-foreground-400">{t("app.emptyState")}</span>
              <button className="px-4 py-2 bg-primary text-primary-foreground rounded">
                {t("app.emptyAction")}
              </button>
            </div>
          ) : viewMode === "grid" ? (
            <WordGrid words={filteredWords} onWordClick={handleWordClick} />
          ) : (
            <WordList words={filteredWords} onWordClick={handleWordClick} />
          )}
        </div>

        {/* BottomBanner */}
        {dueCount > 0 && (
          <BottomBanner dueCount={dueCount} totalCount={words.length} />
        )}
      </main>

      {/* DetailPanel */}
      {selectedWord && (
        <DetailPanel word={selectedWord} onClose={handleCloseDetail} />
      )}
    </div>
  );
}
