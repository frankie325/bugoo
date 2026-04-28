import { useState, useEffect } from 'react';
import { Button, Spinner, Card, CardBody } from '@heroui/react';
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
    <div className="min-h-screen bg-background p-4">
      <Card>
        <CardBody className="space-y-4">
          <div>
            <p className="text-sm text-default-400">原文</p>
            <p className="text-xl font-bold">{text}</p>
          </div>
          <div>
            <p className="text-sm text-default-400">翻译</p>
            {loading ? (
              <div className="flex justify-center py-4"><Spinner /></div>
            ) : (
              <p className={error ? 'text-danger' : ''}>{translation}</p>
            )}
          </div>
          <div className="flex gap-2">
            <Button color="primary" isDisabled={!canAdd} onPress={handleAdd} className="flex-1">
              + 添加到生词本
            </Button>
            <Button variant="flat" onPress={handleClose}>
              关闭
            </Button>
          </div>
        </CardBody>
      </Card>
    </div>
  );
}
