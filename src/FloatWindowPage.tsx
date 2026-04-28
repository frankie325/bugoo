import { useState, useEffect } from 'react';
import { translate } from './lib/api';
import './styles/globals.css';

interface Props {
  text: string;
}

export function FloatWindowPage({ text }: Props) {
  const [translation, setTranslation] = useState<string>('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  useEffect(() => {
    if (!text) return;
    setLoading(true);
    setError(false);
    translate(text, 'EN', 'ZH')
      .then((r) => {
        setTranslation(r.translation);
        setError(false);
      })
      .catch((err) => {
        console.error('Translation failed:', err);
        setTranslation('翻译失败');
        setError(true);
      })
      .finally(() => setLoading(false));
  }, [text]);

  const canAdd = !loading && !error && translation.length > 0 && translation !== '翻译失败';

  const handleAdd = async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    try {
      await invoke('add_word', { word: text, translation });
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      getCurrentWindow().close();
    } catch (err) {
      console.error('Failed to add word:', err);
    }
  };

  const handleClose = async () => {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    getCurrentWindow().close();
  };

  return (
    <div className="float-window">
      <div className="float-word">{text}</div>
      <div className="float-translation">
        {loading ? '翻译中...' : translation}
      </div>
      <div className="float-actions">
        <button onClick={handleAdd} disabled={!canAdd}>
          + 添加到生词本
        </button>
        <button onClick={handleClose}>关闭</button>
      </div>
    </div>
  );
}
