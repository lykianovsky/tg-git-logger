//! View-model construction and HTML report rendering.
//!
//! All data transformations (stats, task extraction, HTML escaping) happen here
//! in Rust, before being passed to the Askama template. The template itself
//! contains no logic — it only iterates and renders pre-computed values.

use crate::domain::repository::entities::repository_task_tracker::RepositoryTaskTracker;
use crate::domain::shared::date::range::DateRange;
use crate::domain::task::services::task_tracker_service::TaskTrackerService;
use crate::domain::version_control::value_objects::report::{
    VersionControlDateRangeReport, VersionControlDateRangeReportCommit,
    VersionControlDateRangeReportPullRequest,
};
use askama::Template;
use std::collections::{HashMap, HashSet};

// ── View models ───────────────────────────────────────────────────────────────

pub struct DayActivity {
    /// Display label, e.g. "12 Jan"
    pub day: String,
    pub commits: usize,
    /// Bar height in pixels (0–60), proportional to the busiest day.
    pub bar_px: usize,
}

pub struct TaskItem {
    /// Full task URL.
    pub url: String,
    /// HTML-escaped display label (e.g. "TASK-123").
    pub label: String,
}

pub struct ContributorRow {
    /// HTML-escaped author login / name.
    pub name: String,
    pub commits: usize,
    pub additions: i64,
    pub deletions: i64,
    /// Bar width in percent (0–100), relative to the top contributor.
    pub pct: usize,
}

pub struct PrRow {
    pub number: i64,
    /// Pre-computed safe HTML (may contain `<a>` task links).
    pub title_html: String,
    pub state_icon: &'static str,
    pub state_class: &'static str,
    pub state_label: &'static str,
    /// HTML-escaped author name.
    pub author: String,
    pub additions: i64,
    pub deletions: i64,
    pub changed_files: i64,
    pub created_at: String,
    /// Empty string when not merged.
    pub merged_at: String,
}

pub struct CommitRow {
    /// Short SHA (7 chars).
    pub sha: String,
    /// Pre-computed safe HTML (may contain `<a>` task links).
    pub message_html: String,
    /// HTML-escaped author login / name.
    pub author: String,
    pub additions: i64,
    pub deletions: i64,
    /// "—" when `changed_files` is None.
    pub changed_files: String,
    pub date: String,
}

// ── Askama template ───────────────────────────────────────────────────────────

/// All values are pre-escaped / pre-computed in Rust.
/// The template uses `escape = "none"` so it outputs them verbatim.
#[derive(Template)]
#[template(path = "report/full_report.html", escape = "none")]
pub struct FullReportTemplate {
    // Header
    pub title: String,
    pub is_personal: bool,
    pub repo_owner: String,
    pub repo_name: String,
    pub branch: String,
    pub period: String,
    pub generated_at: String,

    // Summary stats
    pub commits_count: usize,
    pub prs_count: usize,
    pub total_additions: i64,
    pub total_deletions: i64,
    pub net_changes_str: String,
    pub total_changed_files: i64,
    pub active_days: usize,
    pub avg_commit_size: i64,
    pub authors_count: usize,
    pub tasks_count: usize,

    // Flattened PR summary (avoids nested struct in template)
    pub pr_summary_merged: usize,
    pub pr_summary_open: usize,
    pub pr_summary_closed: usize,
    pub pr_summary_merge_rate: String,
    pub pr_summary_avg_merge_time: String,

    // Sections
    pub activity: Vec<DayActivity>,
    pub tasks: Vec<TaskItem>,
    pub contributors: Vec<ContributorRow>,
    pub pull_requests: Vec<PrRow>,
    pub commits: Vec<CommitRow>,
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Build and render the full HTML report for the given `report` data.
pub fn build_html_report(
    report: &VersionControlDateRangeReport,
    author: &Option<String>,
    date_range: &DateRange,
    repo_owner: &str,
    repo_name: &str,
    branch: &str,
    tracker: Option<&RepositoryTaskTracker>,
    task_tracker_service: &dyn TaskTrackerService,
    kaiten_base: &str,
) -> Result<String, askama::Error> {
    let task_url_template =
        tracker.map(|t| format!("{}{}", kaiten_base, t.path_to_card));
    let extract_pattern = tracker.map(|t| t.extract_pattern_regexp.as_str());

    let commits = &report.commits;
    let prs = &report.pull_requests;

    // ── Aggregate stats ───────────────────────────────────────────────────────

    let total_additions: i64 = commits.iter().map(|c| c.additions).sum();
    let total_deletions: i64 = commits.iter().map(|c| c.deletions).sum();
    let total_changed_files: i64 = commits.iter().filter_map(|c| c.changed_files).sum();
    let net_changes = total_additions - total_deletions;
    let net_changes_str = if net_changes >= 0 {
        format!("+{}", net_changes)
    } else {
        net_changes.to_string()
    };
    let avg_commit_size = if commits.is_empty() {
        0
    } else {
        total_additions / commits.len() as i64
    };

    let active_days: HashSet<String> = commits
        .iter()
        .map(|c| c.authored_at.format("%Y-%m-%d").to_string())
        .collect();

    let unique_authors: HashSet<String> = commits
        .iter()
        .filter_map(|c| {
            c.author
                .as_ref()
                .and_then(|a| a.login.clone().or_else(|| a.name.clone()))
        })
        .collect();

    // ── PR summary ────────────────────────────────────────────────────────────

    let (
        pr_merged,
        pr_open,
        pr_closed,
        pr_merge_rate,
        pr_avg_merge_time,
    ) = compute_pr_summary(prs);

    // ── Activity chart ────────────────────────────────────────────────────────

    let activity = build_activity(commits);

    // ── Tasks ─────────────────────────────────────────────────────────────────

    let tasks = build_tasks(
        commits,
        prs,
        task_url_template.as_deref(),
        extract_pattern,
        task_tracker_service,
    );
    let tasks_count = tasks.len();

    // ── Contributors ──────────────────────────────────────────────────────────

    let contributors = build_contributors(commits);

    // ── PR rows ───────────────────────────────────────────────────────────────

    let pull_requests = build_pr_rows(
        prs,
        task_url_template.as_deref(),
        extract_pattern,
        task_tracker_service,
    );

    // ── Commit rows ───────────────────────────────────────────────────────────

    let commit_rows = build_commit_rows(
        commits,
        task_url_template.as_deref(),
        extract_pattern,
        task_tracker_service,
    );

    // ── Period ────────────────────────────────────────────────────────────────

    let period = format!(
        "{} — {}",
        date_range.since.format("%d %b %Y"),
        date_range.until.format("%d %b %Y")
    );

    let title = match author {
        Some(a) => format!("Персональный отчёт: {}", html_escape(a)),
        None => "Отчёт по репозиторию".to_string(),
    };

    let is_personal = author.is_some();

    // ── Render ────────────────────────────────────────────────────────────────

    FullReportTemplate {
        title,
        is_personal,
        repo_owner: html_escape(repo_owner),
        repo_name: html_escape(repo_name),
        branch: html_escape(branch),
        period,
        generated_at: chrono::Utc::now()
            .format("%d %b %Y, %H:%M UTC")
            .to_string(),

        commits_count: commits.len(),
        prs_count: prs.len(),
        total_additions,
        total_deletions,
        net_changes_str,
        total_changed_files,
        active_days: active_days.len(),
        avg_commit_size,
        authors_count: unique_authors.len(),
        tasks_count,

        pr_summary_merged: pr_merged,
        pr_summary_open: pr_open,
        pr_summary_closed: pr_closed,
        pr_summary_merge_rate: pr_merge_rate,
        pr_summary_avg_merge_time: pr_avg_merge_time,

        activity,
        tasks,
        contributors,
        pull_requests,
        commits: commit_rows,
    }
    .render()
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn compute_pr_summary(
    prs: &[VersionControlDateRangeReportPullRequest],
) -> (usize, usize, usize, String, String) {
    let merged: Vec<_> = prs.iter().filter(|p| p.merged_at.is_some()).collect();
    let open: Vec<_> = prs.iter().filter(|p| p.state == "open").collect();
    let closed: Vec<_> = prs
        .iter()
        .filter(|p| p.state == "closed" && p.merged_at.is_none())
        .collect();

    let total = merged.len() + open.len() + closed.len();
    let merge_rate = if total == 0 {
        "—".into()
    } else {
        format!("{:.0}%", merged.len() as f64 / total as f64 * 100.0)
    };

    let total_merge_secs: f64 = merged
        .iter()
        .filter_map(|p| {
            p.merged_at
                .map(|m| m.signed_duration_since(p.created_at).num_seconds() as f64)
        })
        .sum();

    let avg_merge_time = if merged.is_empty() {
        "—".into()
    } else {
        format_duration(total_merge_secs / merged.len() as f64)
    };

    (merged.len(), open.len(), closed.len(), merge_rate, avg_merge_time)
}

fn build_activity(commits: &[VersionControlDateRangeReportCommit]) -> Vec<DayActivity> {
    // key: "YYYY-MM-DD" for sorting; value: (display label, count)
    let mut day_map: HashMap<String, (String, usize)> = HashMap::new();
    for c in commits {
        let key = c.authored_at.format("%Y-%m-%d").to_string();
        let label = c.authored_at.format("%d %b").to_string();
        let entry = day_map.entry(key).or_insert((label, 0));
        entry.1 += 1;
    }

    let mut days: Vec<(String, String, usize)> = day_map
        .into_iter()
        .map(|(k, (l, n))| (k, l, n))
        .collect();
    days.sort_by(|a, b| a.0.cmp(&b.0));

    let max = days.iter().map(|(_, _, n)| *n).max().unwrap_or(1);
    const BAR_MAX_PX: usize = 60;

    days.into_iter()
        .map(|(_, label, count)| DayActivity {
            day: label,
            commits: count,
            // Ensure a minimum 4 px so even single-commit days are visible.
            bar_px: ((count * BAR_MAX_PX / max).max(4)),
        })
        .collect()
}

fn build_tasks(
    commits: &[VersionControlDateRangeReportCommit],
    prs: &[VersionControlDateRangeReportPullRequest],
    task_url_template: Option<&str>,
    extract_pattern: Option<&str>,
    task_tracker_service: &dyn TaskTrackerService,
) -> Vec<TaskItem> {
    let (Some(url_template), Some(pattern)) = (task_url_template, extract_pattern) else {
        return vec![];
    };

    let mut texts: Vec<&str> = commits.iter().map(|c| c.message.as_str()).collect();
    texts.extend(prs.iter().map(|p| p.title.as_str()));

    let mut seen: HashSet<u64> = HashSet::new();
    let mut result = Vec::new();

    for text in texts {
        for (matched, task_id) in
            task_tracker_service.extract_all_matches_with_pattern(text, pattern)
        {
            if seen.insert(task_id.0) {
                let url = url_template.replace("{id}", &task_id.0.to_string());
                result.push(TaskItem {
                    url,
                    label: html_escape(&matched),
                });
            }
        }
    }

    result
}

fn build_contributors(commits: &[VersionControlDateRangeReportCommit]) -> Vec<ContributorRow> {
    #[derive(Default)]
    struct Stats {
        commits: usize,
        additions: i64,
        deletions: i64,
    }

    let mut map: HashMap<String, Stats> = HashMap::new();
    for c in commits {
        let name = c
            .author
            .as_ref()
            .and_then(|a| a.login.clone().or_else(|| a.name.clone()))
            .unwrap_or_else(|| "unknown".into());
        let e = map.entry(name).or_default();
        e.commits += 1;
        e.additions += c.additions;
        e.deletions += c.deletions;
    }

    let mut top: Vec<(String, Stats)> = map.into_iter().collect();
    top.sort_by(|a, b| b.1.commits.cmp(&a.1.commits));

    let max_commits = top.first().map(|(_, s)| s.commits).unwrap_or(1);

    top.into_iter()
        .map(|(name, stats)| ContributorRow {
            name: html_escape(&name),
            pct: stats.commits * 100 / max_commits,
            commits: stats.commits,
            additions: stats.additions,
            deletions: stats.deletions,
        })
        .collect()
}

fn build_pr_rows(
    prs: &[VersionControlDateRangeReportPullRequest],
    task_url_template: Option<&str>,
    extract_pattern: Option<&str>,
    task_tracker_service: &dyn TaskTrackerService,
) -> Vec<PrRow> {
    let mut sorted = prs.to_vec();
    sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    sorted
        .iter()
        .map(|pr| {
            let (state_icon, state_class, state_label) =
                match (pr.merged_at.is_some(), pr.state.as_str()) {
                    (true, _) => ("✅", "merged", "Merged"),
                    (_, "open") => ("🟢", "open", "Open"),
                    _ => ("🔴", "closed", "Closed"),
                };

            PrRow {
                number: pr.number,
                title_html: linkify_task(
                    &pr.title,
                    task_url_template,
                    extract_pattern,
                    task_tracker_service,
                ),
                state_icon,
                state_class,
                state_label,
                author: html_escape(pr.author.as_deref().unwrap_or("—")),
                additions: pr.additions,
                deletions: pr.deletions,
                changed_files: pr.changed_files,
                created_at: pr.created_at.format("%d %b %Y").to_string(),
                merged_at: pr
                    .merged_at
                    .map(|m| m.format("%d %b %Y").to_string())
                    .unwrap_or_default(),
            }
        })
        .collect()
}

fn build_commit_rows(
    commits: &[VersionControlDateRangeReportCommit],
    task_url_template: Option<&str>,
    extract_pattern: Option<&str>,
    task_tracker_service: &dyn TaskTrackerService,
) -> Vec<CommitRow> {
    let mut sorted = commits.to_vec();
    sorted.sort_by(|a, b| b.authored_at.cmp(&a.authored_at));

    sorted
        .iter()
        .map(|c| {
            let sha = c.sha[..7.min(c.sha.len())].to_string();
            let msg_line = c.message.lines().next().unwrap_or(&c.message);

            CommitRow {
                sha,
                message_html: linkify_task(
                    msg_line,
                    task_url_template,
                    extract_pattern,
                    task_tracker_service,
                ),
                author: html_escape(
                    c.author
                        .as_ref()
                        .and_then(|a| a.login.as_deref().or(a.name.as_deref()))
                        .unwrap_or("—"),
                ),
                additions: c.additions,
                deletions: c.deletions,
                changed_files: c
                    .changed_files
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "—".into()),
                date: c.authored_at.format("%d %b %Y, %H:%M").to_string(),
            }
        })
        .collect()
}

/// Replaces the first task match in `text` with an HTML `<a>` link.
/// Falls back to plain HTML-escaped text when no pattern / match found.
fn linkify_task(
    text: &str,
    task_url_template: Option<&str>,
    extract_pattern: Option<&str>,
    task_tracker_service: &dyn TaskTrackerService,
) -> String {
    let (Some(url_template), Some(pattern)) = (task_url_template, extract_pattern) else {
        return html_escape(text);
    };

    let Some((matched, task_id)) =
        task_tracker_service.extract_match_with_pattern(text, pattern)
    else {
        return html_escape(text);
    };

    let url = url_template.replace("{id}", &task_id.0.to_string());

    if let Some(pos) = text.find(&matched) {
        let before = &text[..pos];
        let after = &text[pos + matched.len()..];
        format!(
            "{}<a href=\"{}\" class=\"task-link\">{}</a>{}",
            html_escape(before),
            html_escape(&url),
            html_escape(&matched),
            html_escape(after),
        )
    } else {
        html_escape(text)
    }
}

fn format_duration(seconds: f64) -> String {
    match seconds {
        s if s < 60.0 => format!("{:.0} сек", s),
        s if s < 3600.0 => format!("{:.0} мин", s / 60.0),
        s if s < 86400.0 => format!("{:.1} ч", s / 3600.0),
        s => format!("{:.1} дн", s / 86400.0),
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
