use std::fs::File;
use std::io;
use std::collections::HashMap; 
use clap::{Parser, Subcommand};
use chrono::TimeZone;
use regex::Regex;
use colored::*; 

#[derive(Parser)]
struct Cli {
    /// Filter commands by keywords (OR logic - matches any keyword)
    #[arg(help = "Filter commands (e.g., 'docker python')", value_name = "QUERY")]
    query: Vec<String>,

    #[command(subcommand)]
    command: Option<Commands>,
} 

#[derive(Subcommand)]
enum Commands {
    /// Show most frequent commands; optional filter
    Freq { cmd: Option<String> },
} 

#[derive(Debug)]
struct HistoryEntry {
    timestamp: Option<i64>,
    command: String,
}

fn main() -> io::Result<()> {
    let cli: Cli = Cli::parse();
    // Construct history path from HOME for portability
    let path = get_history_path();

    // Compile regex once and reuse
    let re = Regex::new(r"^: (\d+):\d+;(.*)$").unwrap();

    // Defer reading/parsing to only the requested subcommand
    match &cli.command {
        Some(Commands::Freq{cmd}) => {
            // Read all entries and then compute counts with optional filter
            let entries = parse_history_entries(&path, None, &re)?;
            let counts = counts_from_entries(&entries, cmd.as_deref());
            print_top_commands(counts);
        }
        None => {
            // No subcommand: default to "recent" or apply top-level query filter
            if !cli.query.is_empty() {
                let filters: Vec<&str> = cli.query.iter().map(|s| s.as_str()).collect();
                let entries = parse_history_entries(&path, Some(&filters), &re)?;
                print_recent_commands(&entries, Some(&filters));
            } else {
                let entries = parse_history_entries(&path, None, &re)?;
                print_recent_commands(&entries, None);
            }
        }
    }

    Ok(())
}

fn get_history_path() -> String {
    match std::env::var("HOME") {
        Ok(home) => format!("{}/.zsh_history", home),
        Err(_) => "/Users/Atomu/.zsh_history".to_string(),
    }
}

/// Build counts map from parsed entries with an optional filter
fn counts_from_entries(entries: &[HistoryEntry], filter: Option<&str>) -> HashMap<String, i32> {
    let mut counts = HashMap::new();
    for entry in entries {
        if let Some(f) = filter {
            let f = f.trim().to_lowercase();
            let cmd_lower = entry.command.to_lowercase();
            if !cmd_lower.contains(&f) || cmd_lower.contains("recall") {
                continue;
            }
        }
        *counts.entry(entry.command.clone()).or_insert(0) += 1;
    }
    counts
}

fn parse_history_entries(path: &str, cmd_filters: Option<&[&str]>, re: &Regex) -> io::Result<Vec<HistoryEntry>> {
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    io::Read::read_to_end(&mut file, &mut bytes)?;
    // Lossy-decode rather than skip invalid-UTF-8 lines: the continuation
    // lookahead below indexes by physical line position, so every line must
    // keep its slot in `lines` even if its bytes don't decode cleanly.
    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    let mut entries = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let Some(caps) = re.captures(&lines[i]) else {
            i += 1;
            continue;
        };
        let ts: i64 = caps[1].parse().unwrap_or(0);
        let mut cmd = caps[2].to_string();

        // zsh's history file escapes each embedded newline inside a
        // multi-line command by appending its own trailing '\' marker, on
        // top of whatever '\' the user actually typed for shell
        // line-continuation. Keep consuming physical lines while the
        // current tail ends in '\' AND the next physical line isn't itself
        // a new history entry — that combination is the only reliable
        // signal it's a continuation marker rather than a literal trailing
        // backslash the user typed on the command's last line.
        let mut j = i;
        while cmd.ends_with('\\') {
            match lines.get(j + 1) {
                Some(next) if !re.is_match(next) => {
                    cmd.pop(); // drop the file format's marker only
                    cmd.push('\n');
                    cmd.push_str(next);
                    j += 1;
                }
                _ => break,
            }
        }
        i = j + 1;

        let cmd_lower = cmd.to_lowercase();

        // Skip self-invocations of the recall tool. Only check the first
        // line: a multi-line command may legitimately mention "recall" in
        // an argument on a later line, and that shouldn't hide the whole
        // reconstructed command.
        if cmd_lower.lines().next().unwrap_or("").contains("recall") {
            continue;
        }

        if let Some(filters) = cmd_filters {
            // AND logic: match only if all filters are found anywhere in
            // the (possibly multi-line) reconstructed command
            let matches = filters.iter().all(|f| {
                let f = f.trim().to_lowercase();
                cmd_lower.contains(&f)
            });
            if !matches {
                continue;
            }
        }
        entries.push(HistoryEntry {
            timestamp: Some(ts),
            command: cmd,
        });
    }
    Ok(entries)
}

fn print_top_commands(counts: HashMap<String, i32>) {
    println!("\n{}", "--- MOST FREQUENT COMMANDS ---".bold().yellow());
    
    // Convert HashMap to a Vector so we can sort it
    let mut count_vec: Vec<(&String, &i32)> = counts.iter().collect();
    
    // Sort by count (descending)
    count_vec.sort_by(|a, b| b.1.cmp(a.1));

    for (cmd, count) in count_vec.iter().take(10) {
        println!("{:<5} | {}", count.to_string().green(), cmd);
    }
}

fn highlight_keywords(text: &str, keywords: Option<&[&str]>) -> String {
    match keywords {
        None => text.to_string(),
        Some(kws) => {
            let mut result = text.to_string();
            for kw in kws {
                let kw_lower = kw.to_lowercase();
                // Case-insensitive replacement with highlighting
                let mut new_result = String::new();
                let mut remaining = result.as_str();
                while let Some(pos) = remaining.to_lowercase().find(&kw_lower) {
                    new_result.push_str(&remaining[..pos]);
                    let matched = &remaining[pos..pos + kw.len()];
                    new_result.push_str(&matched.red().to_string());
                    remaining = &remaining[pos + kw.len()..];
                }
                new_result.push_str(remaining);
                result = new_result;
            }
            result
        }
    }
}

fn print_recent_commands(entries: &Vec<HistoryEntry>, keywords: Option<&[&str]>) {
    println!("\n{}", "--- RECENT COMMANDS ---".bold().yellow());
    let mut sorted: Vec<&HistoryEntry> = entries.iter().collect();
    sorted.sort_by_key(|e| e.timestamp.unwrap_or(0));
    
    // Deduplicate: keep only the most recent occurrence of each command
    let mut seen: HashMap<&str, &HistoryEntry> = HashMap::new();
    for entry in &sorted {
        seen.insert(&entry.command, entry);
    }
    let mut unique: Vec<&HistoryEntry> = seen.into_values().collect();
    unique.sort_by_key(|e| e.timestamp.unwrap_or(0));
    
    let start = if unique.len() > 1000 { unique.len() - 1000 } else { 0 };

    for entry in &unique[start..] {
        let highlighted_cmd = highlight_keywords(&entry.command, keywords);
        if let Some(ts) = entry.timestamp {
            if let Some(dt) = chrono::Local.timestamp_opt(ts, 0).single() {
                println!("{} | {}", dt.format("%Y-%m-%d %H:%M:%S").to_string().blue(), highlighted_cmd);
            } else {
                println!("{} | {}", "Invalid date".blue(), highlighted_cmd);
            }
        } else {
            println!("{} | {}", "N/A".blue(), highlighted_cmd);
        }
    }
}