use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::config::application::ApplicationConfig;
use crate::delivery::bot::telegram::commands::admin::TelegramBotAdminCommandHandler;
use crate::delivery::bot::telegram::commands::bind_repository::TelegramBotBindRepositoryCommandHandler;
use crate::delivery::bot::telegram::commands::digest::TelegramBotDigestCommandHandler;
use crate::delivery::bot::telegram::commands::my_prs::TelegramBotMyPrsCommandHandler;
use crate::delivery::bot::telegram::commands::notifications::TelegramBotNotificationsCommandHandler;
use crate::delivery::bot::telegram::commands::pending_reviews::TelegramBotPendingReviewsCommandHandler;
use crate::delivery::bot::telegram::commands::register::TelegramBotRegisterCommandHandler;
use crate::delivery::bot::telegram::commands::release_plan::TelegramBotReleasePlanCommandHandler;
use crate::delivery::bot::telegram::commands::releases::TelegramBotReleasesCommandHandler;
use crate::delivery::bot::telegram::commands::whoami::TelegramBotWhoamiCommandHandler;
use crate::delivery::bot::telegram::commands::report::TelegramBotVersionControlReportCommandHandler;
use crate::delivery::bot::telegram::commands::setup::TelegramBotSetupCommandHandler;
use crate::delivery::bot::telegram::commands::setup_notifications::TelegramBotSetupNotificationsCommandHandler;
use crate::delivery::bot::telegram::commands::setup_webhook::TelegramBotSetupWebhookCommandHandler;
use crate::delivery::bot::telegram::commands::start::TelegramBotStartCommandHandler;
use crate::delivery::bot::telegram::commands::status::TelegramBotStatusCommandHandler;
use crate::delivery::bot::telegram::commands::task::TelegramBotTaskCommandHandler;
use crate::delivery::bot::telegram::commands::unregister::TelegramBotUnregisterCommandHandler;
use crate::delivery::bot::telegram::commands::vacation::TelegramBotVacationCommandHandler;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::delivery::bot::telegram::dialogues::TelegramBotDialogueType;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::macros::BotCommands;
use teloxide::prelude::Requester;
use teloxide::types::{ChatKind, Message, User};

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "Доступные команды:")]
pub enum TelegramBotCommand {
    #[command(description = "Запустить бота")]
    Start,
    #[command(description = "Создать пользователя")]
    Register,
    #[command(description = "Получить отчет")]
    Report,
    #[command(
        rename = "bind_repository",
        description = "Привязать/отвязать репозиторий"
    )]
    BindRepository,
    #[command(description = "Панель администратора")]
    Admin,
    #[command(description = "Получить карточку по ID: /task 12345")]
    Task(String),
    #[command(
        rename = "setup_webhook",
        description = "Привязать чат к webhook-логам репозитория (push/release/CI/PR)"
    )]
    SetupWebhook,

    #[command(
        rename = "setup_notifications",
        description = "Привязать чат к командным уведомлениям (теги ревью, релизы, stale)"
    )]
    SetupNotifications,

    #[command(description = "Деактивировать аккаунт")]
    Unregister,

    #[command(description = "Настройка дайджест-уведомлений")]
    Digest,

    #[command(description = "Настройка уведомлений (DND, snooze, vacation)")]
    Notifications,

    #[command(description = "Уйти в отпуск: /vacation 5d или /vacation off")]
    Vacation(String),

    #[command(description = "Завершить настройку: репо + тихие часы")]
    Setup,

    #[command(rename = "release_plan", description = "Создать план релиза")]
    ReleasePlan,

    #[command(description = "Список запланированных релизов")]
    Releases,

    #[command(description = "Мой профиль и настройки")]
    Whoami,

    #[command(description = "Статус сервисов и health-pings (Admin)")]
    Status,

    #[command(rename = "my_prs", description = "Мои открытые PR")]
    MyPrs,

    #[command(rename = "pending_reviews", description = "PR, ожидающие моего ревью")]
    PendingReviews,
}

pub async fn handle(
    bot: Bot,
    user: User,
    msg: Message,
    cmd: TelegramBotCommand,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    config: Arc<ApplicationConfig>,
    shared_dependency: Arc<crate::bootstrap::shared_dependency::ApplicationSharedDependency>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if matches!(
        &cmd,
        TelegramBotCommand::SetupWebhook | TelegramBotCommand::SetupNotifications
    ) {
        if !matches!(msg.chat.kind, ChatKind::Public(_)) {
            bot.send_message(
                msg.chat.id,
                t!("telegram_bot.commands.group_only").to_string(),
            )
            .await?;
            return Ok(());
        }

        let is_notifications = matches!(cmd, TelegramBotCommand::SetupNotifications);
        let context = TelegramBotCommandContext {
            bot,
            user,
            msg,
            cmd,
            config,
        };
        if is_notifications {
            return TelegramBotSetupNotificationsCommandHandler::new(
                context,
                executors.clone(),
                Arc::new(dialogue),
            )
            .execute()
            .await;
        }
        return TelegramBotSetupWebhookCommandHandler::new(
            context,
            executors.clone(),
            Arc::new(dialogue),
        )
        .execute()
        .await;
    }

    // All other commands are private-chat only
    if !matches!(msg.chat.kind, ChatKind::Private(_)) {
        bot.send_message(
            msg.chat.id,
            t!("telegram_bot.commands.private_only").to_string(),
        )
        .await?;
        return Ok(());
    }

    let context = TelegramBotCommandContext {
        bot,
        user,
        msg,
        cmd,
        config,
    };

    match context.cmd {
        TelegramBotCommand::Start => {
            TelegramBotStartCommandHandler::new(context)
                .execute()
                .await?;
        }
        TelegramBotCommand::Register => {
            TelegramBotRegisterCommandHandler::new(
                context,
                executors.commands.create_oauth_link.clone(),
                Arc::new(dialogue),
            )
            .execute()
            .await?;
        }
        TelegramBotCommand::Report => {
            TelegramBotVersionControlReportCommandHandler::new(context, Arc::new(dialogue))
                .execute()
                .await?;
        }
        TelegramBotCommand::BindRepository => {
            TelegramBotBindRepositoryCommandHandler::new(
                context,
                executors.clone(),
                Arc::new(dialogue),
            )
            .execute()
            .await?;
        }
        TelegramBotCommand::Admin => {
            TelegramBotAdminCommandHandler::new(
                context,
                executors.queries.get_user_roles_by_telegram_id.clone(),
                Arc::new(dialogue),
            )
            .execute()
            .await?;
        }
        TelegramBotCommand::Task(raw_id) => {
            TelegramBotTaskCommandHandler::new(
                context.bot,
                context.msg,
                executors.queries.get_task_card.clone(),
                raw_id,
            )
            .execute()
            .await?;
        }

        TelegramBotCommand::Unregister => {
            TelegramBotUnregisterCommandHandler::new(
                context,
                executors.commands.deactivate_user.clone(),
            )
            .execute()
            .await?;
        }

        TelegramBotCommand::Digest => {
            TelegramBotDigestCommandHandler::new(context, executors.clone(), Arc::new(dialogue))
                .execute()
                .await?;
        }

        TelegramBotCommand::Notifications => {
            TelegramBotNotificationsCommandHandler::new(
                context,
                executors.clone(),
                shared_dependency.clone(),
                Arc::new(dialogue),
            )
            .execute()
            .await?;
        }

        TelegramBotCommand::Vacation(raw_arg) => {
            let social_user_id = crate::domain::user::value_objects::social_user_id::SocialUserId(
                context.user.id.0 as i32,
            );
            TelegramBotVacationCommandHandler::new(
                context.bot,
                context.msg,
                executors.clone(),
                raw_arg,
                social_user_id,
            )
            .execute()
            .await?;
        }

        TelegramBotCommand::Setup => {
            let social_user_id = crate::domain::user::value_objects::social_user_id::SocialUserId(
                context.user.id.0 as i32,
            );
            TelegramBotSetupCommandHandler::new(
                context.bot,
                context.msg,
                executors.clone(),
                Arc::new(dialogue),
                social_user_id,
            )
            .execute()
            .await?;
        }

        TelegramBotCommand::ReleasePlan => {
            TelegramBotReleasePlanCommandHandler::new(context.bot, context.msg, Arc::new(dialogue))
                .execute()
                .await?;
        }

        TelegramBotCommand::Releases => {
            TelegramBotReleasesCommandHandler::new(
                context,
                executors.clone(),
                Arc::new(dialogue),
            )
            .execute()
            .await?;
        }

        TelegramBotCommand::Whoami => {
            TelegramBotWhoamiCommandHandler::new(context, executors.clone())
                .execute()
                .await?;
        }

        TelegramBotCommand::Status => {
            TelegramBotStatusCommandHandler::new(context, executors.clone())
                .execute()
                .await?;
        }

        TelegramBotCommand::MyPrs => {
            TelegramBotMyPrsCommandHandler::new(context, executors.clone())
                .execute()
                .await?;
        }

        TelegramBotCommand::PendingReviews => {
            TelegramBotPendingReviewsCommandHandler::new(context, executors.clone())
                .execute()
                .await?;
        }

        // Handled above before private-chat guard
        TelegramBotCommand::SetupWebhook | TelegramBotCommand::SetupNotifications => {}
    }

    Ok(())
}
