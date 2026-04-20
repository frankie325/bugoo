import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Word } from './WordList';

interface WordDetailProps {
  word: Word;
  onBack: () => void;
  onDeleted: () => void;
}

export function WordDetail({ word, onBack, onDeleted }: WordDetailProps) {
  const [isDeleting, setIsDeleting] = useState(false);

  const handleSpeak = async () => {
    try {
      await invoke('speak_text', { text: word.word, lang: word.source_lang || null });
    } catch (e) {
      console.error('TTS failed:', e);
    }
  };

  const handleDelete = async () => {
    if (!confirm(`确定要删除单词 "${word.word}" 吗？`)) {
      return;
    }

    setIsDeleting(true);
    try {
      await invoke('delete_word', { wordId: word.id });
      onDeleted();
    } catch (e) {
      console.error('Failed to delete word:', e);
      setIsDeleting(false);
    }
  };

  const formatTimestamp = (timestamp: number): string => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className="word-detail">
      <button className="back-button" onClick={onBack}>
        ← 返回
      </button>

      <div className="word-header">
        <h1 className="word-title">{word.word}</h1>
        <button
          className="speak-button"
          onClick={handleSpeak}
          aria-label="播放发音"
        >
          🔊
        </button>
      </div>

      <p className="word-translation-detail">{word.translation}</p>

      <div className="word-info-section">
        <div className="info-row">
          <span className="info-label">状态</span>
          <span className={`info-value status-${word.status}`}>
            {word.status === 'learning' ? '复习中' : '已记住'}
          </span>
        </div>

        {word.tags && (
          <div className="info-row">
            <span className="info-label">标签</span>
            <span className="info-value">{word.tags || '-'}</span>
          </div>
        )}

        <div className="info-row">
          <span className="info-label">下次复习</span>
          <span className="info-value">
            {formatTimestamp(word.next_review_at)}
          </span>
        </div>

        <div className="info-row">
          <span className="info-label">间隔</span>
          <span className="info-value">{word.interval} 天</span>
        </div>

        <div className="info-row">
          <span className="info-label">难度系数</span>
          <span className="info-value">{word.ease_factor.toFixed(2)}</span>
        </div>

        <div className="info-row">
          <span className="info-label">复习次数</span>
          <span className="info-value">{word.repetitions}</span>
        </div>
      </div>

      {word.notes && (
        <div className="word-notes-section">
          <h3 className="section-title">笔记</h3>
          <p className="notes-content">{word.notes}</p>
        </div>
      )}

      <div className="word-meta-section">
        <p className="meta-text">
          创建于 {formatTimestamp(word.created_at)}
        </p>
      </div>

      <button
        className="delete-button"
        onClick={handleDelete}
        disabled={isDeleting}
      >
        {isDeleting ? '删除中...' : '删除单词'}
      </button>
    </div>
  );
}