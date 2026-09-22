//! Сборка представления и рендер HTML-отчёта по прогону тестов.
//!
//! Все преобразования (проценты, экранирование, группировка по файлам) делаются здесь,
//! в шаблоне остаётся только вывод готовых значений — как в отчётах по репозиторию.

use crate::application::test_run::queries::get_run_failures::response::TestFailureWithCard;
use crate::domain::repository::entities::repository::Repository;
use crate::domain::test_run::entities::test_run::{TestRun, TestRunTotals};
use crate::domain::test_run::value_objects::test_run_status::TestRunStatus;
use crate::domain::test_run::value_objects::test_run_trigger::TestRunTrigger;
use askama::Template;
use rust_i18n::t;

const SHA_SHORT_LENGTH: usize = 7;
const PERCENT: f64 = 100.0;
const MILLISECONDS_IN_SECOND: u64 = 1000;
const SECONDS_IN_MINUTE: u64 = 60;

pub struct FailureRow {
    pub project: String,
    pub title: String,
    /// Пустая строка, если текста ошибки в отчёте не было
    pub error: String,
    /// Пустая строка, если карточка по тесту ещё не заведена
    pub card_url: String,
}

pub struct FailureFile {
    pub path: String,
    pub failures: Vec<FailureRow>,
}

#[derive(Template)]
#[template(path = "report/test_run_report.html", escape = "none")]
pub struct TestRunReportTemplate {
    pub repo_owner: String,
    pub repo_name: String,
    pub git_ref: String,
    pub sha_short: String,
    pub args: String,
    pub trigger_label: String,
    pub finished_label: String,
    pub duration_label: String,
    pub run_url: String,

    pub status_label: String,
    pub status_class: String,
    pub label_total: String,
    pub label_passed: String,
    pub label_failed: String,
    pub label_flaky: String,
    pub label_skipped: String,
    pub failures_title: String,
    pub no_failures_label: String,
    pub card_exists_label: String,
    pub run_in_ci_label: String,
    pub generated_label: String,

    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub flaky: u32,
    pub skipped: u32,
    pub passed_pct: u32,
    pub failed_pct: u32,
    pub flaky_pct: u32,

    pub failures_count: usize,
    pub failure_files: Vec<FailureFile>,
}

pub fn build_html_report(
    run: &TestRun,
    repository: &Repository,
    failures: &[TestFailureWithCard],
) -> Result<String, askama::Error> {
    let totals = run.totals.unwrap_or_default();
    let (status_label, status_class) = status_view(run.status);

    TestRunReportTemplate {
        repo_owner: html_escape(&repository.owner),
        repo_name: html_escape(&repository.name),
        git_ref: html_escape(&run.git_ref),
        sha_short: run
            .sha
            .as_deref()
            .map(|sha| html_escape(&sha[..SHA_SHORT_LENGTH.min(sha.len())]))
            .unwrap_or_default(),
        args: html_escape(run.args.as_deref().unwrap_or_default()),
        trigger_label: trigger_label(run.trigger),
        finished_label: run
            .finished_at
            .map(|value| value.format("%d.%m.%Y, %H:%M UTC").to_string())
            .unwrap_or_else(|| "—".to_string()),
        duration_label: format_duration(totals.duration_ms),
        run_url: html_escape(run.run_url.as_deref().unwrap_or_default()),

        status_label,
        status_class: status_class.to_string(),
        label_total: t!("report.test_run.stats.total").to_string(),
        label_passed: t!("report.test_run.stats.passed").to_string(),
        label_failed: t!("report.test_run.stats.failed").to_string(),
        label_flaky: t!("report.test_run.stats.flaky").to_string(),
        label_skipped: t!("report.test_run.stats.skipped").to_string(),
        failures_title: t!("report.test_run.failures_title", count = failures.len()).to_string(),
        no_failures_label: t!("report.test_run.no_failures").to_string(),
        card_exists_label: t!("report.test_run.card_exists").to_string(),
        run_in_ci_label: t!("report.test_run.run_in_ci").to_string(),
        generated_label: t!(
            "report.test_run.generated_at",
            value = chrono::Utc::now().format("%d.%m.%Y, %H:%M UTC").to_string()
        )
        .to_string(),

        total: totals.total,
        passed: totals.passed,
        failed: totals.failed,
        flaky: totals.flaky,
        skipped: totals.skipped,
        passed_pct: percent(totals.passed, &totals),
        failed_pct: percent(totals.failed, &totals),
        flaky_pct: percent(totals.flaky, &totals),

        failures_count: failures.len(),
        failure_files: group_by_file(failures),
    }
    .render()
}

/// Упавшие тесты группируются по файлу: так виден отвалившийся блок целиком
fn group_by_file(failures: &[TestFailureWithCard]) -> Vec<FailureFile> {
    let mut files: Vec<FailureFile> = Vec::new();

    for item in failures {
        let row = FailureRow {
            project: html_escape(&item.failure.project),
            title: html_escape(&item.failure.title),
            error: html_escape(item.failure.error_excerpt.as_deref().unwrap_or_default()),
            card_url: item
                .card
                .as_ref()
                .map(|card| html_escape(&card.card_url))
                .unwrap_or_default(),
        };

        match files.iter_mut().find(|file| file.path == item.failure.file) {
            Some(file) => file.failures.push(row),
            None => files.push(FailureFile {
                path: html_escape(&item.failure.file),
                failures: vec![row],
            }),
        }
    }

    files
}

fn percent(value: u32, totals: &TestRunTotals) -> u32 {
    if totals.total == 0 {
        return 0;
    }

    (f64::from(value) / f64::from(totals.total) * PERCENT).round() as u32
}

fn status_view(status: TestRunStatus) -> (String, &'static str) {
    let class = match status {
        TestRunStatus::Passed => "status-passed",
        TestRunStatus::Failed => "status-failed",
        TestRunStatus::Queued | TestRunStatus::Running => "status-running",
        TestRunStatus::Cancelled | TestRunStatus::Unknown => "status-unknown",
    };

    let key = format!("report.test_run.status.{}", status.as_str());

    (t!(&key).to_string(), class)
}

fn trigger_label(trigger: TestRunTrigger) -> String {
    let key = format!("report.test_run.trigger.{}", trigger.as_str());

    t!(&key).to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_under_minute_is_seconds() {
        assert_eq!(format_duration(45_000), "45 с");
    }

    #[test]
    fn duration_over_minute_has_minutes() {
        assert_eq!(format_duration(125_000), "2 мин 5 с");
    }

    #[test]
    fn percent_of_empty_run_is_zero() {
        let totals = TestRunTotals::default();

        assert_eq!(percent(0, &totals), 0);
    }
}
