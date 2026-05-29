import { useState, useMemo, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import {
  ViewToggle,
  WordList,
  WordGrid,
  BottomBanner,
  DetailPanel,
  SearchInput,
  StatusFilter,
  TagSection,
} from "./components";
import { useWords } from "../../hooks/useWords";
import { useWordStore, type FilterStatus } from "../../stores/wordStore";
import { Avatar, Button } from "@heroui/react";
import { Settings } from "lucide-react";
import { useNavigate } from "react-router-dom";
import type { Word } from "../../lib/api";
import type { TagItem } from "../../types/tag";
import { getTags, createTag, deleteTag, updateTag } from "../../lib/api";
import { getNextSortOrder, type TagCreatePlacement } from "../../lib/tagSort";
type ViewMode = "grid" | "list";

function getWordTagIds(tags: string): string[] {
  return tags
    .split(/[,\s]+/)
    .map((tag) => tag.trim())
    .filter(Boolean);
}

export function HomePage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [viewMode, setViewMode] = useState<ViewMode>("grid");
  const [searchQuery, setSearchQuery] = useState("");
  const [filterStatus, setFilterStatus] = useState<FilterStatus>("all");
  const [selectedTag, setSelectedTag] = useState<string | null>(null);
  const [tags, setTags] = useState<TagItem[]>([]);

  const selectedWord = useWordStore((state) => state.selectedWord);
  const setSelectedWord = useWordStore((state) => state.setSelectedWord);

  const { data: words = [], isLoading } = useWords(searchQuery || undefined);
  // 加载标签
  useEffect(() => {
    getTags().then(setTags);
  }, []);

  const filteredWords = useMemo(() => {
    let result = words;

    if (filterStatus !== "all") {
      result = result.filter((w) => w.status === filterStatus);
    }

    if (selectedTag) {
      result = result.filter((w) =>
        getWordTagIds(w.tags).includes(selectedTag),
      );
    }

    return result;
  }, [words, filterStatus, selectedTag]);

  const tagCounts = useMemo(() => {
    return words.reduce<Record<string, number>>((counts, word) => {
      for (const tagId of getWordTagIds(word.tags)) {
        counts[tagId] = (counts[tagId] ?? 0) + 1;
      }
      return counts;
    }, {});
  }, [words]);

  const dueCount = useMemo(() => {
    return words.filter((w) => {
      const nextReview = w.next_review_at;
      if (!nextReview) return false;
      return nextReview <= Date.now();
    }).length;
  }, [words]);

  const handleWordClick = (word: Word) => {
    setSelectedWord(word);
  };

  const handleCloseDetail = () => {
    setSelectedWord(null);
  };

  const handleTagCreate = useCallback(
    async (
      name: string,
      color: string,
      options?: {
        anchorTagId?: string;
        placement?: TagCreatePlacement;
      },
    ) => {
      const result = getNextSortOrder(
        tags,
        options?.placement ?? "end",
        options?.anchorTagId,
      );

      if (result.kind === "insert") {
        const newTag = await createTag({
          name,
          color,
          sort_order: result.order,
        });
        setTags((prev) =>
          [...prev, newTag].sort((a, b) => a.sort_order - b.sort_order),
        );
      } else {
        // Update each existing tag's sort_order individually to preserve the gap for the new tag
        for (const t of result.reorderedTags) {
          await updateTag(t.id, { sort_order: t.sort_order });
        }
        const newTag = await createTag({
          name,
          color,
          sort_order: result.newOrder,
        });
        setTags(
          [...result.reorderedTags, newTag].sort(
            (a, b) => a.sort_order - b.sort_order,
          ),
        );
      }
    },
    [tags],
  );

  const handleTagUpdate = useCallback(
    async (tagId: string, name: string, color: string) => {
      const updatedTag = await updateTag(tagId, { name, color });
      setTags((prev) =>
        prev
          .map((tag) => (tag.id === tagId ? updatedTag : tag))
          .sort((a, b) => a.sort_order - b.sort_order),
      );
    },
    [],
  );

  const handleTagDelete = useCallback(async (tagId: string) => {
    await deleteTag(tagId);
    setTags((prev) => prev.filter((t) => t.id !== tagId));
  }, []);

  const handleTagReorder = useCallback((reorderedTags: TagItem[]) => {
    setTags(reorderedTags);
  }, []);

  const handleMouseEnter = () => {
    console.log("enter");
  };

  return (
    <div className="flex h-screen" onMouseEnter={handleMouseEnter}>
      {/* Sidebar */}
      <aside className="w-60 p-4 flex flex-col gap-4">
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
          tags={tags}
          tagCounts={tagCounts}
          selectedTag={selectedTag}
          onTagSelect={setSelectedTag}
          onTagCreate={handleTagCreate}
          onTagUpdate={handleTagUpdate}
          onTagDelete={handleTagDelete}
          onTagReorder={handleTagReorder}
        />
        <div className="mt-auto">
          <Button variant="ghost" onClick={() => navigate("/settings")}>
            <Settings size={18} />
          </Button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex flex-col bg-background">
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
