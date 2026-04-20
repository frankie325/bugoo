import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import type { Word } from './components/WordList';
import { WordList } from './components/WordList';
import { WordDetail } from './components/WordDetail';
import { FloatWindow } from './components/FloatWindow';
import { addWord } from './lib/api';

function App() {
  const [selectedWord, setSelectedWord] = useState<Word | null>(null);
  const [floatWindowText, setFloatWindowText] = useState<string | null>(null);

  useEffect(() => {
    const unlisten = listen<string>('trigger-translation', (event) => {
      const text = event.payload;
      if (text && text.trim().length > 0) {
        setFloatWindowText(text);
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

  const handleAddToWordList = async (word: string, translation: string) => {
    try {
      await addWord(word, translation);
      setFloatWindowText(null);
    } catch (err) {
      console.error('Failed to add word:', err);
    }
  };

  const handleFloatClose = () => {
    setFloatWindowText(null);
  };

  return (
    <>
      {floatWindowText && (
        <FloatWindow
          selectedText={floatWindowText}
          onAddToWordList={handleAddToWordList}
          onClose={handleFloatClose}
        />
      )}
      <div className="app">
        <header className="app-header">
          <h1>布谷鸟 - 生词本</h1>
        </header>
        <main className="app-main">
          {selectedWord ? (
            <WordDetail word={selectedWord} onBack={handleBack} onDeleted={handleDeleted} />
          ) : (
            <WordList onSelectWord={handleSelectWord} selectedWordId={null} />
          )}
        </main>
      </div>
    </>
  );
}

export default App;