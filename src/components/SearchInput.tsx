import { Input } from '@heroui/react';

interface SearchInputProps {
  value: string;
  onChange: (value: string) => void;
}

export function SearchInput({ value, onChange }: SearchInputProps) {
  return (
    <Input
      className="flex-1 max-w-md"
      placeholder="搜索单词..."
      value={value}
      onChange={(e) => onChange(e.target.value)}
    />
  );
}
