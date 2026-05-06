import { ListBox, ListBoxItem, Chip } from "@heroui/react";
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

const statusOptions: {
  key: FilterStatus;
  label: string;
  icon?: () => JSX.Element;
}[] = [
  { key: "all", label: "全部", icon: () => <Album size={16} /> },
  {
    key: "reviewing",
    label: "今天待复习",
    icon: () => <img src={FireSvg} width={16} height={16} />,
  },
  { key: "learning", label: "复习中", icon: () => <ScanEye size={16} /> },
  { key: "mastered", label: "已记住", icon: () => <ListChecks size={16} /> },
  { key: "new", label: "新添加", icon: () => <SquarePlus size={16} /> },
];

export function StatusFilter({
  words,
  currentFilter,
  onFilterChange,
}: StatusFilterProps) {
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

  return (
    <div className="flex flex-col gap-2">
      <span className="text-sm font-medium text-gray-500">学习</span>
      <ListBox
        aria-label="状态筛选"
        selectionMode="single"
        selectedKeys={[currentFilter]}
        onSelectionChange={(keys) => {
          const key = Array.from(keys)[0] as FilterStatus;
          if (key) onFilterChange(key);
        }}
      >
        {statusOptions.map((option) => (
          <ListBoxItem
            key={option.key}
            id={option.key}
            className={`group py-2 hover:bg-accent-6 hover:text-accent-1 ${currentFilter === option.key ? "bg-accent-6 text-accent-1" : ""}`}
          >
            <div className="flex items-center gap-2 w-full ">
              {option.icon && option.icon()}
              <span className="flex-1">{option.label}</span>
              <Chip
                size="sm"
                variant="soft"
                className={`group-hover:bg-accent-4 group-hover:text-accent-1 ${currentFilter === option.key ? "bg-accent-4 text-accent-1" : ""}`}
              >
                {counts[option.key]}
              </Chip>
            </div>
          </ListBoxItem>
        ))}
      </ListBox>
    </div>
  );
}
