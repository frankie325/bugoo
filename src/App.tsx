import { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { readText } from '@tauri-apps/plugin-clipboard-manager';
import { invoke } from '@tauri-apps/api/core';
import type { Word } from './components/WordList';
import { WordList } from './components/WordList';
import { WordDetail } from './components/WordDetail';

function App() {
  const [selectedWord, setSelectedWord] = useState<Word | null>(null);
  const [theme, setTheme] = useState<'light' | 'dark'>(() => {
    if (typeof window !== 'undefined') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    return 'light';
  });
  const lastClipboardRef = useRef<string>('');

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
  }, [theme]);

  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = (e: MediaQueryListEvent) => setTheme(e.matches ? 'dark' : 'light');
    mediaQuery.addEventListener('change', handler);
    return () => mediaQuery.removeEventListener('change', handler);
  }, []);

  // Poll clipboard for auto-detection - opens separate float window
  useEffect(() => {
    const pollClipboard = async () => {
      try {
        const text = await readText();
        console.log('[DEBUG] Clipboard read:', text);
        if (text && text.trim().length > 0 && text !== lastClipboardRef.current) {
          console.log('[DEBUG] New text detected, calling open_float_window:', text);
          lastClipboardRef.current = text;
          try {
            await invoke('open_float_window', { text });
            console.log('[DEBUG] open_float_window succeeded');
          } catch (err) {
            console.error('[DEBUG] Failed to open float window:', err);
          }
        }
      } catch (err) {
        console.error('[DEBUG] Clipboard read error:', err);
      }
    };

    // Poll every 800ms
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
    <div className="app">
      <header className="app-header">
        <h1>布谷鸟 - 生词本</h1>
        <button
          className="theme-toggle"
          onClick={() => setTheme(t => t === 'light' ? 'dark' : 'light')}
          aria-label="Toggle theme"
        >
          {theme === 'light' ? '🌙' : '☀️'}
        </button>
      </header>
      <main className="app-main">
        <button onClick={async () => {
          try {
            console.log('[TEST] Calling open_float_window with "test"');
            await invoke('open_float_window', { text: 'test' });
            console.log('[TEST] open_float_window returned');
          } catch (e) {
            console.error('[TEST] Error:', e);
          }
        }}>Test Float Window</button>
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