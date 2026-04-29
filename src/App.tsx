import { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { readText } from '@tauri-apps/plugin-clipboard-manager';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '@heroui/react';
import type { Word } from './components/WordList';
import { WordList } from './components/WordList';
import { WordDetail } from './components/WordDetail';

function App() {
  const [selectedWord, setSelectedWord] = useState<Word | null>(null);
  const [isDark, setIsDark] = useState(() =>
    typeof window !== 'undefined'
      ? window.matchMedia('(prefers-color-scheme: dark)').matches
      : false
  );
  const lastClipboardRef = useRef<string>('');

  useEffect(() => {
    document.documentElement.classList.toggle('dark', isDark);
  }, [isDark]);

  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = (e: MediaQueryListEvent) => setIsDark(e.matches);
    mediaQuery.addEventListener('change', handler);
    return () => mediaQuery.removeEventListener('change', handler);
  }, []);

  // Poll clipboard for auto-detection - opens separate float window
  useEffect(() => {
    const pollClipboard = async () => {
      try {
        const text = await readText();
        if (text && text.trim().length > 0 && text !== lastClipboardRef.current) {
          lastClipboardRef.current = text;
          try {
            await invoke('open_float_window', { text });
          } catch (err) {
            console.error('Failed to open float window:', err);
          }
        }
      } catch (err) {
        console.error('Clipboard read error:', err);
      }
    };

    const interval = setInterval(pollClipboard, 800);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const unlisten = listen<string>('trigger-translation', (event) => {
      const text = event.payload;
      if (text && text.trim().length > 0) {
        invoke('open_float_window', { text });
        lastClipboardRef.current = text;
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleSelectWord = (word: Word) => {
    setSelectedWord(word);
  };

  const handleBack = () => {
    setSelectedWord(null);
  };

  const handleDeleted = () => {
    setSelectedWord(null);
  };

  return (
    <div className="min-h-screen bg-background">
      <header className="flex items-center justify-between px-6 py-4 border-b border-divider">
        <h1 className="text-xl font-bold">布谷鸟 - 生词本</h1>
        <Button
          variant="ghost"
          size="sm"
          onPress={() => setIsDark(d => !d)}
        >
          {isDark ? '☀️' : '🌙'}
        </Button>
      </header>
      <main className="p-6">
        {selectedWord ? (
          <WordDetail word={selectedWord} onBack={handleBack} onDeleted={handleDeleted} />
        ) : (
          <WordList onSelectWord={handleSelectWord} selectedWordId={null} />
        )}
      </main>
    </div>
  );
}

export default App;