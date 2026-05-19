use crate::ports::outbound::translation::{
    TranslationError, TranslationFuture, TranslationProvider, TranslationRequest,
};

pub struct GoogleTranslationProvider;

impl TranslationProvider for GoogleTranslationProvider {
    fn translate<'a>(&'a self, _request: TranslationRequest) -> TranslationFuture<'a> {
        Box::pin(async { Err(TranslationError::UnsupportedEngine("google".to_string())) })
    }
}
