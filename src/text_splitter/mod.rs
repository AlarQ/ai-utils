#![allow(dead_code)]

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use text_service::TextSplitter;

mod text_service;

#[derive(Debug)]
struct Report {
    file: String,
    avg_chunk_size: f64,
    median_chunk_size: usize,
    min_chunk_size: usize,
    max_chunk_size: usize,
    total_chunks: usize,
}

fn process_file(file_path: &PathBuf, splitter: &TextSplitter, limit: usize) -> Result<Report> {
    let text = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    let docs = splitter.split(&text, limit)?;

    let json_path = file_path.with_extension("json");
    fs::write(
        &json_path,
        serde_json::to_string_pretty(&docs)
            .with_context(|| "Failed to serialize chunks to JSON")?,
    )
    .with_context(|| format!("Failed to write JSON file: {}", json_path.display()))?;

    let chunk_sizes: Vec<usize> = docs.iter().map(|doc| doc.metadata.tokens).collect();
    let avg_chunk_size = chunk_sizes.iter().sum::<usize>() as f64 / chunk_sizes.len() as f64;
    let min_chunk_size = *chunk_sizes.iter().min().unwrap_or(&0);
    let max_chunk_size = *chunk_sizes.iter().max().unwrap_or(&0);
    let mut sorted_sizes = chunk_sizes.clone();
    sorted_sizes.sort_unstable();
    let median_chunk_size = sorted_sizes[sorted_sizes.len() / 2];

    Ok(Report {
        file: file_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
        avg_chunk_size,
        median_chunk_size,
        min_chunk_size,
        max_chunk_size,
        total_chunks: chunk_sizes.len(),
    })
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn test() -> Result<()> {
        // Initialize tracing
        env::set_var("INPUT_PATH", "example_article.md");
        tracing_subscriber::fmt::init();

        let input_path =
            std::env::var("INPUT_PATH").context("INPUT_PATH environment variable not set")?;
        let input_path = PathBuf::from(input_path);

        let token_limit = std::env::var("TOKEN_LIMIT")
            .unwrap_or_else(|_| "1000".to_string())
            .parse::<usize>()
            .context("TOKEN_LIMIT must be a valid number")?;

        let splitter = TextSplitter::new(None);
        let mut reports = Vec::new();

        if input_path.is_file() {
            let report = process_file(&input_path, &splitter, token_limit)?;
            reports.push(report);
        } else if input_path.is_dir() {
            for entry in fs::read_dir(&input_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
                    if let Ok(report) = process_file(&path, &splitter, token_limit) {
                        reports.push(report);
                    }
                }
            }
        }

        // Print reports in a table format
        println!("\nProcessing Report:");
        println!(
            "{:<30} {:<15} {:<15} {:<15} {:<15} {:<15}",
            "File", "Avg Size", "Median Size", "Min Size", "Max Size", "Total Chunks"
        );
        println!("{:-<105}", "");

        for report in reports {
            println!(
                "{:<30} {:<15.2} {:<15} {:<15} {:<15} {:<15}",
                report.file,
                report.avg_chunk_size,
                report.median_chunk_size,
                report.min_chunk_size,
                report.max_chunk_size,
                report.total_chunks
            );
        }

        Ok(())
    }
}
