use vingilot_core::reader::ExtractedArticle;

/// Supported Reader View visual themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReaderTheme {
    #[default]
    Light,
    Sepia,
    Dark,
    TrueBlack,
}

impl ReaderTheme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Sepia => "sepia",
            Self::Dark => "dark",
            Self::TrueBlack => "true-black",
        }
    }

    pub fn background_color(&self) -> &'static str {
        match self {
            Self::Light => "#ffffff",
            Self::Sepia => "#fbf0d9",
            Self::Dark => "#1e1e1e",
            Self::TrueBlack => "#000000",
        }
    }

    pub fn text_color(&self) -> &'static str {
        match self {
            Self::Light => "#111111",
            Self::Sepia => "#433422",
            Self::Dark => "#e0e0e0",
            Self::TrueBlack => "#e6e6e6",
        }
    }
}

/// Reader View UI controller managing active article rendering, theme, and font size.
#[derive(Debug, Clone)]
pub struct ReaderView {
    is_active: bool,
    article: Option<ExtractedArticle>,
    theme: ReaderTheme,
    font_size: u32, // in px (e.g. 18)
}

impl Default for ReaderView {
    fn default() -> Self {
        Self::new()
    }
}

impl ReaderView {
    pub fn new() -> Self {
        Self {
            is_active: false,
            article: None,
            theme: ReaderTheme::Light,
            font_size: 18,
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn activate(&mut self, article: ExtractedArticle) {
        self.article = Some(article);
        self.is_active = true;
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.article = None;
    }

    pub fn article(&self) -> Option<&ExtractedArticle> {
        self.article.as_ref()
    }

    pub fn theme(&self) -> ReaderTheme {
        self.theme
    }

    pub fn set_theme(&mut self, theme: ReaderTheme) {
        self.theme = theme;
    }

    pub fn font_size(&self) -> u32 {
        self.font_size
    }

    pub fn increase_font_size(&mut self) {
        if self.font_size < 36 {
            self.font_size += 2;
        }
    }

    pub fn decrease_font_size(&mut self) {
        if self.font_size > 12 {
            self.font_size -= 2;
        }
    }

    /// Generates self-contained standalone HTML with customized styling for WebKitGTK display.
    pub fn render_html(&self) -> Option<String> {
        let article = self.article.as_ref()?;
        let bg = self.theme.background_color();
        let fg = self.theme.text_color();
        let byline_html = article
            .byline
            .as_deref()
            .map(|b| format!("<p class=\"byline\">By {}</p>", b))
            .unwrap_or_default();

        Some(format!(
            r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{}</title>
<style>
body {{
    background-color: {};
    color: {};
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    font-size: {}px;
    line-height: 1.65;
    max-width: 680px;
    margin: 40px auto;
    padding: 0 20px;
}}
h1 {{ line-height: 1.25; margin-bottom: 8px; }}
.byline {{ opacity: 0.75; font-style: italic; margin-bottom: 24px; }}
img {{ max-width: 100%; height: auto; border-radius: 6px; }}
</style>
</head>
<body>
<h1>{}</h1>
{}
<div class="content">
{}
</div>
</body>
</html>"#,
            article.title, bg, fg, self.font_size, article.title, byline_html, article.content_html
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reader_view_html_generation() {
        let mut view = ReaderView::new();
        assert!(!view.is_active());
        assert!(view.render_html().is_none());

        let article = ExtractedArticle {
            title: "Exploring Aman".to_string(),
            byline: Some("Eärendil".to_string()),
            content_html: "<p>The blessed realm.</p>".to_string(),
            text_content: "The blessed realm.".to_string(),
            word_count: 3,
            estimated_reading_time_minutes: 1,
        };

        view.activate(article);
        assert!(view.is_active());

        view.set_theme(ReaderTheme::TrueBlack);
        assert_eq!(view.theme().background_color(), "#000000");

        let html = view.render_html().unwrap();
        assert!(html.contains("Exploring Aman"));
        assert!(html.contains("#000000"));
        assert!(html.contains("By Eärendil"));
    }
}
