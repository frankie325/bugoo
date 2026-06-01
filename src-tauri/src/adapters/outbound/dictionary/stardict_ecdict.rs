use crate::ports::outbound::dictionary::{
    normalize_dictionary_text, DictionaryError, DictionaryLookupRequest, DictionaryLookupResult,
    DictionaryProvider,
};
use crate::ports::outbound::translation::TranslationExample;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;

pub struct StarDictEcdictDictionaryProvider {
    ifo_path: PathBuf,
    idx_path: PathBuf,
    dict_path: PathBuf,
}

#[derive(Debug)]
struct StarDictIndexEntry {
    word: String,
    offset: u32,
    size: u32,
}

impl StarDictEcdictDictionaryProvider {
    pub fn new(resource_dir: PathBuf, file_stem: &str) -> Result<Self, DictionaryError> {
        let ifo_path = resource_dir.join(format!("{file_stem}.ifo"));
        let idx_path = resource_dir.join(format!("{file_stem}.idx"));
        let dict_path = resource_dir.join(format!("{file_stem}.dict"));

        for path in [&ifo_path, &idx_path, &dict_path] {
            if !path.exists() {
                return Err(DictionaryError::ResourceMissing(path.display().to_string()));
            }
        }

        Ok(Self {
            ifo_path,
            idx_path,
            dict_path,
        })
    }

    fn lookup_entry(&self, word: &str) -> Result<Option<StarDictIndexEntry>, DictionaryError> {
        let file = File::open(&self.idx_path)
            .map_err(|error| DictionaryError::QueryFailed(error.to_string()))?;
        let mut reader = BufReader::new(file);

        loop {
            let Some(entry) = read_index_entry(&mut reader)? else {
                return Ok(None);
            };

            if entry.word == word {
                return Ok(Some(entry));
            }
        }
    }

    fn read_definition(&self, entry: &StarDictIndexEntry) -> Result<String, DictionaryError> {
        let mut file = File::open(&self.dict_path)
            .map_err(|error| DictionaryError::QueryFailed(error.to_string()))?;
        file.seek(SeekFrom::Start(entry.offset as u64))
            .map_err(|error| DictionaryError::QueryFailed(error.to_string()))?;

        let mut buffer = vec![0_u8; entry.size as usize];
        file.read_exact(&mut buffer)
            .map_err(|error| DictionaryError::QueryFailed(error.to_string()))?;

        String::from_utf8(buffer).map_err(|error| DictionaryError::QueryFailed(error.to_string()))
    }

    fn validate_ifo(&self) -> Result<(), DictionaryError> {
        let file = File::open(&self.ifo_path)
            .map_err(|error| DictionaryError::QueryFailed(error.to_string()))?;
        let mut reader = BufReader::new(file);
        let mut first_line = String::new();
        reader
            .read_line(&mut first_line)
            .map_err(|error| DictionaryError::QueryFailed(error.to_string()))?;

        if !first_line.trim().starts_with("StarDict") {
            return Err(DictionaryError::ResourceMissing(
                self.ifo_path.display().to_string(),
            ));
        }

        Ok(())
    }
}

impl DictionaryProvider for StarDictEcdictDictionaryProvider {
    fn lookup(
        &self,
        request: DictionaryLookupRequest,
    ) -> Result<Option<DictionaryLookupResult>, DictionaryError> {
        let word = normalize_dictionary_text(&request.text);
        if word.is_empty() {
            return Err(DictionaryError::EmptyText);
        }

        self.validate_ifo()?;

        let Some(entry) = self.lookup_entry(&word)? else {
            return Ok(None);
        };

        let raw_definition = self.read_definition(&entry)?;
        let parsed = parse_ecdict_definition(&raw_definition);

        Ok(Some(DictionaryLookupResult {
            word: entry.word,
            translation: parsed.translation,
            phonetic: parsed.phonetic,
            part_of_speech: parsed.part_of_speech,
            definitions: parsed.definitions,
            examples: parsed.examples,
        }))
    }

    fn supports_language_pair(&self, source_lang: &str, target_lang: &str) -> bool {
        let src = source_lang.trim().to_lowercase();
        let tgt = target_lang.trim().to_lowercase();
        is_ecdict_supported_source(&src) && is_ecdict_supported_target(&tgt)
    }
}

fn is_ecdict_supported_source(lang: &str) -> bool {
    matches!(lang, "en")
}

fn is_ecdict_supported_target(lang: &str) -> bool {
    matches!(
        lang,
        "zh" | "zh-cn" | "zh-hans" | "zt" | "zh-tw" | "zh-hant"
    )
}

fn read_index_entry<R: Read>(
    reader: &mut R,
) -> Result<Option<StarDictIndexEntry>, DictionaryError> {
    let mut word_bytes = Vec::new();
    let mut byte = [0_u8; 1];

    loop {
        match reader.read_exact(&mut byte) {
            Ok(()) => {
                if byte[0] == 0 {
                    break;
                }
                word_bytes.push(byte[0]);
            }
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
                if word_bytes.is_empty() {
                    return Ok(None);
                }
                return Err(DictionaryError::QueryFailed(error.to_string()));
            }
            Err(error) => return Err(DictionaryError::QueryFailed(error.to_string())),
        }
    }

    let mut offset_bytes = [0_u8; 4];
    let mut size_bytes = [0_u8; 4];
    reader
        .read_exact(&mut offset_bytes)
        .map_err(|error| DictionaryError::QueryFailed(error.to_string()))?;
    reader
        .read_exact(&mut size_bytes)
        .map_err(|error| DictionaryError::QueryFailed(error.to_string()))?;

    let word = String::from_utf8(word_bytes)
        .map_err(|error| DictionaryError::QueryFailed(error.to_string()))?;

    Ok(Some(StarDictIndexEntry {
        word: normalize_dictionary_text(&word),
        offset: u32::from_be_bytes(offset_bytes),
        size: u32::from_be_bytes(size_bytes),
    }))
}

#[derive(Debug, Default)]
struct ParsedEcdictDefinition {
    translation: String,
    phonetic: Option<String>,
    part_of_speech: Vec<String>,
    definitions: Vec<String>,
    examples: Vec<TranslationExample>,
}

fn parse_ecdict_definition(raw: &str) -> ParsedEcdictDefinition {
    let mut parsed = ParsedEcdictDefinition::default();
    let lines = raw
        .lines()
        .flat_map(|line| line.split('\n'))
        .map(|line| line.trim().trim_start_matches(';').trim())
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    for line in lines {
        if parsed.phonetic.is_none() && line.starts_with('[') && line.ends_with(']') {
            parsed.phonetic = Some(
                line.trim_start_matches('[')
                    .trim_end_matches(']')
                    .to_string(),
            );
            continue;
        }

        if let Some((left, right)) = line.split_once('.') {
            let pos = left.trim();
            let definition = right.trim();
            if !pos.is_empty()
                && !definition.is_empty()
                && pos.chars().all(|char| char.is_ascii_alphabetic())
            {
                if !parsed.part_of_speech.iter().any(|value| value == pos) {
                    parsed.part_of_speech.push(pos.to_string());
                }
                parsed.definitions.push(line.clone());
                continue;
            }
        }

        parsed.definitions.push(line);
    }

    parsed.translation = parsed.definitions.join("\n");
    parsed
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_dictionary() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("bugoo-stardict-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();

        let stem = "stardict-ecdict-2.4.2";
        fs::write(
            dir.join(format!("{stem}.ifo")),
            "StarDict's dict ifo file\nversion=2.4.2\nwordcount=1\nidxfilesize=14\nbookname=ECDICT\n",
        )
        .unwrap();

        let definition = "[həˈləʊ]\nint. 你好\nn. 问候";
        fs::write(dir.join(format!("{stem}.dict")), definition.as_bytes()).unwrap();

        let mut idx = Vec::new();
        idx.extend_from_slice(b"hello");
        idx.push(0);
        idx.extend_from_slice(&0_u32.to_be_bytes());
        idx.extend_from_slice(&(definition.len() as u32).to_be_bytes());
        fs::write(dir.join(format!("{stem}.idx")), idx).unwrap();

        dir
    }

    #[test]
    fn lookup_returns_dictionary_result_when_word_exists() {
        let dir = create_test_dictionary();
        let provider = StarDictEcdictDictionaryProvider::new(dir, "stardict-ecdict-2.4.2").unwrap();

        let result = provider
            .lookup(DictionaryLookupRequest {
                text: "Hello".to_string(),
                source_lang: "en".to_string(),
                target_lang: "zh-CN".to_string(),
            })
            .unwrap()
            .unwrap();

        assert_eq!(result.word, "hello");
        assert_eq!(result.translation, "int. 你好\nn. 问候");
        assert_eq!(result.phonetic, Some("həˈləʊ".to_string()));
        assert_eq!(result.part_of_speech, vec!["int", "n"]);
        assert_eq!(result.definitions, vec!["int. 你好", "n. 问候"]);
        assert!(result.examples.is_empty());
    }

    #[test]
    fn lookup_returns_none_when_word_is_missing() {
        let dir = create_test_dictionary();
        let provider = StarDictEcdictDictionaryProvider::new(dir, "stardict-ecdict-2.4.2").unwrap();

        let result = provider
            .lookup(DictionaryLookupRequest {
                text: "missing".to_string(),
                source_lang: "en".to_string(),
                target_lang: "zh-CN".to_string(),
            })
            .unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn new_rejects_missing_dictionary_files() {
        let dir = std::env::temp_dir().join(format!("missing-{}", uuid::Uuid::new_v4()));

        let result = StarDictEcdictDictionaryProvider::new(dir, "stardict-ecdict-2.4.2");

        assert!(matches!(result, Err(DictionaryError::ResourceMissing(_))));
    }

    #[test]
    fn parse_definition_extracts_phonetic_and_pos() {
        let parsed = parse_ecdict_definition("[test]\nn. 测试\nv. 测验");

        assert_eq!(parsed.phonetic, Some("test".to_string()));
        assert_eq!(parsed.part_of_speech, vec!["n", "v"]);
        assert_eq!(parsed.definitions, vec!["n. 测试", "v. 测验"]);
    }

    #[test]
    fn supports_language_pair_accepts_english_source() {
        let dir = create_test_dictionary();
        let provider = StarDictEcdictDictionaryProvider::new(dir, "stardict-ecdict-2.4.2").unwrap();

        assert!(provider.supports_language_pair("en", "zh"));
        assert!(provider.supports_language_pair("EN", "ZH-CN"));
        assert!(provider.supports_language_pair("en", "zh-TW"));
    }

    #[test]
    fn supports_language_pair_rejects_unsupported_languages() {
        let dir = create_test_dictionary();
        let provider = StarDictEcdictDictionaryProvider::new(dir, "stardict-ecdict-2.4.2").unwrap();

        assert!(!provider.supports_language_pair("ja", "zh"));
        assert!(!provider.supports_language_pair("en", "ja"));
    }
}
