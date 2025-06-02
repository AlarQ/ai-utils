use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tiktoken_rs::cl100k_base;
use tracing::{debug, info};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Doc {
    pub text: String,
    pub metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub tokens: usize,
    pub headers: Headers,
    pub urls: Vec<String>,
    pub images: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Headers(HashMap<String, Vec<String>>);

impl Headers {
    fn new() -> Self {
        Headers(HashMap::new())
    }

    fn insert(&mut self, key: String, value: String) {
        self.0.entry(key).or_insert_with(Vec::new).push(value);
    }

    fn clear_lower_headers(&mut self, level: usize) {
        for l in (level + 1)..=6 {
            self.0.remove(&format!("h{}", l));
        }
    }
}

pub struct TextSplitter {
    tokenizer: tiktoken_rs::CoreBPE,
    model_name: String,
}

impl TextSplitter {
    pub fn new(model_name: Option<String>) -> Self {
        Self {
            tokenizer: cl100k_base().unwrap(),
            model_name: model_name.unwrap_or_else(|| "gpt-4".to_string()),
        }
    }

    fn count_tokens(&self, text: &str) -> usize {
        let formatted_content = self.format_for_tokenization(text);
        self.tokenizer
            .encode_with_special_tokens(&formatted_content)
            .len()
    }

    fn format_for_tokenization(&self, text: &str) -> String {
        format!(
            "<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant<|im_end|>",
            text
        )
    }

    pub fn split(&self, text: &str, limit: usize) -> Result<Vec<Doc>> {
        info!("Starting split process with limit: {} tokens", limit);
        let mut chunks = Vec::new();
        let mut position = 0;
        let total_length = text.len();
        let mut current_headers = Headers::new();

        while position < total_length {
            info!("Processing chunk starting at position: {}", position);
            let (chunk_text, chunk_end) = self.get_chunk(text, position, limit)?;
            let tokens = self.count_tokens(&chunk_text);
            debug!("Chunk tokens: {}", tokens);

            let headers_in_chunk = self.extract_headers(&chunk_text);
            self.update_current_headers(&mut current_headers, &headers_in_chunk);

            let (content, urls, images) = self.extract_urls_and_images(&chunk_text);

            chunks.push(Doc {
                text: content,
                metadata: Metadata {
                    tokens,
                    headers: current_headers.clone(),
                    urls,
                    images,
                },
            });

            info!("Chunk processed. New position: {}", chunk_end);
            position = chunk_end;
        }

        info!("Split process completed. Total chunks: {}", chunks.len());
        Ok(chunks)
    }

    fn get_chunk(&self, text: &str, start: usize, limit: usize) -> Result<(String, usize)> {
        debug!("Getting chunk starting at {} with limit {}", start, limit);
        let overhead = self.count_tokens(&self.format_for_tokenization("")) - self.count_tokens("");

        let mut end = (start + ((text.len() - start) * limit / self.count_tokens(&text[start..])))
            .min(text.len());

        let mut chunk_text = text[start..end].to_string();
        let mut tokens = self.count_tokens(&chunk_text);

        while tokens + overhead > limit && end > start {
            debug!(
                "Chunk exceeds limit with {} tokens. Adjusting end position...",
                tokens + overhead
            );
            end = self.find_new_chunk_end(text, start, end);
            chunk_text = text[start..end].to_string();
            tokens = self.count_tokens(&chunk_text);
        }

        end = self.adjust_chunk_end(text, start, end, tokens + overhead, limit);
        chunk_text = text[start..end].to_string();
        debug!("Final chunk end: {}", end);
        Ok((chunk_text, end))
    }

    fn adjust_chunk_end(
        &self,
        text: &str,
        start: usize,
        end: usize,
        _current_tokens: usize,
        limit: usize,
    ) -> usize {
        let min_chunk_tokens = (limit as f64 * 0.8) as usize;

        let next_newline = text[end..].find('\n').map(|pos| end + pos + 1);
        let prev_newline = text[..end].rfind('\n').map(|pos| pos + 1);

        // Try extending to next newline
        if let Some(next) = next_newline {
            let chunk_text = text[start..next].to_string();
            let tokens = self.count_tokens(&chunk_text);
            if tokens <= limit && tokens >= min_chunk_tokens {
                debug!("Extending chunk to next newline at position {}", next);
                return next;
            }
        }

        // Try reducing to previous newline
        if let Some(prev) = prev_newline {
            if prev > start {
                let chunk_text = text[start..prev].to_string();
                let tokens = self.count_tokens(&chunk_text);
                if tokens <= limit && tokens >= min_chunk_tokens {
                    debug!("Reducing chunk to previous newline at position {}", prev);
                    return prev;
                }
            }
        }

        // Return original end if adjustments aren't suitable
        end
    }

    fn find_new_chunk_end(&self, _text: &str, start: usize, end: usize) -> usize {
        // Reduce end position to try to fit within token limit
        let new_end = end - ((end - start) / 10);
        if new_end <= start {
            start + 1
        } else {
            new_end
        }
    }

    fn extract_headers(&self, text: &str) -> Headers {
        let mut headers = Headers::new();
        let header_regex = Regex::new(r"(?m)^(#{1,6})\s+(.*)$").unwrap();

        for cap in header_regex.captures_iter(text) {
            let level = cap[1].len();
            let content = cap[2].trim().to_string();
            headers.insert(format!("h{}", level), content);
        }

        headers
    }

    fn update_current_headers(&self, current: &mut Headers, extracted: &Headers) {
        for level in 1..=6 {
            let key = format!("h{}", level);
            if let Some(values) = extracted.0.get(&key) {
                for value in values {
                    current.insert(key.clone(), value.clone());
                }
                current.clear_lower_headers(level);
            }
        }
    }

    fn extract_urls_and_images(&self, text: &str) -> (String, Vec<String>, Vec<String>) {
        let mut urls = Vec::new();
        let mut images = Vec::new();
        let url_index = 0;
        let image_index = 0;

        let image_regex = Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap();
        let url_regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();

        let content = image_regex
            .replace_all(text, |caps: &regex::Captures| {
                let url = caps[2].to_string();
                images.push(url);
                let alt_text = &caps[1];
                format!("![{}]({{$img{}}})", alt_text, image_index)
            })
            .to_string();

        let content = url_regex
            .replace_all(&content, |caps: &regex::Captures| {
                let url = caps[2].to_string();
                urls.push(url);
                let link_text = &caps[1];
                format!("[{}]({{$url{}}})", link_text, url_index)
            })
            .to_string();

        (content, urls, images)
    }
}
