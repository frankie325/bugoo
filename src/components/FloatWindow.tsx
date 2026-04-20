import { useState, useEffect } from 'react';
import { translate } from '../lib/api';

interface Props {
  selectedText: string;
  onAddToWordList: (word: string, translation: string) => void;
  onClose: () => void;
}

export function FloatWindow({ selectedText, onAddToWordList, onClose }: Props) {
  const [translation, setTranslation] = useState<string>('');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!selectedText) return;
    setLoading(true);
    translate(selectedText, 'EN', 'ZH')
      .then((r) => setTranslation(r.translation))
      .catch(() => setTranslation('翻译失败'))
      .finally(() => setLoading(false));
  }, [selectedText]);

  return (
    <div className="float-window">
      <div className="float-word">{selectedText}</div>
      <div className="float-translation">
        {loading ? '翻译中...' : translation}
      </div>
      <div className="float-actions">
        <button onClick={() => onAddToWordList(selectedText, translation)}>
          + 添加到生词本
        </button>
        <button onClick={onClose}>关闭</button>
      </div>
    </div>
  );
}
