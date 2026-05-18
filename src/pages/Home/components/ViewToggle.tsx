import { Button, ButtonGroup } from '@heroui/react';
import { useTranslation } from 'react-i18next';

export type ViewMode = 'grid' | 'list';

interface ViewToggleProps {
  mode: ViewMode;
  onModeChange: (mode: ViewMode) => void;
}

export default function ViewToggle({ mode, onModeChange }: ViewToggleProps) {
  const { t } = useTranslation();
  return (
    <ButtonGroup>
      <Button
        size="sm"
        variant={mode === 'grid' ? 'primary' : 'ghost'}
        onPress={() => onModeChange('grid')}
      >
        {t("home.viewGrid")}
      </Button>
      <Button
        size="sm"
        variant={mode === 'list' ? 'primary' : 'ghost'}
        onPress={() => onModeChange('list')}
      >
        {t("home.viewList")}
      </Button>
    </ButtonGroup>
  );
}
