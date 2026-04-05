use anyhow::Result;
use clap::Args;
use owo_colors::OwoColorize;

use crate::{git, meta};

#[derive(Args)]
pub struct StatusArgs {
    /// Compact one-line-per-worktree output
    #[arg(long)]
    pub short: bool,
}

pub fn run(args: StatusArgs) -> Result<()> {
    let repo = git::open_repo()?;
    let worktrees = git::list_worktrees(&repo)?;
    let store = meta::load(&repo)?;

    let cwd = std::env::current_dir()
        .ok()
        .and_then(|p| p.canonicalize().ok());

    if !args.short {
        println!(
            "{:<2} {} {} {} {}",
            "",
            format!("{:<20}", "NAME").bold(),
            format!("{:<8}", "STATE").bold(),
            format!("{:<14}", "DIFF").bold(),
            "TASK".bold(),
        );
    }

    for wt in &worktrees {
        let stat = git::open_repo_at(&wt.path)
            .and_then(|r| git::diff_stat(&r, &wt.path))
            .unwrap_or_default();

        let is_current = cwd.as_deref().is_some_and(|c| {
            wt.path
                .canonicalize()
                .map(|p| c.starts_with(&p))
                .unwrap_or(false)
        });
        let task = store
            .worktrees
            .get(&wt.name)
            .and_then(|m| m.task.as_deref())
            .unwrap_or("");

        if args.short {
            let marker = if is_current {
                format!("{}", "*".green().bold())
            } else {
                " ".to_string()
            };
            if stat.is_clean() {
                println!("{}{} {}", marker, wt.name, "clean".green());
            } else {
                println!(
                    "{}{} {} {}",
                    marker,
                    wt.name,
                    format!("+{}", stat.insertions).green(),
                    format!("-{}", stat.deletions).red(),
                );
            }
        } else {
            let marker = if is_current {
                format!("{}", "*".green().bold())
            } else {
                " ".to_string()
            };
            let name_col = format!("{:<20}", truncate_str(&wt.name, 20));
            let state_col = if stat.is_clean() {
                format!("{:<8}", "clean").green().to_string()
            } else {
                format!("{:<8}", "dirty").red().to_string()
            };
            let diff_col = if stat.is_clean() {
                format!("{:<14}", "")
            } else {
                let ins = format!("+{}", stat.insertions).green().to_string();
                let del = format!("-{}", stat.deletions).red().to_string();
                format!("{} {:<8}", ins, del)
            };

            println!(
                "{:<2} {} {} {} {}",
                marker, name_col, state_col, diff_col, task
            );
        }
    }

    Ok(())
}

fn truncate_str(s: &str, max: usize) -> &str {
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end.min(s.len())]
}
