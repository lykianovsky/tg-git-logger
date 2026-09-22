//! Сборка представления и рендер страницы состояния качества.
//!
//! Считает всё здесь: доли, высоты столбиков, подписи — в шаблоне остаётся только вывод.

use crate::domain::repository::entities::repository::Repository;
use crate::domain::test_run::entities::test_run::TestRun;
use crate::domain::test_run::repositories::test_run_repository::TestFailureCount;
use crate::domain::test_run::value_objects::test_run_status::TestRunStatus;
use askama::Template;
use rust_i18n::t;

const PERCENT: f64 = 100.0;
const MILLISECONDS_IN_SECOND: u64 = 1000;
const SECONDS_IN_MINUTE: u64 = 60;
/// Высота столбика тренда в пикселях
const MAX_BAR_HEIGHT: u64 = 90;
const MIN_BAR_HEIGHT: u64 = 6;
/// Сколько тестов показываем в списке самых частых падений
const TOP_FAILURES_LIMIT: usize = 10;

pub struct RunBar {
    /// Высота столбика в пикселях
    pub height: u64,
    pub css_class: &'static str,
    /// Подпись при наведении
    pub hint: String,
}

pub struct TopFailureRow {
    pub title: String,
    pub file: String,
    pub project: String,
    pub count: u32,
}

#[derive(Template)]
#[template(path = "report/quality_dashboard.html", escape = "none")]
pub struct QualityDashboardTemplate {
    pub repo_owner: String,
    pub repo_name: String,
    pub generated_label: String,

    pub runs_count: usize,
    pub green_share: u32,
    pub green_share_label: String,
    pub failed_runs: usize,
    pub average_duration: String,
    pub last_status_label: String,
    pub last_status_class: String,

    pub bars: Vec<RunBar>,
    pub top_failures: Vec<TopFailureRow>,

    pub label_runs: String,
    pub label_green_share: String,
    pub label_failed_runs: String,
    pub label_duration: String,
    pub title_trend: String,
    pub title_top_failures: String,
    pub empty_label: String,
    pub times_label: String,
}

pub fn build_html_dashboard(
    repository: &Repository,
    runs: &[TestRun],
    failures: &[TestFailureCount],
) -> Result<String, askama::Error> {
    // История приходит от свежих к старым, а на графике время идёт слева направо
    let ordered: Vec<&TestRun> = runs.iter().rev().collect();
    let finished: Vec<&&TestRun> = ordered.iter().filter(|run| !run.is_active()).collect();
    let green = finished
        .iter()
        .filter(|run| run.status == TestRunStatus::Passed)
        .count();
    let failed = finished
        .iter()
        .filter(|run| run.status == TestRunStatus::Failed)
        .count();
    let green_share = match finished.is_empty() {
        true => 0,
        false => ((green as f64 / finished.len() as f64) * PERCENT).round() as u32,
    };

    let (last_status_label, last_status_class) = match runs.first() {
        Some(run) => status_view(run.status),
        None => (
            t!("report.test_run.status.unknown").to_string(),
            "status-unknown",
        ),
    };

    QualityDashboardTemplate {
        repo_owner: html_escape(&repository.owner),
        repo_name: html_escape(&repository.name),
        generated_label: t!(
            "report.test_run.generated_at",
            value = chrono::Utc::now().format("%d.%m.%Y, %H:%M UTC").to_string()
        )
        .to_string(),

        runs_count: finished.len(),
        green_share,
        green_share_label: format!("{green_share}%"),
        failed_runs: failed,
        average_duration: average_duration(&finished),
        last_status_label,
        last_status_class: last_status_class.to_string(),

        bars: build_bars(&ordered),
        top_failures: build_top_failures(failures),

        label_runs: t!("report.dashboard.runs").to_string(),
        label_green_share: t!("report.dashboard.green_share").to_string(),
        label_failed_runs: t!("report.dashboard.failed_runs").to_string(),
        label_duration: t!("report.dashboard.duration").to_string(),
        title_trend: t!("report.dashboard.trend").to_string(),
        title_top_failures: t!("report.dashboard.top_failures").to_string(),
        empty_label: t!("report.dashboard.empty").to_string(),
        times_label: t!("report.dashboard.times").to_string(),
    }
    .render()
}

/// Столбик — один прогон: высота по длительности, цвет по итогу
fn build_bars(runs: &[&TestRun]) -> Vec<RunBar> {
    let longest = runs
        .iter()
        .filter_map(|run| run.totals.map(|totals| totals.duration_ms))
        .max()
        .unwrap_or(0);

    runs.iter()
        .map(|run| {
            let duration_ms = run.totals.map(|totals| totals.duration_ms).unwrap_or(0);
            let height = match longest {
                0 => MIN_BAR_HEIGHT,
                longest => MIN_BAR_HEIGHT.max(duration_ms * MAX_BAR_HEIGHT / longest.max(1)),
            };
            let totals = run.totals.unwrap_or_default();

            RunBar {
                height,
                css_class: bar_class(run.status),
                hint: html_escape(&format!(
                    "{} · {} · {}",
                    run.git_ref,
                    format_duration(duration_ms),
                    t!(
                        "telegram_bot.test_run.totals_value",
                        passed = totals.passed,
                        failed = totals.failed,
                        flaky = totals.flaky,
                        skipped = totals.skipped
                    )
                )),
            }
        })
        .collect()
}

fn build_top_failures(failures: &[TestFailureCount]) -> Vec<TopFailureRow> {
    failures
        .iter()
        .take(TOP_FAILURES_LIMIT)
        .map(|failure| TopFailureRow {
            title: html_escape(&failure.title),
            file: html_escape(&failure.file),
            project: html_escape(&failure.project),
            count: failure.count,
        })
        .collect()
}

fn average_duration(runs: &[&&TestRun]) -> String {
    let durations: Vec<u64> = runs
        .iter()
        .filter_map(|run| run.totals.map(|totals| totals.duration_ms))
        .filter(|duration| *duration > 0)
        .collect();

    if durations.is_empty() {
        return "—".to_string();
    }

    format_duration(durations.iter().sum::<u64>() / durations.len() as u64)
}

fn bar_class(status: TestRunStatus) -> &'static str {
    match status {
        TestRunStatus::Passed => "bar-passed",
        TestRunStatus::Failed => "bar-failed",
        TestRunStatus::Queued | TestRunStatus::Running => "bar-running",
        _ => "bar-unknown",
    }
}

fn status_view(status: TestRunStatus) -> (String, &'static str) {
    let class = match status {
        TestRunStatus::Passed => "status-passed",
        TestRunStatus::Failed => "status-failed",
        TestRunStatus::Queued | TestRunStatus::Running => "status-running",
        _ => "status-unknown",
    };
    let key = format!("report.test_run.status.{}", status.as_str());

    (t!(&key).to_string(), class)
}

fn format_duration(duration_ms: u64) -> String {
    let seconds = duration_ms / MILLISECONDS_IN_SECOND;

    if seconds < SECONDS_IN_MINUTE {
        return t!("report.test_run.duration_seconds", value = seconds).to_string();
    }

    t!(
        "report.test_run.duration_minutes",
        minutes = seconds / SECONDS_IN_MINUTE,
        seconds = seconds % SECONDS_IN_MINUTE
    )
    .to_string()
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
