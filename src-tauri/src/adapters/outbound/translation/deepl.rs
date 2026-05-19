use crate::ports::outbound::translation::{
    TranslationError, TranslationFuture, TranslationProvider, TranslationRequest,
};

pub struct DeepLTranslationProvider;

impl TranslationProvider for DeepLTranslationProvider {
    fn translate<'a>(&'a self, _request: TranslationRequest) -> TranslationFuture<'a> {
        Box::pin(async { Err(TranslationError::UnsupportedEngine("deepl".to_string())) })
    }
}
