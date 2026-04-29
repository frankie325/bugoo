import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button, Card, Chip, Separator, Modal, ModalDialog, ModalHeader, ModalBody, ModalFooter } from '@heroui/react';
import type { Word } from './WordList';

interface WordDetailProps {
  word: Word;
  onBack: () => void;
  onDeleted: () => void;
}

export function WordDetail({ word, onBack, onDeleted }: WordDetailProps) {
  const [isDeleting, setIsDeleting] = useState(false);
  const [isModalOpen, setIsModalOpen] = useState(false);

  const handleSpeak = async () => {
    try {
      await invoke('speak_text', { text: word.word, lang: word.source_lang || null });
    } catch (e) {
      console.error('TTS failed:', e);
    }
  };

  const handleDelete = async () => {
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
    <div className="space-y-4">
      <Button variant="ghost" onPress={onBack}>← 返回</Button>

      <Card>
        <div className="p-4 space-y-4">
          <div className="flex items-start justify-between">
            <div>
              <h1 className="text-3xl font-bold">{word.word}</h1>
              <p className="text-lg text-default-500">{word.translation}</p>
            </div>
            <Button variant="outline" onPress={handleSpeak}>🔊 发音</Button>
          </div>

          <Separator />

          <div className="flex gap-2">
            <Chip color={word.status === 'learning' ? 'warning' : 'success'}>
              {word.status === 'learning' ? '复习中' : '已记住'}
            </Chip>
            {word.tags && word.tags.split(',').filter(Boolean).map((tag) => (
              <Chip key={tag} variant="soft" size="sm">{tag}</Chip>
            ))}
          </div>

          <Separator />

          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <p className="text-default-400">下次复习</p>
              <p>{formatTimestamp(word.next_review_at)}</p>
            </div>
            <div>
              <p className="text-default-400">间隔</p>
              <p>{word.interval} 天</p>
            </div>
            <div>
              <p className="text-default-400">难度系数</p>
              <p>{word.ease_factor.toFixed(2)}</p>
            </div>
            <div>
              <p className="text-default-400">复习次数</p>
              <p>{word.repetitions}</p>
            </div>
          </div>

          {word.notes && (
            <>
              <Separator />
              <div>
                <p className="text-sm font-medium">笔记</p>
                <p className="text-default-500">{word.notes}</p>
              </div>
            </>
          )}

          <Separator />

          <p className="text-xs text-default-400">
            创建于 {formatTimestamp(word.created_at)}
          </p>
        </div>
      </Card>

      <Button variant="danger" onPress={() => setIsModalOpen(true)}>
        删除单词
      </Button>

      <Modal isOpen={isModalOpen} onOpenChange={setIsModalOpen}>
        <ModalDialog>
          <ModalHeader>确认删除</ModalHeader>
          <ModalBody>
            确定要删除单词 "{word.word}" 吗？此操作无法撤销。
          </ModalBody>
          <ModalFooter>
            <Button variant="ghost" onPress={() => setIsModalOpen(false)}>取消</Button>
            <Button variant="danger" isPending={isDeleting} onPress={handleDelete}>
              删除
            </Button>
          </ModalFooter>
        </ModalDialog>
      </Modal>
    </div>
  );
}