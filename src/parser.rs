// src/parser.rs

use scraper::{Html, Selector};
use url::Url;

/// Parses the HTML content and extracts the title and meta description.
pub fn parse_html(html: &str) -> (Option<String>, Option<String>) {
    let document = Html::parse_document(html);

    // Extract title
    let title_selector = Selector::parse("title").unwrap();
    let title = document
        .select(&title_selector)
        .next()
        .and_then(|elem| Some(elem.text().collect::<Vec<_>>().concat()));

    // Extract meta description
    let meta_selector = Selector::parse(r#"meta[name="description"]"#).unwrap();
    let description = document
        .select(&meta_selector)
        .next()
        .and_then(|elem| elem.value().attr("content").map(|s| s.to_string()));

    (title, description)
}

/// Extracts all href links from the given HTML content.
///
/// # Arguments
///
/// * `html` - A string slice containing the HTML content.
/// * `base_url` - The base URL to resolve relative URLs.
///
/// # Returns
///
/// A vector of extracted absolute URLs as strings.
pub fn extract_links(html: &str, base_url: &Url) -> Vec<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("a[href]").unwrap();
    let mut links = Vec::new();

    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") { // Fixed typo from "helf" to "href"
            // Attempt to resolve the href against the base URL
            if let Ok(mut resolved_url) = base_url.join(href) {
                // Remove fragment to avoid duplicates
                resolved_url.set_fragment(None);

                // Only include http and https schemes
                match resolved_url.scheme() {
                    "http" | "https" => {
                        links.push(resolved_url.to_string()); // Replaced `into_string` with `to_string`
                    },
                    _ => (), // Skip other schemes like mailto, javascript, etc.
                }
            }
        }
    }

    links // Ensure the function returns the `links` vector
}

/// Sanitizes and validates a URL string.
///
/// # Arguments
///
/// * `link` - A string slice containing the URL to sanitize.
///
/// # Returns
///
/// An `Option<String>` containing the sanitized URL if valid, or `None` otherwise.
pub fn sanitize_link(link: &str) -> Option<String> {
    match Url::parse(link) {
        Ok(url) => {
            match url.scheme() {
                "http" | "https" => Some(url.to_string()), // Replaced `into_string` with `to_string`
                _ => None,
            }
        },
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn test_parse_html() {
        let html = r#"
            <html>
                <head>
                    <title>Test Page</title>
                    <meta name="description" content="This is a test page.">
                </head>
                <body></body>
            </html>
        "#;

        let (title, description) = parse_html(html);
        assert_eq!(title, Some("Test Page".to_string()));
        assert_eq!(description, Some("This is a test page.".to_string()));
    }

    #[test]
    fn test_extract_links() {
        let html = r#"
            <html>
                <body>
                    <a href="https://www.google.com">Google</a>
                    <a href="/about">About Us</a>
                    <a href="javascript:void(0)">Invalid Link</a>
                    <a href="mailto:test@example.com">Email</a>
                </body>
            </html>
        "#;

        let base_url = Url::parse("https://www.example.com").unwrap();
        let links = extract_links(html, &base_url);

        assert_eq!(links.len(), 2);
        assert_eq!(links[0], "https://www.google.com/");
        assert_eq!(links[1], "https://www.example.com/about");
    }

    #[test]
    fn test_sanitize_link() {
        let valid_http = "https://www.google.com";
        let valid_https = "https://www.example.com/about";
        let invalid_scheme = "javascript:void(0)";
        let invalid_url = "ht!tp://invalid-url";

        assert_eq!(sanitize_link(valid_http), Some(valid_http.to_string()));
        assert_eq!(sanitize_link(valid_https), Some(valid_https.to_string()));
        assert_eq!(sanitize_link(invalid_scheme), None);
        assert_eq!(sanitize_link(invalid_url), None);
    }
}
