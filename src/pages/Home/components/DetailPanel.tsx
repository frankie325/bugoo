import { Button } from '@heroui/react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  generateWordDetail,
  getWordDetail,
  type Word,
  type WordDetail,
} from '../../../lib/api';

interface DetailPanelProps {
  word: Word;
  onClose: () => void;
}

export default function DetailPanel({ word, onClose }: DetailPanelProps) {
  const { t } = useTranslation();
  const [detail, setDetail] = useState<WordDetail | null>(null);
  const [isLoadingDetail, setIsLoadingDetail] = useState(false);
  const [isGeneratingDetail, setIsGeneratingDetail] = useState(false);
  const [detailError, setDetailError] = useState<string | null>(null);
  const detailRequestIdRef = useRef(0);

  const toReadableError = useCallback((error: unknown) => {
    if (error instanceof Error) {
      return error.message;
    }

    if (typeof error === 'string') {
      return error;
    }

    return t("home.detail.detailLoadFailed");
  }, [t]);

  const loadDetail = useCallback(async () => {
    const requestId = detailRequestIdRef.current + 1;
    detailRequestIdRef.current = requestId;

    setIsLoadingDetail(true);
    setDetail(null);
    setDetailError(null);

    try {
      const result = await getWordDetail(word.id);

      if (detailRequestIdRef.current === requestId) {
        setDetail(result);
      }
    } catch (error) {
      if (detailRequestIdRef.current === requestId) {
        setDetail(null);
        setDetailError(toReadableError(error));
      }
    } finally {
      if (detailRequestIdRef.current === requestId) {
        setIsLoadingDetail(false);
      }
    }
  }, [toReadableError, word.id]);

  useEffect(() => {
    loadDetail();

    return () => {
      detailRequestIdRef.current += 1;
    };
  }, [loadDetail]);

  const handleGenerateDetail = async () => {
    const requestId = detailRequestIdRef.current + 1;
    detailRequestIdRef.current = requestId;

    setIsLoadingDetail(false);
    setIsGeneratingDetail(true);
    setDetailError(null);

    try {
      const result = await generateWordDetail(word.id);

      if (detailRequestIdRef.current === requestId) {
        setDetail(result);
      }
    } catch (error) {
      if (detailRequestIdRef.current === requestId) {
        setDetailError(toReadableError(error));
      }
    } finally {
      if (detailRequestIdRef.current === requestId) {
        setIsGeneratingDetail(false);
      }
    }
  };

  const renderEmptyText = () => (
    <p className="text-sm text-foreground-400">{t("home.detail.noDetail")}</p>
  );

  return (
    <aside className="w-80 border-l border-divider bg-background p-4 flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-medium">{t("home.detail.title")}</h2>
        <Button size="sm" variant="ghost" onPress={onClose}>
          {t("home.detail.close")}
        </Button>
      </div>

      <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto pr-1">
        <div className="flex flex-col gap-2">
          <h3 className="text-2xl font-bold">{word.word}</h3>
          {word.phonetic && (
            <p className="text-sm text-foreground-400">{word.phonetic}</p>
          )}
          <p className="text-foreground-600">{word.translation}</p>
        </div>

        {word.tags && (
          <div className="flex flex-wrap gap-1">
            {word.tags.split(',').map((tag, i) => (
              <span
                key={i}
                className="text-xs px-2 py-1 bg-foreground-100 rounded"
              >
                {tag.trim()}
              </span>
            ))}
          </div>
        )}

        <div className="flex flex-col gap-3 border-t border-divider pt-4">
          {isLoadingDetail ? (
            <p className="text-sm text-foreground-400">{t("app.loading")}</p>
          ) : (
            <>
              {detailError && (
                <div className="flex flex-col gap-3 rounded border border-danger-200 bg-danger-50 p-3 text-sm text-danger">
                  <div>
                    <p className="font-medium">{t("home.detail.detailLoadFailed")}</p>
                    <p className="mt-1 break-words">{detailError}</p>
                  </div>
                  {!detail && (
                    <Button
                      size="sm"
                      variant="ghost"
                      onPress={loadDetail}
                      isDisabled={isGeneratingDetail}
                    >
                      {t("home.detail.retryLoadDetail")}
                    </Button>
                  )}
                </div>
              )}

              {detail ? (
                <>
                  <section className="flex flex-col gap-2">
                    <h4 className="text-sm font-medium">{t("home.detail.definitions")}</h4>
                    {detail.definitions.length > 0 ? (
                      <ul className="flex list-disc flex-col gap-1 pl-5 text-sm text-foreground-600">
                        {detail.definitions.map((definition, index) => (
                          <li key={`${definition}-${index}`}>{definition}</li>
                        ))}
                      </ul>
                    ) : (
                      renderEmptyText()
                    )}
                  </section>

                  <section className="flex flex-col gap-2">
                    <h4 className="text-sm font-medium">{t("home.detail.examples")}</h4>
                    {detail.examples.length > 0 ? (
                      <div className="flex flex-col gap-2">
                        {detail.examples.map((example, index) => (
                          <div
                            key={`${example.sentence}-${index}`}
                            className="rounded bg-foreground-50 p-3 text-sm"
                          >
                            <p className="text-foreground-700">{example.sentence}</p>
                            {example.translation && (
                              <p className="mt-1 text-foreground-400">
                                {example.translation}
                              </p>
                            )}
                          </div>
                        ))}
                      </div>
                    ) : (
                      renderEmptyText()
                    )}
                  </section>

                  <section className="flex flex-col gap-2">
                    <h4 className="text-sm font-medium">{t("home.detail.memoryTip")}</h4>
                    {detail.memoryTip ? (
                      <p className="text-sm text-foreground-600">{detail.memoryTip}</p>
                    ) : (
                      renderEmptyText()
                    )}
                  </section>

                  <section className="flex flex-col gap-2">
                    <h4 className="text-sm font-medium">{t("home.detail.wordDetail")}</h4>
                    {detail.detail ? (
                      <p className="whitespace-pre-line text-sm text-foreground-600">
                        {detail.detail}
                      </p>
                    ) : (
                      renderEmptyText()
                    )}
                  </section>
                </>
              ) : (
                <div className="flex flex-col gap-2">
                  {!detailError && renderEmptyText()}
                  <Button
                    isPending={isGeneratingDetail}
                    onPress={handleGenerateDetail}
                  >
                    {isGeneratingDetail
                      ? t("home.detail.generatingDetail")
                      : t("home.detail.generateDetail")}
                  </Button>
                </div>
              )}
            </>
          )}
        </div>

        <div className="flex flex-col gap-2">
          <span className="text-sm font-medium">{t("home.detail.memoryProgress")}</span>
          <div className="flex justify-between text-sm">
            <span>{t("home.detail.memoryStrength")}</span>
            <span>{Math.round(word.ease_factor * 100)}%</span>
          </div>
          <div className="flex justify-between text-sm">
            <span>{t("home.detail.interval")}</span>
            <span>{t("home.detail.days", { days: word.interval })}</span>
          </div>
          <div className="flex justify-between text-sm">
            <span>{t("home.detail.reviewCount")}</span>
            <span>{word.repetitions}</span>
          </div>
        </div>
      </div>

      <div className="mt-auto flex flex-col gap-2">
        <Button variant="ghost" className="text-success">
          {t("home.detail.remember")}
        </Button>
        <Button variant="ghost" className="text-warning">
          {t("home.detail.fuzzy")}
        </Button>
        <Button variant="ghost" className="text-danger">
          {t("home.detail.forgot")}
        </Button>
      </div>
    </aside>
  );
}
