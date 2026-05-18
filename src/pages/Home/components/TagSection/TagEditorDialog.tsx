import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Button,
  ColorArea,
  ColorPicker,
  ColorSlider,
  ColorSwatch,
  ColorSwatchPicker,
  Input,
  Modal,
  Separator,
  ScrollShadow,
  Surface,
} from "@heroui/react";
import { Tag as TagIcon } from "lucide-react";
import type { TagItem } from "@src/types/tag";
import type { TagCreatePlacement } from "@src/lib/tagSort";

export const TAG_COLORS = [
  "#ef4444",
  "#f97316",
  "#eab308",
  "#22c55e",
  "#06b6d4",
  "#3b82f6",
  "#8b5cf6",
  "#ec4899",
];

export type EditorState =
  | {
      mode: "create";
      anchorTagId?: string;
      placement: TagCreatePlacement;
    }
  | {
      mode: "edit";
      tag: TagItem;
    };

interface TagEditorDialogProps {
  state: EditorState | null;
  onClose: () => void;
  onSubmit: (name: string, color: string) => Promise<void>;
}

export function TagEditorDialog({
  state,
  onClose,
  onSubmit,
}: TagEditorDialogProps) {
  const { t } = useTranslation();
  const [name, setName] = useState("");
  const [color, setColor] = useState(TAG_COLORS[0]);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const isOpen = !!state;

  useEffect(() => {
    if (!state) return;

    if (state.mode === "edit") {
      setName(state.tag.name);
      setColor(state.tag.color);
      return;
    }

    setName("");
    setColor(TAG_COLORS[0]);
  }, [state]);

  const handleSubmit = async () => {
    const trimmedName = name.trim();
    if (!trimmedName) return;

    setIsSubmitting(true);
    try {
      await onSubmit(trimmedName, color);
      onClose();
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Modal.Backdrop isOpen={isOpen} onOpenChange={(open) => !open && onClose()}>
      <Modal.Container>
        <Modal.Dialog className="sm:max-w-[380px]">
          <Modal.CloseTrigger />
          <Modal.Header>
            <Modal.Icon className="bg-default text-foreground">
              <TagIcon className="size-5" />
            </Modal.Icon>
          </Modal.Header>
          <Modal.Body className="">
            <Surface className="p-3 flex flex-col gap-3">
              <div className="flex gap-4">
                <ColorPicker
                  value={color}
                  onChange={(nextColor) => setColor(nextColor.toString("hex"))}
                >
                  <ColorPicker.Trigger>
                    <ColorSwatch size="sm" />
                  </ColorPicker.Trigger>
                  <ColorPicker.Popover>
                    <ColorArea
                      aria-label="Color area"
                      className="max-w-full"
                      colorSpace="hsb"
                      xChannel="saturation"
                      yChannel="brightness"
                    >
                      <ColorArea.Thumb />
                    </ColorArea>
                    <ColorSlider
                      aria-label="Hue slider"
                      channel="hue"
                      className="gap-1 px-1"
                      colorSpace="hsb"
                    >
                      <ColorSlider.Track>
                        <ColorSlider.Thumb />
                      </ColorSlider.Track>
                    </ColorSlider>
                  </ColorPicker.Popover>
                </ColorPicker>
                <Input
                  autoFocus
                  className="flex-1"
                  value={name}
                  variant="secondary"
                  placeholder={t("home.tagNamePlaceholder")}
                  disabled={isSubmitting}
                  onChange={(event) => setName(event.target.value)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter") {
                      event.preventDefault();
                      void handleSubmit();
                    }
                  }}
                />
              </div>

              <Separator />

              <div className="flex flex-col gap-2">
                <span className="text-xs text-foreground">
                  {t("home.tagColorLabel")}
                </span>
                <ScrollShadow hideScrollBar orientation="horizontal">
                  <ColorSwatchPicker
                    className="flex-nowrap"
                    value={color}
                    onChange={(nextColor) =>
                      setColor(nextColor.toString("hex"))
                    }
                  >
                    {TAG_COLORS.map((tagColor) => (
                      <ColorSwatchPicker.Item
                        className="flex-shrink-0"
                        key={tagColor}
                        color={tagColor}
                      >
                        <ColorSwatchPicker.Swatch />
                        <ColorSwatchPicker.Indicator />
                      </ColorSwatchPicker.Item>
                    ))}
                  </ColorSwatchPicker>
                </ScrollShadow>
              </div>
            </Surface>
          </Modal.Body>
          <Modal.Footer>
            <Button
              variant="tertiary"
              onPress={onClose}
              isDisabled={isSubmitting}
            >
              {t("home.deleteTagCancel")}
            </Button>
            <Button
              onPress={handleSubmit}
              isPending={isSubmitting}
              isDisabled={!name.trim()}
            >
              {t("home.tagSave")}
            </Button>
          </Modal.Footer>
        </Modal.Dialog>
      </Modal.Container>
    </Modal.Backdrop>
  );
}