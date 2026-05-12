import { ListBox, ListBoxItem, Chip } from "@heroui/react";
import { useTranslation } from "react-i18next";
import type { Word } from "../../lib/api";
import type { FilterStatus } from "../../stores/wordStore";
import { Album, SquarePlus, ScanEye, ListChecks } from "lucide-react";
import { JSX } from "react";
import FireSvg from "@src/assets/svg/fire.svg";
interface StatusFilterProps {
  words: Word[];
  currentFilter: FilterStatus;
  onFilterChange: (status: FilterStatus) => void;
}

const statusIcons: Record<FilterStatus, (() => JSX.Element) | undefined> = {
  all: () => <Album size={16} />,
  reviewing: () => <img src={FireSvg} width={16} height={16} />,
  learning: () => <ScanEye size={16} />,
  mastered: () => <ListChecks size={16} />,
  new: () => <SquarePlus size={16} />,
};

const statusLabelKeys: Record<FilterStatus, string> = {
  all: "home.status.all",
  reviewing: "home.status.dueToday",
  learning: "home.status.learning",
  mastered: "home.status.mastered",
  new: "home.status.newWord",
};

export function StatusFilter({
  words,
  currentFilter,
  onFilterChange,
}: StatusFilterProps) {
  const { t } = useTranslation();
  const counts = {
    all: words.length,
    new: words.filter((w) => w.status === "new").length,
    learning: words.filter((w) => w.status === "learning").length,
    reviewing: words.filter((w) => {
      const nextReview = w.next_review_at;
      if (!nextReview) return false;
      return nextReview <= Date.now();
    }).length,
    mastered: words.filter((w) => w.status === "mastered").length,
  };

  const statusKeys: FilterStatus[] = ["all", "reviewing", "learning", "mastered", "new"];

  return (
    <div className="flex flex-col gap-2">
      <span className="text-sm font-medium text-gray-500">{t("home.status.label")}</span>
      <ListBox
        aria-label={t("home.status.ariaLabel")}
        selectionMode="single"
        selectedKeys={[currentFilter]}
        onSelectionChange={(keys) => {
          const key = Array.from(keys)[0] as FilterStatus;
          if (key) onFilterChange(key);
        }}
      >
        {statusKeys.map((key) => (
          <ListBoxItem
            key={key}
            id={key}
            className={`group py-2 hover:bg-accent-6 hover:text-accent-1 ${currentFilter === key ? "bg-accent-6 text-accent-1" : ""}`}
          >
            <div className="flex items-center gap-2 w-full ">
              {statusIcons[key]?.()}
              <span className="flex-1">{t(statusLabelKeys[key])}</span>
              <Chip
                size="sm"
                variant="soft"
                className={`group-hover:bg-accent-4 group-hover:text-accent-1 ${currentFilter === key ? "bg-accent-4 text-accent-1" : ""}`}
              >
                {counts[key]}
              </Chip>
            </div>
          </ListBoxItem>
        ))}
      </ListBox>
    </div>
  );
}
