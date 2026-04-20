use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslateResponse {
    pub translation: String,
    pub detected_source_lang: Option<String>,
}

async fn translate_text_deepl(text: &str, from: &str, to: &str) -> Result<String, String> {
    let api_key = std::env::var("DEEPL_API_KEY").map_err(|_| "DEEPL_API_KEY not set")?;

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api-free.deepl.com/v2/translate")
        .header("Authorization", format!("DeepL-Auth-Key {api_key}"))
        .form(&[
            ("text", text),
            ("source_lang", from),
            ("target_lang", to),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    #[derive(Deserialize)]
    struct DeeplResponse {
        translations: Vec<DeeplTranslation>,
    }
    #[derive(Deserialize)]
    struct DeeplTranslation {
        text: String,
    }

    let body: DeeplResponse = resp.json().await.map_err(|e| e.to_string())?;
    body.translations
        .first()
        .map(|t| t.text.clone())
        .ok_or_else(|| "No translation returned".to_string())
}

#[tauri::command]
pub async fn translate_text(
    text: String,
    source_lang: String,
    target_lang: String,
) -> Result<TranslateResponse, String> {
    let translation =
        translate_text_deepl(&text, &source_lang, &target_lang).await?;
    Ok(TranslateResponse {
        translation,
        detected_source_lang: None,
    })
}
