//! View-model construction and HTML report rendering.
//!
//! All data transformations (stats, task extraction, HTML escaping) happen here
//! in Rust, before being passed to the Askama template. The template itself
//! contains no logic — it only iterates and renders pre-computed values.
//!
//! Two separate templates exist:
//! - `PersonalReportTemplate` → `report/personal_report.html`
//! - `RepoReportTemplate`     → `report/repo_report.html`

use crate::domain::repository::entities::repository_task_tracker::RepositoryTaskTracker;
use crate::domain::shared::date::range::DateRange;
use crate::domain::task::services::task_tracker_service::TaskTrackerService;
use crate::domain::version_control::value_objects::report::{
    VersionControlDateRangeReport, VersionControlDateRangeReportCommit,
    VersionControlDateRangeReportPullRequest,
};
use askama::Template;
use rust_i18n::t;
use std::collections::{HashMap, HashSet};

// ── Shared view models ────────────────────────────────────────────────────────

pub struct DayActivity {
    /// Display label, e.g. "12 Jan"
    pub day: String,
    pub commits: usize,
    /// Pluralized word matching `commits`, e.g. "коммит", "коммита", "коммитов".
    pub commits_word: String,
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
    /// Pluralized word matching `commits`.
    pub commits_word: String,
    pub additions: i64,
    pub deletions: i64,
    /// Bar width in percent (0–100), relative to the top contributor.
    pub pct: usize,
    /// Deduplicated tasks extracted from this contributor's commits and PRs.
    pub tasks: Vec<TaskItem>,
    /// Pluralized word matching `tasks.len()`.
    pub tasks_word: String,
}

pub struct PrRow {
    pub number: i64,
    /// Pre-computed safe HTML (may contain `<a>` task links).
    pub title_html: String,
    pub state_icon: &'static str,
    pub state_class: &'static str,
    pub state_label: String,
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

// ── Personal-report-specific view models ─────────────────────────────────────

/// Commit count per day-of-week (Mon–Sun).
pub struct DowBar {
    pub day: String,
    pub commits: usize,
    /// Pluralized word matching `commits`.
    pub commits_word: String,
    /// Bar height in percent (0–100), relative to the busiest day.
    pub pct: usize,
}

/// Commit distribution across four time-of-day buckets.
pub struct TimeOfDayBucket {
    pub icon: &'static str,
    pub label: String,
    pub commits: usize,
    /// Width percent for the progress bar (0–100).
    pub pct: usize,
}

// ── Askama templates ──────────────────────────────────────────────────────────

/// Personal report template — data specific to a single author.
#[derive(Template)]
#[template(path = "report/personal_report.html", escape = "none")]
pub struct PersonalReportTemplate {
    // Header
    pub author_name: String,
    /// First letter of author login, upper-cased — used as avatar placeholder.
    pub avatar_letter: String,
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
    pub tasks_count: usize,

    // PR counters (personal)
    pub pr_merged: usize,
    pub pr_open: usize,
    pub pr_closed: usize,
    pub pr_merge_rate: String,

    // Streak
    /// Longest consecutive-day streak in the period.
    pub longest_streak: usize,
    /// Day with the most commits, e.g. "Пт, 24 Jan — 8 коммитов"
    pub best_day: String,

    // Sections
    pub activity: Vec<DayActivity>,
    pub dow_activity: Vec<DowBar>,
    pub time_buckets: Vec<TimeOfDayBucket>,
    pub tasks: Vec<TaskItem>,
    pub pull_requests: Vec<PrRow>,
    pub commits: Vec<CommitRow>,
}

/// Repository-wide report template — aggregates across all authors.
#[derive(Template)]
#[template(path = "report/repo_report.html", escape = "none")]
pub struct RepoReportTemplate {
    // Header
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
    pub authors_count: usize,
    pub avg_commit_size: i64,
    pub tasks_count: usize,

    // PR summary
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
///
/// When `author` is `Some`, renders a personal report filtered to that author.
/// When `author` is `None`, renders a repository-wide report.
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
    let task_url_template = tracker.map(|t| format!("{}{}", kaiten_base, t.path_to_card));
    let extract_pattern = tracker.map(|t| t.extract_pattern_regexp.as_str());

    let period = format!(
        "{} — {}",
        date_range.since.format("%d %b %Y"),
        date_range.until.format("%d %b %Y")
    );
    let generated_at = chrono::Utc::now().format("%d %b %Y, %H:%M UTC").to_string();

    match author {
        Some(login) => build_personal_report(
            report,
            login,
            period,
            generated_at,
            repo_owner,
            repo_name,
            branch,
            task_url_template.as_deref(),
            extract_pattern,
            task_tracker_service,
        ),
        None => build_repo_report(
            report,
            period,
            generated_at,
            repo_owner,
            repo_name,
            branch,
            task_url_template.as_deref(),
            extract_pattern,
            task_tracker_service,
        ),
    }
}

// ── Personal report builder ───────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn build_personal_report(
    report: &VersionControlDateRangeReport,
    login: &str,
    period: String,
    generated_at: String,
    repo_owner: &str,
    repo_name: &str,
    branch: &str,
    task_url_template: Option<&str>,
    extract_pattern: Option<&str>,
    task_tracker_service: &dyn TaskTrackerService,
) -> Result<String, askama::Error> {
    let commits = &report.commits;
    let prs = &report.pull_requests;

    let total_additions: i64 = commits.iter().map(|c| c.additions).sum();
    let total_deletions: i64 = commits.iter().map(|c| c.deletions).sum();
    let total_changed_files: i64 = commits.iter().filter_map(|c| c.changed_files).sum();
    let net = total_additions - total_deletions;
    let net_changes_str = if net >= 0 {
        format!("+{}", net)
    } else {
        net.to_string()
    };
    let avg_commit_size = if commits.is_empty() {
        0
    } else {
        total_additions / commits.len() as i64
    };

    let active_days_set: HashSet<String> = commits
        .iter()
        .map(|c| c.authored_at.format("%Y-%m-%d").to_string())
        .collect();

    let (pr_merged, pr_open, pr_closed, pr_merge_rate, _) = compute_pr_summary(prs);

    let activity = build_activity(commits);
    let dow_activity = build_dow_activity(commits);
    let time_buckets = build_time_buckets(commits);
    let longest_streak = compute_longest_streak(commits);
    let best_day = compute_best_day(commits);

    let tasks = build_tasks(
        commits,
        prs,
        task_url_template,
        extract_pattern,
        task_tracker_service,
    );
    let tasks_count = tasks.len();

    let pull_requests = build_pr_rows(
        prs,
        task_url_template,
        extract_pattern,
        task_tracker_service,
    );
    let commit_rows = build_commit_rows(
        commits,
        task_url_template,
        extract_pattern,
        task_tracker_service,
    );

    let avatar_letter = login
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();

    PersonalReportTemplate {
        author_name: html_escape(login),
        avatar_letter,
        repo_owner: html_escape(repo_owner),
        repo_name: html_escape(repo_name),
        branch: html_escape(branch),
        period,
        generated_at,

        commits_count: commits.len(),
        prs_count: prs.len(),
        total_additions,
        total_deletions,
        net_changes_str,
        total_changed_files,
        active_days: active_days_set.len(),
        avg_commit_size,
        tasks_count,

        pr_merged,
        pr_open,
        pr_closed,
        pr_merge_rate,

        longest_streak,
        best_day,

        activity,
        dow_activity,
        time_buckets,
        tasks,
        pull_requests,
        commits: commit_rows,
    }
    .render()
}

// ── Repository report builder ─────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn build_repo_report(
    report: &VersionControlDateRangeReport,
    period: String,
    generated_at: String,
    repo_owner: &str,
    repo_name: &str,
    branch: &str,
    task_url_template: Option<&str>,
    extract_pattern: Option<&str>,
    task_tracker_service: &dyn TaskTrackerService,
) -> Result<String, askama::Error> {
    let commits = &report.commits;
    let prs = &report.pull_requests;

    let total_additions: i64 = commits.iter().map(|c| c.additions).sum();
    let total_deletions: i64 = commits.iter().map(|c| c.deletions).sum();
    let total_changed_files: i64 = commits.iter().filter_map(|c| c.changed_files).sum();
    let net = total_additions - total_deletions;
    let net_changes_str = if net >= 0 {
        format!("+{}", net)
    } else {
        net.to_string()
    };
    let avg_commit_size = if commits.is_empty() {
        0
    } else {
        total_additions / commits.len() as i64
    };

    let unique_authors: HashSet<String> = commits
        .iter()
        .filter_map(|c| {
            c.author
                .as_ref()
                .and_then(|a| a.login.clone().or_else(|| a.name.clone()))
        })
        .collect();

    let (pr_merged, pr_open, pr_closed, pr_merge_rate, pr_avg_merge_time) = compute_pr_summary(prs);

    let activity = build_activity(commits);
    let tasks = build_tasks(
        commits,
        prs,
        task_url_template,
        extract_pattern,
        task_tracker_service,
    );
    let tasks_count = tasks.len();

    let contributors = build_contributors(
        commits,
        prs,
        task_url_template,
        extract_pattern,
        task_tracker_service,
    );

    let pull_requests = build_pr_rows(
        prs,
        task_url_template,
        extract_pattern,
        task_tracker_service,
    );
    let commit_rows = build_commit_rows(
        commits,
        task_url_template,
        extract_pattern,
        task_tracker_service,
    );

    RepoReportTemplate {
        repo_owner: html_escape(repo_owner),
        repo_name: html_escape(repo_name),
        branch: html_escape(branch),
        period,
        generated_at,

        commits_count: commits.len(),
        prs_count: prs.len(),
        total_additions,
        total_deletions,
        net_changes_str,
        total_changed_files,
        authors_count: unique_authors.len(),
        avg_commit_size,
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

// ── Shared private helpers ────────────────────────────────────────────────────

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

    (
        merged.len(),
        open.len(),
        closed.len(),
        merge_rate,
        avg_merge_time,
    )
}

fn build_activity(commits: &[VersionControlDateRangeReportCommit]) -> Vec<DayActivity> {
    let mut day_map: HashMap<String, (String, usize)> = HashMap::new();
    for c in commits {
        let key = c.authored_at.format("%Y-%m-%d").to_string();
        let label = c.authored_at.format("%d %b").to_string();
        let entry = day_map.entry(key).or_insert((label, 0));
        entry.1 += 1;
    }

    let mut days: Vec<(String, String, usize)> =
        day_map.into_iter().map(|(k, (l, n))| (k, l, n)).collect();
    days.sort_by(|a, b| a.0.cmp(&b.0));

    let max = days.iter().map(|(_, _, n)| *n).max().unwrap_or(1);
    const BAR_MAX_PX: usize = 60;

    days.into_iter()
        .map(|(_, label, count)| DayActivity {
            day: label,
            commits: count,
            commits_word: plural_commits(count),
            bar_px: (count * BAR_MAX_PX / max).max(4),
        })
        .collect()
}

fn build_dow_activity(commits: &[VersionControlDateRangeReportCommit]) -> Vec<DowBar> {
    let days = [
        t!("report.renderer.days.0").to_string(),
        t!("report.renderer.days.1").to_string(),
        t!("report.renderer.days.2").to_string(),
        t!("report.renderer.days.3").to_string(),
        t!("report.renderer.days.4").to_string(),
        t!("report.renderer.days.5").to_string(),
        t!("report.renderer.days.6").to_string(),
    ];
    let mut counts = [0usize; 7];

    for c in commits {
        // chrono weekday: Mon=0 … Sun=6
        let idx = c.authored_at.weekday().num_days_from_monday() as usize;
        counts[idx] += 1;
    }

    let max = *counts.iter().max().unwrap_or(&1);
    let max = max.max(1);

    days.into_iter()
        .enumerate()
        .map(|(i, day)| DowBar {
            day,
            commits: counts[i],
            commits_word: plural_commits(counts[i]),
            pct: counts[i] * 100 / max,
        })
        .collect()
}

fn build_time_buckets(commits: &[VersionControlDateRangeReportCommit]) -> Vec<TimeOfDayBucket> {
    // Buckets: night 0-5, morning 6-11, day 12-17, evening 18-23
    let mut counts = [0usize; 4];
    for c in commits {
        let h = c.authored_at.hour();
        let idx = match h {
            0..=5 => 0,
            6..=11 => 1,
            12..=17 => 2,
            _ => 3,
        };
        counts[idx] += 1;
    }

    let total = counts.iter().sum::<usize>().max(1);
    let icons = ["🌙", "🌅", "☀️", "🌆"];
    let labels = [
        t!("report.renderer.time_night").to_string(),
        t!("report.renderer.time_morning").to_string(),
        t!("report.renderer.time_day").to_string(),
        t!("report.renderer.time_evening").to_string(),
    ];

    icons
        .iter()
        .zip(labels.into_iter())
        .enumerate()
        .map(|(i, (&icon, label))| TimeOfDayBucket {
            icon,
            label,
            commits: counts[i],
            pct: counts[i] * 100 / total,
        })
        .collect()
}

fn compute_longest_streak(commits: &[VersionControlDateRangeReportCommit]) -> usize {
    let mut days: Vec<chrono::NaiveDate> = commits
        .iter()
        .map(|c| c.authored_at.date_naive())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    days.sort();

    if days.is_empty() {
        return 0;
    }

    let mut max_streak = 1usize;
    let mut cur = 1usize;

    for i in 1..days.len() {
        if days[i].signed_duration_since(days[i - 1]).num_days() == 1 {
            cur += 1;
            max_streak = max_streak.max(cur);
        } else {
            cur = 1;
        }
    }

    max_streak
}

fn compute_best_day(commits: &[VersionControlDateRangeReportCommit]) -> String {
    let mut day_map: HashMap<String, (String, usize)> = HashMap::new();
    for c in commits {
        let key = c.authored_at.format("%Y-%m-%d").to_string();
        let label = c.authored_at.format("%d %b").to_string();
        let e = day_map.entry(key).or_insert((label, 0));
        e.1 += 1;
    }

    day_map
        .into_iter()
        .max_by_key(|(_, (_, n))| *n)
        .map(|(_, (label, n))| {
            let word = plural_commits(n);
            t!("report.renderer.best_day", label = label, count = n, word = word).to_string()
        })
        .unwrap_or_else(|| "—".into())
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
                result.push(TaskItem {
                    url: url_template.replace("{id}", &task_id.0.to_string()),
                    label: html_escape(&matched),
                });
            }
        }
    }

    result
}

fn build_contributors(
    commits: &[VersionControlDateRangeReportCommit],
    prs: &[VersionControlDateRangeReportPullRequest],
    task_url_template: Option<&str>,
    extract_pattern: Option<&str>,
    task_tracker_service: &dyn TaskTrackerService,
) -> Vec<ContributorRow> {
    #[derive(Default)]
    struct Stats {
        commits: usize,
        additions: i64,
        deletions: i64,
        commit_messages: Vec<String>,
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
        e.commit_messages.push(c.message.clone());
    }

    let mut top: Vec<(String, Stats)> = map.into_iter().collect();
    top.sort_by(|a, b| b.1.commits.cmp(&a.1.commits));

    let max_commits = top.first().map(|(_, s)| s.commits).unwrap_or(1);

    top.into_iter()
        .map(|(name, stats)| {
            let tasks = extract_contributor_tasks(
                &name,
                &stats.commit_messages,
                prs,
                task_url_template,
                extract_pattern,
                task_tracker_service,
            );
            let tasks_word = plural_tasks(tasks.len());
            ContributorRow {
                name: html_escape(&name),
                pct: stats.commits * 100 / max_commits,
                commits: stats.commits,
                commits_word: plural_commits(stats.commits),
                additions: stats.additions,
                deletions: stats.deletions,
                tasks,
                tasks_word,
            }
        })
        .collect()
}

fn extract_contributor_tasks(
    contributor: &str,
    commit_messages: &[String],
    prs: &[VersionControlDateRangeReportPullRequest],
    task_url_template: Option<&str>,
    extract_pattern: Option<&str>,
    task_tracker_service: &dyn TaskTrackerService,
) -> Vec<TaskItem> {
    let (Some(url_template), Some(pattern)) = (task_url_template, extract_pattern) else {
        return vec![];
    };

    let mut seen: HashSet<u64> = HashSet::new();
    let mut tasks = Vec::new();

    for msg in commit_messages {
        for (matched, task_id) in
            task_tracker_service.extract_all_matches_with_pattern(msg, pattern)
        {
            if seen.insert(task_id.0) {
                tasks.push(TaskItem {
                    url: url_template.replace("{id}", &task_id.0.to_string()),
                    label: html_escape(&matched),
                });
            }
        }
    }

    for pr in prs {
        if pr.author.as_deref() == Some(contributor) {
            for (matched, task_id) in
                task_tracker_service.extract_all_matches_with_pattern(&pr.title, pattern)
            {
                if seen.insert(task_id.0) {
                    tasks.push(TaskItem {
                        url: url_template.replace("{id}", &task_id.0.to_string()),
                        label: html_escape(&matched),
                    });
                }
            }
        }
    }

    tasks
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
            let (state_icon, state_class, state_label): (&'static str, &'static str, String) =
                match (pr.merged_at.is_some(), pr.state.as_str()) {
                    (true, _) => (
                        "✅",
                        "merged",
                        t!("report.renderer.pr_state.merged").to_string(),
                    ),
                    (_, "open") => (
                        "🟢",
                        "open",
                        t!("report.renderer.pr_state.open").to_string(),
                    ),
                    _ => (
                        "🔴",
                        "closed",
                        t!("report.renderer.pr_state.closed").to_string(),
                    ),
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
fn linkify_task(
    text: &str,
    task_url_template: Option<&str>,
    extract_pattern: Option<&str>,
    task_tracker_service: &dyn TaskTrackerService,
) -> String {
    let (Some(url_template), Some(pattern)) = (task_url_template, extract_pattern) else {
        return html_escape(text);
    };

    let Some((matched, task_id)) = task_tracker_service.extract_match_with_pattern(text, pattern)
    else {
        return html_escape(text);
    };

    let url = url_template.replace("{id}", &task_id.0.to_string());

    if let Some(pos) = text.find(&matched) {
        let before = &text[..pos];
        let after = &text[pos + matched.len()..];
        format!(
            "{}<a href=\"{}\" class=\"task-link\" target=\"_blank\" rel=\"noopener\">{}</a>{}",
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
        s if s < 60.0 => {
            t!("report.renderer.duration_seconds", value = format!("{:.0}", s))
                .to_string()
        }
        s if s < 3600.0 => {
            t!("report.renderer.duration_minutes", value = format!("{:.0}", s / 60.0))
                .to_string()
        }
        s if s < 86400.0 => {
            t!("report.renderer.duration_hours", value = format!("{:.1}", s / 3600.0))
                .to_string()
        }
        s => {
            t!("report.renderer.duration_days", value = format!("{:.1}", s / 86400.0))
                .to_string()
        }
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Returns "one" / "few" / "many" according to Russian noun pluralization rules.
fn plural_form(n: usize) -> &'static str {
    let mod10 = n % 10;
    let mod100 = n % 100;
    if mod10 == 1 && mod100 != 11 {
        "one"
    } else if (2..=4).contains(&mod10) && !(12..=14).contains(&mod100) {
        "few"
    } else {
        "many"
    }
}

fn plural_commits(n: usize) -> String {
    match plural_form(n) {
        "one" => t!("report.renderer.plural_commits.one").to_string(),
        "few" => t!("report.renderer.plural_commits.few").to_string(),
        _ => t!("report.renderer.plural_commits.many").to_string(),
    }
}

fn plural_tasks(n: usize) -> String {
    match plural_form(n) {
        "one" => t!("report.renderer.plural_tasks.one").to_string(),
        "few" => t!("report.renderer.plural_tasks.few").to_string(),
        _ => t!("report.renderer.plural_tasks.many").to_string(),
    }
}

// chrono traits needed for .hour() and .weekday()
use chrono::{Datelike, Timelike};
