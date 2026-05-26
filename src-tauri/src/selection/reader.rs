pub fn read_selected_text() -> Option<String> {
    normalize_selected_text(selection::get_text())
}

fn normalize_selected_text(text: String) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_non_empty_selected_text() {
        let result = normalize_selected_text("  hello  ".to_string());
        assert_eq!(result, Some("hello".to_string()));
    }

    #[test]
    fn returns_none_for_empty_selected_text() {
        let result = normalize_selected_text("   \n\t  ".to_string());
        assert_eq!(result, None);
    }
}
