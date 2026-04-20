#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
pub use mac::MacTts;

#[cfg(target_os = "windows")]
pub use windows::WindowsTts;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
use crate::tts::dummy::DummyTts;

pub trait TtsEngine: Send + Sync {
    fn speak(&self, text: &str, lang: &str) -> Result<(), String>;
}

pub fn new_tts() -> Box<dyn TtsEngine> {
    #[cfg(target_os = "macos")]
    {
        Box::new(MacTts::new())
    }
    #[cfg(target_os = "windows")]
    {
        Box::new(WindowsTts::new())
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Box::new(DummyTts)
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod dummy {
    use super::*;

    pub struct DummyTts;

    impl TtsEngine for DummyTts {
        fn speak(&self, _text: &str, _lang: &str) -> Result<(), String> {
            Ok(())
        }
    }
}
