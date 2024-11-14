use anyhow::{Context, Result};
use dom_content_extraction::scraper::Html;
use dom_content_extraction::DensityTree;
use rayon::prelude::*;
use regex::Regex;
use std::{fs, path::Path};

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
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {:?}", file_path))?;

    let document = Html::parse_document(&content);
    let mut dtree = DensityTree::from_document(&document).unwrap();
    let _ = dtree.calculate_density_sum();
    let extracted_content = dtree.extract_content(&document).unwrap();

    Ok(normalize_text(&extracted_content))
}

fn clean_txt_file(file_path: &Path) -> Result<String> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {:?}", file_path))?;

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
}

fn process_file_pair(txt_path: &Path, html_path: &Path) -> Result<(f64, f64, f64)> {
    let clean_content = clean_txt_file(txt_path)?;
    let clean_content = clean_and_normalize_text(&clean_content);

    // let extracted_content =
    //     clean_and_normalize_text(&extract_content_from_html(html_path)?);

    let extracted_content = extract_content_from_html(html_path)?;
    let extracted_content = clean_and_normalize_text(&extracted_content);

    let lcs_length = calculate_lcs(&clean_content, &extracted_content);
    let precision = lcs_length as f64 / extracted_content.len() as f64;
    let recall = lcs_length as f64 / clean_content.len() as f64;
    let f1_score = 2.0 * (precision * recall) / (precision + recall);

    Ok((precision, recall, f1_score))
}

fn main() -> Result<()> {
    let gold_standard_dir = Path::new("data/GoldStandard");
    let html_input_dir = Path::new("data/finalrun-input");

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
                        Ok((precision, recall, f1))
                            if !precision.is_nan()
                                && !recall.is_nan()
                                && !f1.is_nan() =>
                        {
                            println!("File: {}", file_name);
                            println!("  Precision: {:.2}", precision);
                            println!("  Recall: {:.2}", recall);
                            println!("  F1 Score: {:.2}", f1);
                            println!();
                            Some((precision, recall, f1))
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

    let (total_precision, total_recall, total_f1): (f64, f64, f64) =
        results.iter().fold((0.0, 0.0, 0.0), |acc, &(p, r, f)| {
            (acc.0 + p, acc.1 + r, acc.2 + f)
        });

    let avg_precision = total_precision / total_results as f64;
    let avg_recall = total_recall / total_results as f64;
    let avg_f1 = total_f1 / total_results as f64;

    println!("Overall Performance:");
    println!("  Files processed: {}", total_results);
    println!("  Average Precision: {:.2}", avg_precision);
    println!("  Average Recall: {:.2}", avg_recall);
    println!("  Average F1 Score: {:.2}", avg_f1);

    Ok(())
}
