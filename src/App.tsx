import { useState } from 'react';
import type { Word } from './components/WordList';
import { WordList } from './components/WordList';
import { WordDetail } from './components/WordDetail';

function App() {
  const [selectedWord, setSelectedWord] = useState<Word | null>(null);

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
      </header>
      <main className="app-main">
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