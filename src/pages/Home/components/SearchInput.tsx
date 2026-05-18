import { Input } from '@heroui/react';
import { useTranslation } from 'react-i18next';

interface SearchInputProps {
  value: string;
  onChange: (value: string) => void;
}

export default function SearchInput({ value, onChange }: SearchInputProps) {
  const { t } = useTranslation();
  return (
    <Input
      className="flex-1 max-w-md"
      placeholder={t("home.searchPlaceholder")}
      value={value}
      onChange={(e) => onChange(e.target.value)}
    />
  );
}
