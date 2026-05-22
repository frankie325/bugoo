use super::types::SelectionReadError;

pub fn read_selected_text() -> Result<Option<String>, SelectionReadError> {
    read_selected_text_with(get_selected_text::get_selected_text)
}

pub fn read_selected_text_with<F>(read: F) -> Result<Option<String>, SelectionReadError>
where
    F: FnOnce() -> Result<String, Box<dyn std::error::Error>>,
{
    let text = read().map_err(|error| {
        SelectionReadError::ReadFailed(format!("failed to read selected text: {error}"))
    })?;
    let text = text.trim().to_string();
    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_non_empty_selected_text() {
        let result = read_selected_text_with(|| Ok("  hello  ".to_string()));
        assert_eq!(result, Ok(Some("hello".to_string())));
    }

    #[test]
    fn returns_none_for_empty_selected_text() {
        let result = read_selected_text_with(|| Ok("   \n\t  ".to_string()));
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn maps_reader_errors() {
        let result = read_selected_text_with(|| Err("boom".into()));
        assert_eq!(
            result,
            Err(SelectionReadError::ReadFailed(
                "failed to read selected text: boom".to_string(),
            )),
        );
    }
}
