use serde::{Deserialize, Serialize};

/// Extracted article content suitable for distraction-free reading mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedArticle {
    pub title: String,
    pub byline: Option<String>,
    pub content_html: String,
    pub text_content: String,
    pub word_count: usize,
    pub estimated_reading_time_minutes: usize,
}

/// Reader extraction service providing the JavaScript payload and output parser.
pub struct ReaderExtractor;

impl ReaderExtractor {
    /// Returns the JavaScript script to inject into WebKitGTK to parse readable content.
    pub fn extraction_script() -> &'static str {
        r#"
        (function() {
            function getCleanTitle() {
                let ogTitle = document.querySelector('meta[property="og:title"]');
                if (ogTitle && ogTitle.content) return ogTitle.content.trim();
                let h1 = document.querySelector('h1');
                if (h1 && h1.innerText.trim()) return h1.innerText.trim();
                return document.title.trim();
            }

            function getCleanByline() {
                let authorMeta = document.querySelector('meta[name="author"], meta[property="article:author"]');
                if (authorMeta && authorMeta.content) return authorMeta.content.trim();
                let authorEl = document.querySelector('.byline, .author, [rel="author"]');
                if (authorEl && authorEl.innerText.trim()) return authorEl.innerText.trim();
                return null;
            }

            function getArticleBody() {
                let article = document.querySelector('article, [role="main"], main, .post-content, .entry-content');
                if (!article) {
                    article = document.body;
                }
                return {
                    html: article ? article.innerHTML : "",
                    text: article ? article.innerText : ""
                };
            }

            let body = getArticleBody();
            let title = getCleanTitle();
            let byline = getCleanByline();

            return JSON.stringify({
                title: title,
                byline: byline,
                content_html: body.html,
                text_content: body.text
            });
        })();
        "#
    }

    /// Parses the JSON result string returned by the JavaScript extraction script.
    pub fn parse_extraction_result(json_str: &str) -> Result<ExtractedArticle, String> {
        #[derive(Deserialize)]
        struct RawOutput {
            title: String,
            byline: Option<String>,
            content_html: String,
            text_content: String,
        }

        let raw: RawOutput = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse extractor JSON: {e}"))?;

        let word_count = raw.text_content.split_whitespace().count();
        let estimated_reading_time_minutes = word_count.div_ceil(200); // ~200 WPM

        Ok(ExtractedArticle {
            title: raw.title,
            byline: raw.byline,
            content_html: raw.content_html,
            text_content: raw.text_content,
            word_count,
            estimated_reading_time_minutes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_extraction_result() {
        let sample_json = r#"{
            "title": "A Journey on Vingilot",
            "byline": "Eärendil",
            "content_html": "<p>A ship of wonder was constructed.</p>",
            "text_content": "A ship of wonder was constructed."
        }"#;

        let article = ReaderExtractor::parse_extraction_result(sample_json).unwrap();
        assert_eq!(article.title, "A Journey on Vingilot");
        assert_eq!(article.byline.as_deref(), Some("Eärendil"));
        assert_eq!(article.word_count, 6);
        assert_eq!(article.estimated_reading_time_minutes, 1);
    }
}
