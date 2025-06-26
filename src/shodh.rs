use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::BinaryHeap;
use rayon::prelude::*;

const VERSION: &str = "0.1.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaseSensitivity {
    Sensitive,
    Insensitive,
}

#[derive(Debug)]
struct Config {
    query: String,
    root: String,
    num: usize,
    files_only: bool,
    dirs_only: bool,
    case: CaseSensitivity,
    parallel: bool,
    help: bool,
    version: bool,
}

impl Config {
    fn from_args() -> Result<Self, String> {
        let args: Vec<String> = env::args().collect();
        let mut query = None;
        let mut root = None;
        let mut num = 10;
        let mut files_only = false;
        let mut dirs_only = false;
        let mut case = CaseSensitivity::Insensitive;
        let mut parallel = true;
        let mut help = false;
        let mut version = false;
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "-h" | "--help" => { help = true; i += 1; },
                "-v" | "--version" => { version = true; i += 1; },
                "-n" | "--num" => {
                    if i + 1 >= args.len() {
                        return Err("Expected a number after --num".to_string());
                    }
                    num = args[i + 1].parse().map_err(|_| "Invalid number for --num".to_string())?;
                    i += 2;
                },
                "--files-only" => { files_only = true; i += 1; },
                "--dirs-only" => { dirs_only = true; i += 1; },
                "-i" | "--ignore-case" => { case = CaseSensitivity::Insensitive; i += 1; },
                "-s" | "--case-sensitive" => { case = CaseSensitivity::Sensitive; i += 1; },
                "--no-parallel" => { parallel = false; i += 1; },
                _ => {
                    if query.is_none() {
                        query = Some(args[i].clone());
                        i += 1;
                    } else if root.is_none() {
                        root = Some(args[i].clone());
                        i += 1;
                    } else {
                        return Err(format!("Unknown argument: {}", args[i]));
                    }
                }
            }
        }
        if help || version {
            return Ok(Config {
                query: String::new(),
                root: String::new(),
                num,
                files_only,
                dirs_only,
                case,
                parallel,
                help,
                version,
            });
        }
        let query = query.ok_or("Missing query argument. Use -h for help.")?;
        let root = root.unwrap_or_else(|| ".".to_string());
        Ok(Config {
            query,
            root,
            num,
            files_only,
            dirs_only,
            case,
            parallel,
            help,
            version,
        })
    }
}

// Struct to hold a candidate path and its score
#[derive(Eq, PartialEq)]
struct ScoredPath {
    score: i32,
    path: PathBuf,
}

// For max-heap (highest score first)
impl Ord for ScoredPath {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.cmp(&other.score).then_with(|| other.path.cmp(&self.path))
    }
}
impl PartialOrd for ScoredPath {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn print_help() {
    println!("\x1b[1;36mshodh\x1b[0m - blazing-fast, smart, fuzzy file finder\n");
    println!("\x1b[1mUSAGE\x1b[0m:");
    println!("  shodh [FLAGS] <query> [root_dir]\n");
    println!("\x1b[1mFLAGS\x1b[0m:");
    println!("  -h, --help            Show this help message");
    println!("  -v, --version         Show version info");
    println!("  -n, --num <N>         Limit number of results (default: 10)");
    println!("      --files-only      Only show files");
    println!("      --dirs-only       Only show directories");
    println!("  -i, --ignore-case     Case-insensitive search (default)");
    println!("  -s, --case-sensitive  Case-sensitive search");
    println!("      --no-parallel     Disable parallel scoring");
    println!("\n\x1b[1mEXAMPLES\x1b[0m:");
    println!("  shodh kilo src --files-only -n 20");
    println!("  shodh resume ~/Documents --dirs-only");
}

fn print_version() {
    println!("shodh v{}", VERSION);
}

fn main() {
    let config = match Config::from_args() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
            std::process::exit(1);
        }
    };
    if config.help {
        print_help();
        return;
    }
    if config.version {
        print_version();
        return;
    }
    let mut candidates = Vec::new();
    if let Err(e) = walk_dir(Path::new(&config.root), &mut candidates) {
        eprintln!("\x1b[1;31mError traversing directory:\x1b[0m {}", e);
        std::process::exit(1);
    }
    let scored: Vec<_> = if config.parallel {
        candidates.par_iter()
            .filter_map(|path| filter_and_score(path, &config))
            .collect()
    } else {
        candidates.iter()
            .filter_map(|path| filter_and_score(path, &config))
            .collect()
    };
    let mut heap = BinaryHeap::new();
    for sp in scored {
        heap.push(sp);
    }
    println!("\x1b[1;32m\nResults:\x1b[0m");
    let mut shown = 0;
    for sp in heap.into_sorted_vec().into_iter().rev().take(config.num) {
        let (ty, color) = if sp.path.is_dir() {
            ("DIR ", "\x1b[1;34m")
        } else {
            ("FILE", "\x1b[1;33m")
        };
        println!("{}[{:5}] {}{}\x1b[0m  {}", color, sp.score, ty, color, sp.path.display());
        shown += 1;
    }
    if shown == 0 {
        println!("\x1b[1;31mNo results found.\x1b[0m");
    }
}

// Recursively walk the directory and collect all file and directory paths
fn walk_dir(path: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let meta = fs::metadata(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    if meta.is_dir() {
        let entries = fs::read_dir(path).map_err(|e| format!("{}: {}", path.display(), e))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("{}: {}", path.display(), e))?;
            let p = entry.path();
            out.push(p.clone());
            if p.is_dir() {
                let _ = walk_dir(&p, out); // Continue on error
            }
        }
    } else {
        out.push(path.to_path_buf());
    }
    Ok(())
}

fn filter_and_score(path: &PathBuf, config: &Config) -> Option<ScoredPath> {
    let name = path.file_name()?.to_str()?;
    // Type filtering
    if config.files_only && !path.is_file() {
        return None;
    }
    if config.dirs_only && !path.is_dir() {
        return None;
    }
    // Case sensitivity
    let (query, candidate) = match config.case {
        CaseSensitivity::Insensitive => (config.query.to_lowercase(), name.to_lowercase()),
        CaseSensitivity::Sensitive => (config.query.clone(), name.to_string()),
    };
    let score = fuzzy_score(&query, &candidate);
    if score > 0 {
        Some(ScoredPath { score, path: path.clone() })
    } else {
        None
    }
}

// Smith-Waterman local alignment for fuzzy matching, with big boosts for exact/prefix matches
fn fuzzy_score(query: &str, candidate: &str) -> i32 {
    let q: Vec<char> = query.chars().collect();
    let c: Vec<char> = candidate.chars().collect();
    let m = q.len();
    let n = c.len();
    if m == 0 || n == 0 {
        return 0;
    }
    // Scoring scheme
    let match_score = 2;
    let mismatch_penalty = -1;
    let gap_penalty = -2;
    // DP matrix
    let mut dp = vec![vec![0; n + 1]; m + 1];
    let mut max_score = 0;
    for i in 1..=m {
        for j in 1..=n {
            let score_diag = if q[i - 1] == c[j - 1] {
                dp[i - 1][j - 1] + match_score
            } else {
                dp[i - 1][j - 1] + mismatch_penalty
            };
            let score_up = dp[i - 1][j] + gap_penalty;
            let score_left = dp[i][j - 1] + gap_penalty;
            let score = 0.max(score_diag).max(score_up).max(score_left);
            dp[i][j] = score;
            if score > max_score {
                max_score = score;
            }
        }
    }
    // Boost for exact match
    if query == candidate {
        max_score += 10000;
    }
    // Boost for prefix match
    if candidate.starts_with(query) && query != candidate {
        max_score += 5000;
    }
    max_score
} 