use crate::domain::test_run::entities::test_run::TestRunTotals;
use crate::domain::test_run::ports::test_runner::ParsedTestFailure;
use serde_json::Value;

/// Сколько символов ошибки кладём в список упавших: в сообщении чата всё равно больше не видно
const ERROR_EXCERPT_MAX_CHARS: usize = 300;
const MILLISECONDS_IN_SECOND: f64 = 1000.0;

/// Итоги прогона из машиночитаемого отчёта Playwright (json-репортёр).
/// Читаем через `serde_json::Value`: формат отчёта меняется от версии к версии,
/// и жёсткая схема ломала бы разбор на полях, которые нам не нужны.
#[derive(Debug, Clone)]
pub struct ParsedSummary {
    pub totals: TestRunTotals,
    pub failures: Vec<ParsedTestFailure>,
}

pub fn parse_summary(raw: &str) -> Option<ParsedSummary> {
    let root: Value = serde_json::from_str(raw).ok()?;
    let stats = root.get("stats")?;

    let passed = stats.get("expected").and_then(Value::as_u64).unwrap_or(0) as u32;
    let failed = stats.get("unexpected").and_then(Value::as_u64).unwrap_or(0) as u32;
    let flaky = stats.get("flaky").and_then(Value::as_u64).unwrap_or(0) as u32;
    let skipped = stats.get("skipped").and_then(Value::as_u64).unwrap_or(0) as u32;
    // Длительность отчёта — в секундах с дробной частью
    let duration_ms = stats
        .get("duration")
        .and_then(Value::as_f64)
        .map(|seconds| (seconds * MILLISECONDS_IN_SECOND) as u64)
        .unwrap_or(0);

    let mut failures = Vec::new();

    if let Some(suites) = root.get("suites").and_then(Value::as_array) {
        collect_failures(suites, &mut failures);
    }

    Some(ParsedSummary {
        totals: TestRunTotals {
            total: passed + failed + flaky + skipped,
            passed,
            failed,
            flaky,
            skipped,
            duration_ms,
        },
        failures,
    })
}

/// Сьюты вложены друг в друга: файл → describe → вложенный describe
fn collect_failures(suites: &[Value], failures: &mut Vec<ParsedTestFailure>) {
    for suite in suites {
        if let Some(specs) = suite.get("specs").and_then(Value::as_array) {
            for spec in specs {
                collect_spec_failures(suite, spec, failures);
            }
        }

        if let Some(nested) = suite.get("suites").and_then(Value::as_array) {
            collect_failures(nested, failures);
        }
    }
}

fn collect_spec_failures(suite: &Value, spec: &Value, failures: &mut Vec<ParsedTestFailure>) {
    // ok=true — тест прошёл, в том числе с повтора; такие в список не попадают
    if spec.get("ok").and_then(Value::as_bool).unwrap_or(true) {
        return;
    }

    let title = spec
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let file = spec
        .get("file")
        .or_else(|| suite.get("file"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let Some(tests) = spec.get("tests").and_then(Value::as_array) else {
        return;
    };

    for test in tests {
        let status = test.get("status").and_then(Value::as_str).unwrap_or("");

        if status != "unexpected" {
            continue;
        }

        let project = test
            .get("projectName")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();

        failures.push(ParsedTestFailure {
            project,
            file: file.clone(),
            title: title.clone(),
            error_excerpt: extract_error(test),
        });
    }
}

fn extract_error(test: &Value) -> Option<String> {
    let results = test.get("results").and_then(Value::as_array)?;

    results.iter().find_map(|result| {
        let message = result
            .get("error")
            .and_then(|error| error.get("message"))
            .and_then(Value::as_str)?;

        Some(shorten(message))
    })
}

/// Первая строка ошибки без ANSI-последовательностей: Playwright красит вывод
fn shorten(message: &str) -> String {
    let without_colors = strip_ansi(message);
    let first_line = without_colors
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or_default();

    if first_line.chars().count() <= ERROR_EXCERPT_MAX_CHARS {
        return first_line.to_string();
    }

    first_line
        .chars()
        .take(ERROR_EXCERPT_MAX_CHARS)
        .collect::<String>()
        + "…"
}

fn strip_ansi(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars();

    while let Some(character) = chars.next() {
        if character != '\u{1b}' {
            result.push(character);
            continue;
        }

        // Пропускаем управляющую последовательность до её завершающей буквы
        for escaped in chars.by_ref() {
            if escaped.is_ascii_alphabetic() {
                break;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAILED_REPORT: &str = r#"{
        "stats": {"duration": 12.5, "expected": 3, "unexpected": 1, "flaky": 1, "skipped": 2},
        "suites": [{
            "title": "tests/accounts/rates/rate-form.spec.ts",
            "file": "tests/accounts/rates/rate-form.spec.ts",
            "suites": [{
                "title": "Форма тарифа",
                "file": "tests/accounts/rates/rate-form.spec.ts",
                "specs": [
                    {
                        "title": "пустая форма — ошибки",
                        "ok": false,
                        "file": "tests/accounts/rates/rate-form.spec.ts",
                        "tests": [{
                            "projectName": "accounts",
                            "status": "unexpected",
                            "results": [{"error": {"message": "\u001b[31mError:\u001b[39m expect(locator).toBeVisible() failed\n\nLocator: x"}}]
                        }]
                    },
                    {
                        "title": "тариф создаётся",
                        "ok": true,
                        "file": "tests/accounts/rates/rate-form.spec.ts",
                        "tests": [{"projectName": "accounts", "status": "expected", "results": []}]
                    }
                ]
            }]
        }]
    }"#;

    #[test]
    fn reads_totals_from_stats() {
        let summary = parse_summary(FAILED_REPORT).expect("summary is parsed");

        assert_eq!(summary.totals.passed, 3);
        assert_eq!(summary.totals.failed, 1);
        assert_eq!(summary.totals.flaky, 1);
        assert_eq!(summary.totals.skipped, 2);
        assert_eq!(summary.totals.total, 7);
        assert_eq!(summary.totals.duration_ms, 12_500);
    }

    #[test]
    fn collects_only_failed_specs() {
        let summary = parse_summary(FAILED_REPORT).expect("summary is parsed");

        assert_eq!(summary.failures.len(), 1);

        let failure = &summary.failures[0];

        assert_eq!(failure.project, "accounts");
        assert_eq!(failure.title, "пустая форма — ошибки");
        assert_eq!(failure.file, "tests/accounts/rates/rate-form.spec.ts");
    }

    #[test]
    fn error_excerpt_is_first_line_without_colors() {
        let summary = parse_summary(FAILED_REPORT).expect("summary is parsed");
        let excerpt = summary.failures[0]
            .error_excerpt
            .clone()
            .expect("error is present");

        assert_eq!(excerpt, "Error: expect(locator).toBeVisible() failed");
    }

    #[test]
    fn report_without_failures_has_empty_list() {
        let raw = r#"{"stats": {"duration": 1.0, "expected": 2, "unexpected": 0, "flaky": 0, "skipped": 0}, "suites": []}"#;
        let summary = parse_summary(raw).expect("summary is parsed");

        assert!(summary.failures.is_empty());
        assert_eq!(summary.totals.total, 2);
    }

    #[test]
    fn broken_json_is_rejected() {
        assert!(parse_summary("not a json").is_none());
    }

    /// Отчёт прогона, где все тесты пропущены: так выглядит сборка, остановленная до тестов
    #[test]
    fn skipped_report_has_no_failures() {
        let raw = r#"{
            "stats": {"duration": 289.5, "expected": 0, "unexpected": 0, "flaky": 0, "skipped": 385},
            "suites": [{
                "title": "tests/landing/auth/login.spec.ts",
                "file": "tests/landing/auth/login.spec.ts",
                "specs": [{
                    "title": "вход по паролю",
                    "ok": true,
                    "file": "tests/landing/auth/login.spec.ts",
                    "tests": [{"projectName": "landing", "status": "skipped", "results": []}]
                }]
            }]
        }"#;
        let summary = parse_summary(raw).expect("summary is parsed");

        assert_eq!(summary.totals.total, 385);
        assert_eq!(summary.totals.skipped, 385);
        assert!(summary.failures.is_empty());
    }
}
