use crate::application::version_control::queries::build_report::command::{
    BuildVersionControlDateRangeReportExecutorCommand,
    BuildVersionControlDateRangeReportExecutorCommandForWho,
};
use crate::application::version_control::queries::build_report::error::BuildVersionControlDateRangeReportExecutorError;
use crate::application::version_control::queries::build_report::response::BuildVersionControlDateRangeReportExecutorResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::shared::date::range::DateRange;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::repositories::user_vc_accounts_repository::UserVersionControlAccountsRepository;
use crate::domain::version_control::ports::version_control_client::{
    VersionControlClient, VersionControlClientDateRangeReportError,
};
use crate::domain::version_control::value_objects::report::{
    VersionControlDateRangeReport, VersionControlDateRangeReportCommit,
    VersionControlDateRangeReportPullRequest,
};
use crate::utils::builder::message::MessageBuilder;
use crate::utils::security::crypto::reversible::ReversibleCipher;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

const COMMITS_LIMIT: usize = 5;
const PRS_LIMIT: usize = 5;
const TOP_COMMITS: usize = 3;
const TOP_CONTRIBUTORS: usize = 5;
const DIVIDER: &str = "━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n";

struct PrStats<'a> {
    merged: Vec<&'a VersionControlDateRangeReportPullRequest>,
    open: Vec<&'a VersionControlDateRangeReportPullRequest>,
    closed: Vec<&'a VersionControlDateRangeReportPullRequest>,
}

// TODO: ВАЙБ-КОД
impl<'a> PrStats<'a> {
    fn from(prs: &'a [VersionControlDateRangeReportPullRequest]) -> Self {
        Self {
            merged: prs.iter().filter(|p| p.merged_at.is_some()).collect(),
            open: prs.iter().filter(|p| p.state == "open").collect(),
            closed: prs
                .iter()
                .filter(|p| p.state == "closed" && p.merged_at.is_none())
                .collect(),
        }
    }

    fn merge_rate(&self) -> String {
        let total = self.merged.len() + self.open.len() + self.closed.len();
        if total == 0 {
            return "—".into();
        }
        format!("{:.0}%", self.merged.len() as f64 / total as f64 * 100.0)
    }

    fn avg_merge_time(&self) -> Option<String> {
        let times: Vec<f64> = self
            .merged
            .iter()
            .filter_map(|p| {
                p.merged_at
                    .map(|m| m.signed_duration_since(p.created_at).num_seconds() as f64)
            })
            .collect();

        if times.is_empty() {
            return None;
        }

        let avg = times.iter().sum::<f64>() / times.len() as f64;
        Some(match avg {
            s if s < 60.0 => format!("{:.0} сек", s),
            s if s < 3600.0 => format!("{:.0} мин", s / 60.0),
            s if s < 86400.0 => format!("{:.1} ч", s / 3600.0),
            s => format!("{:.1} дн", s / 86400.0),
        })
    }
}

struct ContribStats {
    commits: usize,
    additions: i64,
    deletions: i64,
}

pub struct BuildVersionControlDateRangeReportExecutor {
    pub reversible_cipher: Arc<ReversibleCipher>,
    pub user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    pub user_version_control_service_repo: Arc<dyn UserVersionControlAccountsRepository>,
    pub version_control_client: Arc<dyn VersionControlClient>,
}

impl BuildVersionControlDateRangeReportExecutor {
    pub fn friendly_error_message(
        &self,
        error: &BuildVersionControlDateRangeReportExecutorError,
    ) -> String {
        tracing::error!("{error}");

        match error {
            BuildVersionControlDateRangeReportExecutorError::VersionControlClientDateRangeReportError(
                VersionControlClientDateRangeReportError::Unauthorized(reason)
            ) => {
                format!("🔐 Нет доступа к репозиторию.\nПричина: {}", reason)
            }
            BuildVersionControlDateRangeReportExecutorError::VersionControlClientDateRangeReportError(
                VersionControlClientDateRangeReportError::Transport(reason)
            ) => {
                format!("🌐 Ошибка соединения: {}", reason)
            }
            BuildVersionControlDateRangeReportExecutorError::FindSocialServiceByIdError(..) => {
                "🔐 Вы должны пройти регистрацию".to_string()
            }
            _ => {
                "❌ Неизвестная ошибка".to_string()
            }
        }
    }

    // TODO: ВАЙБ-КОД
    fn render(
        &self,
        report: &VersionControlDateRangeReport,
        author: &Option<String>,
        date_range: &DateRange,
    ) -> String {
        if let Some(author) = &author {
            return self.render_for_author(report, author, date_range);
        }

        self.render_for_repository(report, date_range)
    }

    fn render_for_author(
        &self,
        report: &VersionControlDateRangeReport,
        author: &str,
        date_range: &DateRange,
    ) -> String {
        let mut b = MessageBuilder::new();
        let commits = &report.commits;
        let prs = &report.pull_requests;

        b = self.render_header(
            &mut b,
            &format!(
                "👤 <b>Персональный отчёт:</b> <b>{}</b>",
                MessageBuilder::escape_html(author)
            ),
            date_range,
        );

        if commits.is_empty() && prs.is_empty() {
            return Self::render_empty(b);
        }

        let pr_stats = PrStats::from(prs);
        let total_additions: i64 = commits.iter().map(|c| c.additions).sum();
        let total_deletions: i64 = commits.iter().map(|c| c.deletions).sum();
        let active_days: HashSet<String> = commits
            .iter()
            .map(|c| c.authored_at.format("%Y-%m-%d").to_string())
            .collect();

        b = self.render_summary(
            b,
            commits.len(),
            prs.len(),
            Some(active_days.len()),
            None,
            commits.iter().filter_map(|c| c.changed_files).sum(),
        );

        b = self.render_code_stats(
            b,
            total_additions,
            total_deletions,
            Some(total_additions / commits.len().max(1) as i64),
        );

        b = self.render_activity(b, commits);

        b = self.render_pr_block(b, &pr_stats, prs, true);

        b = self.render_recent_commits(b, commits, false);

        b = self.render_recent_prs(b, prs);

        b = b.raw(DIVIDER);

        b = self.render_top_commits(b, commits);

        b.build()
    }

    fn render_for_repository(
        &self,
        report: &VersionControlDateRangeReport,
        date_range: &DateRange,
    ) -> String {
        let mut b = MessageBuilder::new();
        let commits = &report.commits;
        let prs = &report.pull_requests;

        b = self.render_header(&mut b, "🏢 <b>Отчёт по репозиторию</b>", date_range);

        if commits.is_empty() && prs.is_empty() {
            return Self::render_empty(b);
        }

        let pr_stats = PrStats::from(prs);
        let total_additions: i64 = commits.iter().map(|c| c.additions).sum();
        let total_deletions: i64 = commits.iter().map(|c| c.deletions).sum();
        let unique_authors: HashSet<String> = commits
            .iter()
            .filter_map(|c| {
                c.author
                    .as_ref()
                    .and_then(|a| a.login.clone().or_else(|| a.name.clone()))
            })
            .collect();

        b = self.render_summary(
            b,
            commits.len(),
            prs.len(),
            None,
            Some(unique_authors.len()),
            commits.iter().filter_map(|c| c.changed_files).sum(),
        );

        b = self.render_code_stats(b, total_additions, total_deletions, None);

        b = self.render_activity(b, commits);

        b = self.render_pr_block(b, &pr_stats, prs, false);

        b = b.raw(DIVIDER);

        b = self.render_top_contributors(b, commits);

        b = self.render_recent_prs(b, prs);

        b = self.render_recent_commits(b, commits, true);

        b.build()
    }

    fn render_header(
        &self,
        _b: &mut MessageBuilder,
        title: &str,
        date_range: &DateRange,
    ) -> MessageBuilder {
        let period = format!(
            "{} — {}",
            date_range.since.format("%d %b %Y"),
            date_range.until.format("%d %b %Y")
        );
        MessageBuilder::new()
            .raw(&format!("{}\n", title))
            .raw(&format!("📅 <b>Период:</b> {}\n", period))
            .raw(DIVIDER)
    }

    fn render_summary(
        &self,
        b: MessageBuilder,
        commits: usize,
        prs: usize,
        active_days: Option<usize>,
        contributors: Option<usize>,
        files: i64,
    ) -> MessageBuilder {
        let mut b = b.empty_line().raw("📊 <b>Сводка</b>\n");
        if let Some(c) = contributors {
            b = b.raw(&format!("  👥 Контрибьютеров: <b>{}</b>\n", c));
        }
        b = b
            .raw(&format!("  🔨 Коммитов: <b>{}</b>\n", commits))
            .raw(&format!("  📦 Pull Requests: <b>{}</b>\n", prs));
        if let Some(d) = active_days {
            b = b.raw(&format!("  🗓 Активных дней: <b>{}</b>\n", d));
        }
        b.raw(&format!("  📂 Файлов изменено: <b>{}</b>\n", files))
            .empty_line()
    }

    fn render_code_stats(
        &self,
        b: MessageBuilder,
        additions: i64,
        deletions: i64,
        avg_commit_size: Option<i64>,
    ) -> MessageBuilder {
        let net = additions - deletions;
        let sign = if net >= 0 { "+" } else { "" };
        let mut b = b
            .raw("💻 <b>Изменения в коде</b>\n")
            .raw(&format!("  ➕ Добавлено: <b>{}</b>\n", additions))
            .raw(&format!("  ➖ Удалено: <b>{}</b>\n", deletions))
            .raw(&format!("  📈 Баланс: <b>{}{}</b>\n", sign, net));
        if let Some(avg) = avg_commit_size {
            b = b.raw(&format!("  📏 Средний коммит: <b>+{} строк</b>\n", avg));
        }
        b.empty_line()
    }

    fn render_activity(
        &self,
        b: MessageBuilder,
        commits: &[VersionControlDateRangeReportCommit],
    ) -> MessageBuilder {
        let mut day_map: HashMap<String, usize> = HashMap::new();
        for c in commits {
            *day_map
                .entry(c.authored_at.format("%d %b").to_string())
                .or_default() += 1;
        }
        let busiest = day_map
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(d, n)| format!("{} ({} коммитов)", d, n))
            .unwrap_or_else(|| "—".into());

        let biggest = commits.iter().max_by_key(|c| c.additions + c.deletions);

        let mut b = b.raw("🔥 <b>Активность</b>\n").raw(&format!(
            "  🗓 Самый активный день: <b>{}</b>\n",
            MessageBuilder::escape_html(&Self::truncate(&busiest, 40))
        ));

        if let Some(c) = biggest {
            let sha = &c.sha[..7.min(c.sha.len())];
            let msg = Self::truncate(c.message.lines().next().unwrap_or(&c.message), 50);
            b = b.raw(&format!(
                "  💥 Крупнейший коммит: <code>{}</code> <i>{}</i> (+{} / -{})\n",
                sha,
                MessageBuilder::escape_html(&msg),
                c.additions,
                c.deletions
            ));
        }

        b.empty_line()
    }

    fn render_pr_block(
        &self,
        b: MessageBuilder,
        stats: &PrStats,
        prs: &[VersionControlDateRangeReportPullRequest],
        show_merge_time: bool,
    ) -> MessageBuilder {
        if prs.is_empty() {
            return b;
        }

        let biggest = prs.iter().max_by_key(|p| p.additions + p.deletions);

        let mut b = b
            .raw("🔀 <b>Pull Requests</b>\n")
            .raw(&format!(
                "  ✅ Merged: <b>{}</b>  🟢 Open: <b>{}</b>  🔴 Closed: <b>{}</b>\n",
                stats.merged.len(),
                stats.open.len(),
                stats.closed.len()
            ))
            .raw(&format!("  📊 Merge rate: <b>{}</b>\n", stats.merge_rate()));

        if show_merge_time {
            if let Some(t) = stats.avg_merge_time() {
                b = b.raw(&format!("  ⏱ Среднее до merge: <b>{}</b>\n", t));
            }
        } else if let Some(pr) = biggest {
            let title = Self::truncate(&pr.title, 40);
            b = b.raw(&format!(
                "  💥 Крупнейший PR: #{} <i>{}</i> (+{} / -{})\n",
                pr.number,
                MessageBuilder::escape_html(&title),
                pr.additions,
                pr.deletions
            ));
        }

        b.empty_line()
    }

    fn render_recent_commits(
        &self,
        b: MessageBuilder,
        commits: &[VersionControlDateRangeReportCommit],
        show_author: bool,
    ) -> MessageBuilder {
        if commits.is_empty() {
            return b;
        }

        let mut sorted = commits.to_vec();
        sorted.sort_by(|a, b| b.authored_at.cmp(&a.authored_at));
        let hidden = sorted.len().saturating_sub(COMMITS_LIMIT);
        sorted.truncate(COMMITS_LIMIT);

        let mut b = b.raw("🕐 <b>Последние коммиты</b>\n");

        for c in &sorted {
            let sha = &c.sha[..7.min(c.sha.len())];
            let msg = Self::truncate(c.message.lines().next().unwrap_or(&c.message), 45);
            let date = c.authored_at.format("%d %b, %H:%M");

            b = if show_author {
                let author = c
                    .author
                    .as_ref()
                    .and_then(|a| a.login.as_deref().or(a.name.as_deref()))
                    .unwrap_or("unknown");
                b.raw(&format!(
                    "  • <code>{}</code> <i>{}</i>\n    {} — {}\n",
                    sha,
                    MessageBuilder::escape_html(&msg),
                    MessageBuilder::escape_html(author),
                    date
                ))
            } else {
                b.raw(&format!(
                    "  • <code>{}</code> <i>{}</i> — {}\n",
                    sha,
                    MessageBuilder::escape_html(&msg),
                    date
                ))
            };
        }

        if hidden > 0 {
            b = b.raw(&format!("  <i>...и ещё {} коммитов</i>\n", hidden));
        }

        b.empty_line()
    }

    fn render_recent_prs(
        &self,
        b: MessageBuilder,
        prs: &[VersionControlDateRangeReportPullRequest],
    ) -> MessageBuilder {
        if prs.is_empty() {
            return b;
        }

        let mut sorted = prs.to_vec();
        sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let hidden = sorted.len().saturating_sub(PRS_LIMIT);
        sorted.truncate(PRS_LIMIT);

        let mut b = b.raw("📋 <b>Последние Pull Requests</b>\n");

        for pr in &sorted {
            let icon = match (pr.merged_at.is_some(), pr.state.as_str()) {
                (true, _) => "✅",
                (_, "open") => "🟢",
                _ => "🔴",
            };
            let title = Self::truncate(&pr.title, 40);
            let author = pr
                .author
                .as_deref()
                .map(|a| format!(" · @{}", a))
                .unwrap_or_default();

            b = b.raw(&format!(
                "  {} #{} <i>{}</i>{}\n",
                icon,
                pr.number,
                MessageBuilder::escape_html(&title),
                MessageBuilder::escape_html(&author)
            ));
        }

        if hidden > 0 {
            b = b.raw(&format!("  <i>...и ещё {} PR</i>\n", hidden));
        }

        b.empty_line()
    }

    fn render_top_commits(
        &self,
        b: MessageBuilder,
        commits: &[VersionControlDateRangeReportCommit],
    ) -> MessageBuilder {
        if commits.is_empty() {
            return b;
        }

        let mut top = commits.to_vec();
        top.sort_by(|a, b| (b.additions + b.deletions).cmp(&(a.additions + a.deletions)));
        top.truncate(TOP_COMMITS);

        let medals = ["🥇", "🥈", "🥉"];
        let mut b = b.raw("\n🏆 <b>Топ коммитов по объёму изменений</b>\n");

        for (i, c) in top.iter().enumerate() {
            let sha = &c.sha[..7.min(c.sha.len())];
            let msg = Self::truncate(c.message.lines().next().unwrap_or(&c.message), 48);
            b = b.raw(&format!(
                "  {} <code>{}</code> <i>{}</i>\n      ➕{} ➖{}\n",
                medals[i],
                sha,
                MessageBuilder::escape_html(&msg),
                c.additions,
                c.deletions
            ));
        }

        b.empty_line()
    }

    fn render_top_contributors(
        &self,
        b: MessageBuilder,
        commits: &[VersionControlDateRangeReportCommit],
    ) -> MessageBuilder {
        if commits.is_empty() {
            return b;
        }

        let mut map: HashMap<String, ContribStats> = HashMap::new();
        for c in commits {
            let name = c
                .author
                .as_ref()
                .and_then(|a| a.login.clone().or_else(|| a.name.clone()))
                .unwrap_or_else(|| "unknown".into());
            let e = map.entry(name).or_insert(ContribStats {
                commits: 0,
                additions: 0,
                deletions: 0,
            });
            e.commits += 1;
            e.additions += c.additions;
            e.deletions += c.deletions;
        }

        let mut top: Vec<(String, ContribStats)> = map.into_iter().collect();
        top.sort_by(|a, b| b.1.commits.cmp(&a.1.commits));
        top.truncate(TOP_CONTRIBUTORS);

        let max_commits = top.first().map(|(_, s)| s.commits).unwrap_or(1);
        let medals = ["🥇", "🥈", "🥉", "4️⃣", "5️⃣"];

        let mut b = b.raw("\n🏆 <b>Топ контрибьютеров</b>\n");

        for (i, (name, stats)) in top.iter().enumerate() {
            let filled = (stats.commits * 8 / max_commits).max(1);
            let bar = format!("{}{}", "█".repeat(filled), "░".repeat(8 - filled));
            b = b.raw(&format!(
                "  {} <b>{}</b>\n     <code>{}</code> {} коммитов | ➕{} ➖{}\n",
                medals[i],
                MessageBuilder::escape_html(name),
                bar,
                stats.commits,
                stats.additions,
                stats.deletions
            ));
        }

        b.empty_line()
    }

    fn render_empty(b: MessageBuilder) -> String {
        b.raw("\n😴 <i>За этот период активности не обнаружено.</i>\n")
            .build()
    }

    fn truncate(s: &str, max_chars: usize) -> String {
        let mut chars = s.chars();
        let out: String = chars.by_ref().take(max_chars).collect();
        if chars.next().is_some() {
            format!("{}…", out)
        } else {
            out
        }
    }
}

impl CommandExecutor for BuildVersionControlDateRangeReportExecutor {
    type Command = BuildVersionControlDateRangeReportExecutorCommand;
    type Response = BuildVersionControlDateRangeReportExecutorResponse;
    type Error = BuildVersionControlDateRangeReportExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let social_user = self
            .user_socials_repo
            .find_by_social_user_id(&cmd.social_user_id)
            .await?;

        let version_control_user = self
            .user_version_control_service_repo
            .find_by_user_id(&social_user.user_id)
            .await?;

        let decrypted_token = self
            .reversible_cipher
            .decrypt(version_control_user.access_token.value())?;

        let author = match cmd.for_who {
            BuildVersionControlDateRangeReportExecutorCommandForWho::Me => {
                Some(version_control_user.version_control_login)
            }
            BuildVersionControlDateRangeReportExecutorCommandForWho::Repository => None,
        };

        let report = self
            .version_control_client
            .get_details_by_range(&decrypted_token, &cmd.date_range, author.as_deref())
            .await?;

        tracing::debug!("Version control report built: {:?}", report);

        Ok(BuildVersionControlDateRangeReportExecutorResponse {
            text: self.render(&report, &author, &cmd.date_range),
        })
    }
}
