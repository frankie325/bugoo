import { Button, ButtonGroup } from '@heroui/react';

type ViewMode = 'grid' | 'list';

interface ViewToggleProps {
  mode: ViewMode;
  onModeChange: (mode: ViewMode) => void;
}

export function ViewToggle({ mode, onModeChange }: ViewToggleProps) {
  return (
    <ButtonGroup>
      <Button
        size="sm"
        variant={mode === 'grid' ? 'primary' : 'ghost'}
        onPress={() => onModeChange('grid')}
      >
        Grid
      </Button>
      <Button
        size="sm"
        variant={mode === 'list' ? 'primary' : 'ghost'}
        onPress={() => onModeChange('list')}
      >
        List
      </Button>
    </ButtonGroup>
  );
}
