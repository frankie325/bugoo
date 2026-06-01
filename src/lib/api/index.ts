export {
  translate,
  type TranslationResult,
  type TranslationExample,
} from "./translate";
export {
  getWordDetail,
  generateWordDetail,
  type WordDetail,
} from "./wordDetails";
export {
  addWord,
  getWords,
  deleteWord,
  updateWord,
  type Word,
  type WordUpdate,
} from "./word";
export { getSettings, setSetting, seedSettings } from "./settings";
export {
  getTags,
  createTag,
  updateTag,
  deleteTag,
  reorderTags,
} from "./tags";
export {
  speakText,
  stopSpeech,
  listVoices,
  setVoice,
  type VoiceInfo,
} from "./tts";
export {
  getTranslationLanguages,
  type TranslationLanguage,
  type TranslationLanguages,
  type SourceToTargetMapping,
} from "./translationLanguages";
