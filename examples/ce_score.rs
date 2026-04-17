// TODO: whole thing should be optimized because now it's really too much slow!
use anyhow::{Context, Result};
use dom_content_extraction::{DensityTree, scraper::Html};
use rayon::prelude::*;
use regex::Regex;
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};
use strsim::sorensen_dice;

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<&str>>().join(" ")
}

fn clean_and_normalize_text(text: &str) -> String {
    // Remove all punctuation except apostrophes
    let punctuation_regex = Regex::new(r"[^\w\s']").unwrap();
    let text = punctuation_regex.replace_all(text, " ");

    // Replace multiple spaces with a single space
    let space_regex = Regex::new(r"\s+").unwrap();
    let text = space_regex.replace_all(&text, " ");

    // Convert to lowercase
    let text = text.to_lowercase();

    // Trim leading and trailing spaces
    text.trim().to_string()
}

fn extract_content_from_html(file_path: &Path) -> Result<String> {
    // let content = fs::read_to_string(file_path)
    //     .with_context(|| format!("Failed to read file: {:?}", file_path))?;
    let content = fs::read(file_path)
        .with_context(|| format!("Failed to read file: {:?}", file_path))?;
    let content = String::from_utf8_lossy(&content).into_owned();

    let document = Html::parse_document(&content);
    let mut dtree = DensityTree::from_document(&document).unwrap();
    let _ = dtree.calculate_density_sum();
    let extracted_content = dtree.extract_content(&document).unwrap();

    Ok(normalize_text(&extracted_content))
}

fn clean_txt_file(file_path: &Path) -> Result<String> {
    // let content = fs::read_to_string(file_path)
    //     .with_context(|| format!("Failed to read file: {:?}", file_path))?;
    let content = fs::read(file_path)
        .with_context(|| format!("Failed to read file: {:?}", file_path))?;
    let content = String::from_utf8_lossy(&content).into_owned();

    // Remove URL line from the top
    let content = content.lines().skip(1).collect::<Vec<&str>>().join("\n");

    // Remove tags markup
    let re = Regex::new(r"<[hl/p]+>")?;
    let content = re.replace_all(&content, "");

    // Remove extra spaces and newlines
    let content = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join(" ");

    Ok(content)
}

fn calculate_lcs(s1: &str, s2: &str) -> usize {
    // Split into words instead of characters
    let s1: Vec<&str> = s1.split_whitespace().collect();
    let s2: Vec<&str> = s2.split_whitespace().collect();
    let (m, n) = (s1.len(), s2.len());
    let mut prev = vec![0; n + 1];
    let mut curr = vec![0; n + 1];

    for i in 1..=m {
        for j in 1..=n {
            if s1[i - 1] == s2[j - 1] {
                curr[j] = prev[j - 1] + 1;
            } else {
                curr[j] = curr[j - 1].max(prev[j]);
            }
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    // Convert word count to approximate character count
    let lcs_words = prev[n];
    if lcs_words == 0 {
        return 0;
    }

    // Calculate average word length in both strings
    let avg_word_len1 = if s1.is_empty() {
        0.0
    } else {
        s1.iter().map(|w| w.len()).sum::<usize>() as f64 / s1.len() as f64
    };
    let avg_word_len2 = if s2.is_empty() {
        0.0
    } else {
        s2.iter().map(|w| w.len()).sum::<usize>() as f64 / s2.len() as f64
    };
    let avg_word_len = (avg_word_len1 + avg_word_len2) / 2.0;

    // Convert to character count (add 1 for space between words)
    (lcs_words as f64 * (avg_word_len + 1.0)) as usize
}

/* fn calculate_lcs(s1: &str, s2: &str) -> usize {
    let s1: Vec<char> = s1.chars().collect();
    let s2: Vec<char> = s2.chars().collect();
    let (m, n) = (s1.len(), s2.len());
    let mut prev = vec![0; n + 1];
    let mut curr = vec![0; n + 1];

    for i in 1..=m {
        for j in 1..=n {
            if s1[i - 1] == s2[j - 1] {
                curr[j] = prev[j - 1] + 1;
            } else {
                curr[j] = curr[j - 1].max(prev[j]);
            }
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
} */

struct ScoringResult {
    precision: f64,
    recall: f64,
    f1: f64,
    dice: f64,
    duration: Duration,
}

fn process_file_pair(txt_path: &Path, html_path: &Path) -> Result<ScoringResult> {
    let file_start = Instant::now();
    let clean_content = clean_txt_file(txt_path)?;
    let clean_content = clean_and_normalize_text(&clean_content);

    let extracted_content = extract_content_from_html(html_path)?;
    let extracted_content = clean_and_normalize_text(&extracted_content);

    let lcs_length = calculate_lcs(&clean_content, &extracted_content);
    let precision = lcs_length as f64 / extracted_content.len() as f64;
    let recall = lcs_length as f64 / clean_content.len() as f64;
    let f1 = 2.0 * (precision * recall) / (precision + recall);
    let dice = sorensen_dice(&extracted_content, &clean_content);

    Ok(ScoringResult {
        precision,
        recall,
        f1,
        dice,
        duration: file_start.elapsed(),
    })
}

fn main() -> Result<()> {
    let gold_standard_dir = Path::new("data/GoldStandard");
    let html_input_dir = Path::new("data/finalrun-input");
    let start_time = Instant::now();

    let entries: Vec<_> =
        fs::read_dir(gold_standard_dir)?.collect::<std::io::Result<Vec<_>>>()?;

    let results: Vec<_> = entries
        .par_iter()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("txt") {
                let file_name = path.file_stem().unwrap().to_str().unwrap();
                let html_path = html_input_dir.join(format!("{}.html", file_name));

                if html_path.exists() {
                    match process_file_pair(&path, &html_path) {
                        Ok(result)
                            if !result.precision.is_nan()
                                && !result.recall.is_nan()
                                && !result.f1.is_nan() =>
                        {
                            println!("File: {}", file_name);
                            println!("  Precision: {:.2}", result.precision);
                            println!("  Recall: {:.2}", result.recall);
                            println!("  F1 Score: {:.2}", result.f1);
                            println!("  Sorensen-Dice: {:.2}", result.dice);
                            println!("  Processing time: {:.2?}", result.duration);
                            if result.duration > Duration::from_millis(500) {
                                println!("  ⚠️ SLOW PROCESSING");
                            }
                            println!();
                            Some(result)
                        }
                        Ok(_) => {
                            println!(
                                "File: {} produced NaN results (skipped)",
                                file_name
                            );
                            None
                        }
                        Err(e) => {
                            println!(
                                "Error processing file {}: {:?}",
                                file_name, e
                            );
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let total_results = results.len();
    if total_results == 0 {
        println!("No valid results found.");
        return Ok(());
    }

    let (total_precision, total_recall, total_f1, total_dice) =
        results.iter().fold((0.0, 0.0, 0.0, 0.0), |acc, r| {
            (
                acc.0 + r.precision,
                acc.1 + r.recall,
                acc.2 + r.f1,
                acc.3 + r.dice,
            )
        });

    let n = total_results as f64;
    println!("Overall Performance:");
    println!("  Files processed: {}", total_results);
    println!("  Average Precision: {:.2}", total_precision / n);
    println!("  Average Recall: {:.2}", total_recall / n);
    println!("  Average F1 Score: {:.2}", total_f1 / n);
    println!("  Average Sorensen-Dice: {:.2}", total_dice / n);

    let total_duration = start_time.elapsed();
    println!("Total processing time: {:.2?}", total_duration);
    println!(
        "Average time per file: {:.2?}",
        total_duration / total_results as u32
    );

    Ok(())
}
