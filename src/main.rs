use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::collections::HashMap; 
use clap::{Parser, Subcommand};
use chrono::TimeZone;
use regex::Regex;
use colored::*; 

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
} 

#[derive(Subcommand)]
enum Commands {
    Freq {cmd: Option<String>},
    Find {cmd: String},
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
        // No subcommand: default to "recent"
        None => {
            let entries = parse_history_entries(&path, None, &re)?;
            print_recent_commands(&entries);
        }
        Some(Commands::Find{cmd}) => {
            // cmd is &String, pass as &str
            let entries = parse_history_entries(&path, Some(cmd.as_str()), &re)?;
            print_recent_commands(&entries);
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
            if !(cmd_lower.starts_with(&f) || cmd_lower.contains(&f)) {
                continue;
            }
        }
        *counts.entry(entry.command.clone()).or_insert(0) += 1;
    }
    counts
}




fn parse_history_entries(path: &str, cmd_filter: Option<&str>, re: &Regex) -> io::Result<Vec<HistoryEntry>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    for line in reader.lines() {
        let line = match line {
            Ok(s) => s,
            Err(_) => continue,
        };
        if let Some(caps) = re.captures(&line) {
            let cmd = caps[2].to_string();
            let ts: i64 = caps[1].parse().unwrap_or(0);
            if let Some(filter) = cmd_filter {
                let filter = filter.trim().to_lowercase();
                let cmd_lower = cmd.to_lowercase();
                if !(cmd_lower.starts_with(&filter) || cmd_lower.contains(&filter)) {
                    continue;
                }
            }
            entries.push(HistoryEntry {
                timestamp: Some(ts),
                command: cmd,
            });
        }
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

fn print_recent_commands(entries: &Vec<HistoryEntry>) {
    println!("\n{}", "--- RECENT COMMANDS ---".bold().yellow());
    let mut sorted: Vec<&HistoryEntry> = entries.iter().collect();
    sorted.sort_by_key(|e| e.timestamp.unwrap_or(0));
    let start = if sorted.len() > 1000 { sorted.len() - 1000 } else { 0 };

    for entry in &sorted[start..] {
        if let Some(ts) = entry.timestamp {
            if let Some(dt) = chrono::Local.timestamp_opt(ts, 0).single() {
                println!("{} | {}", dt.format("%Y-%m-%d %H:%M:%S").to_string().blue(), entry.command);
            } else {
                println!("{} | {}", "Invalid date".blue(), entry.command);
            }
        } else {
            println!("{} | {}", "N/A".blue(), entry.command);
        }
    }
}