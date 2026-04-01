use crate::application::user::commands::assign_user_role::command::AssignUserRoleCommand;
use crate::application::user::commands::remove_user_role::command::RemoveUserRoleCommand;
use crate::application::user::commands::toggle_user_active::command::ToggleUserActiveCommand;
use crate::application::user::queries::get_all_users::query::GetAllUsersQuery;
use crate::application::user::queries::get_all_users::response::UserListItem;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::admin::TelegramBotDialogueAdminState;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::actions::admin_user_menu::TelegramBotAdminUserMenuAction;
use crate::delivery::bot::telegram::keyboards::actions::admin_users::{
    TelegramBotAdminUsersListAction, USER_SELECT_PREFIX, user_select_callback,
};
use crate::delivery::bot::telegram::keyboards::actions::choose_role::TelegramBotChooseRoleAction;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
use crate::domain::role::value_objects::role_name::RoleName;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::user_id::UserId;
use crate::utils::builder::message::MessageBuilder;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::payloads::{EditMessageTextSetters, SendMessageSetters};
use teloxide::prelude::*;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, MessageId, ParseMode};
use teloxide::{Bot, dptree};

pub struct TelegramBotDialogueAdminUsersDispatcher;

impl TelegramBotDialogueAdminUsersDispatcher {
    pub fn query_branches(
    ) -> Handler<'static, Result<(), Box<dyn std::error::Error + Send + Sync>>, DpHandlerDescription>
    {
        dptree::entry()
            .branch(
                case![TelegramBotDialogueAdminState::UserList].endpoint(handle_list_action),
            )
            .branch(
                case![TelegramBotDialogueAdminState::UserMenu { user_id }]
                    .endpoint(handle_user_menu),
            )
            .branch(
                case![TelegramBotDialogueAdminState::UserAssignRole { user_id }]
                    .endpoint(handle_assign_role),
            )
            .branch(
                case![TelegramBotDialogueAdminState::UserRemoveRole { user_id }]
                    .endpoint(handle_remove_role),
            )
    }

    pub async fn show_list(
        bot: &Bot,
        chat_id: ChatId,
        message_id: MessageId,
        executors: &ApplicationBoostrapExecutors,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let result = executors
            .queries
            .get_all_users
            .execute(&GetAllUsersQuery)
            .await;

        let users = match result {
            Ok(r) => r.users,
            Err(e) => {
                tracing::error!(error = %e, "Failed to get users");

                bot.edit_message_text(
                    chat_id,
                    message_id,
                    t!("telegram_bot.dialogues.admin.users.load_error").to_string(),
                )
                .await?;

                return Ok(());
            }
        };

        let mut builder = MessageBuilder::new()
            .bold(&t!("telegram_bot.dialogues.admin.users.title").to_string())
            .empty_line();

        if users.is_empty() {
            builder = builder.line(
                &t!("telegram_bot.dialogues.admin.users.empty").to_string(),
            );
        } else {
            for user in &users {
                let active_icon = if user.is_active { "✅" } else { "⏸" };

                let login = user
                    .social_login
                    .as_deref()
                    .unwrap_or("—");

                let roles_str = if user.roles.is_empty() {
                    "—".to_string()
                } else {
                    user.roles
                        .iter()
                        .map(|r| role_display_name(r))
                        .collect::<Vec<_>>()
                        .join(", ")
                };

                let line = format!(
                    "{} <b>{}</b> (ID:{}) — {}",
                    active_icon, login, user.user_id.0, roles_str,
                );

                builder = builder.raw(&line).raw("\n");
            }
        }

        let text = builder.build();

        let mut keyboard = KeyboardBuilder::new();

        if !users.is_empty() {
            keyboard = keyboard.row::<TelegramBotAdminUsersListAction>(vec![
                TelegramBotAdminUsersListAction::Select,
            ]);
        }

        keyboard = keyboard.row::<TelegramBotAdminUsersListAction>(vec![
            TelegramBotAdminUsersListAction::Cancel,
        ]);

        bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard.build())
            .await?;

        Ok(())
    }
}

fn role_display_name(role: &RoleName) -> &'static str {
    match role {
        RoleName::Admin => "Админ",
        RoleName::Developer => "Разработчик",
        RoleName::QualityAssurance => "QA",
    }
}

async fn handle_list_action(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let data = query.data.as_deref().unwrap_or("");

    let action = match TelegramBotAdminUsersListAction::from_callback_data(data) {
        Ok(a) => a,
        Err(_) => {
            let msg = match query.message {
                Some(m) => m,
                None => return Ok(()),
            };

            bot.send_message(
                msg.chat().id,
                t!("telegram_bot.common.cancelled").to_string(),
            )
            .await?;

            dialogue.exit().await.ok();
            return Ok(());
        }
    };

    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };

    match action {
        TelegramBotAdminUsersListAction::Select => {
            let users = executors
                .queries
                .get_all_users
                .execute(&GetAllUsersQuery)
                .await
                .map(|r| r.users)
                .unwrap_or_default();

            if users.is_empty() {
                bot.send_message(
                    msg.chat().id,
                    t!("telegram_bot.dialogues.admin.users.empty").to_string(),
                )
                .await?;

                dialogue.exit().await.ok();
                return Ok(());
            }

            let rows: Vec<Vec<InlineKeyboardButton>> = users
                .iter()
                .map(|u| {
                    let active_icon = if u.is_active { "✅" } else { "⏸" };
                    let login = u.social_login.as_deref().unwrap_or("—");
                    let label = format!("{} {} (ID:{})", active_icon, login, u.user_id.0);

                    vec![InlineKeyboardButton::callback(
                        label,
                        user_select_callback(u.user_id.0),
                    )]
                })
                .collect();

            bot.send_message(
                msg.chat().id,
                t!("telegram_bot.dialogues.admin.users.select_user").to_string(),
            )
            .reply_markup(InlineKeyboardMarkup::new(rows))
            .await?;

            dialogue
                .update(TelegramBotDialogueState::Admin(
                    TelegramBotDialogueAdminState::UserMenu { user_id: 0 },
                ))
                .await?;
        }

        TelegramBotAdminUsersListAction::Cancel => {
            bot.send_message(
                msg.chat().id,
                t!("telegram_bot.common.cancelled").to_string(),
            )
            .await?;

            dialogue.exit().await.ok();
        }
    }

    Ok(())
}

async fn handle_user_menu(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
    user_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let data = query.data.as_deref().unwrap_or("");

    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };

    // If user_id == 0, we're in selection mode — parse from callback data
    if data.starts_with(USER_SELECT_PREFIX) {
        let parsed_id: i32 = match data.strip_prefix(USER_SELECT_PREFIX).and_then(|s| s.parse().ok()) {
            Some(v) => v,
            None => {
                dialogue.exit().await.ok();
                return Ok(());
            }
        };

        return show_user_menu(&bot, &dialogue, &executors, msg.chat().id, parsed_id).await;
    }

    let action = match TelegramBotAdminUserMenuAction::from_callback_data(data) {
        Ok(a) => a,
        Err(e) => {
            tracing::error!(error = %e, "Unknown user menu action");
            dialogue.exit().await.ok();
            return Ok(());
        }
    };

    match action {
        TelegramBotAdminUserMenuAction::Toggle => {
            let user = find_user_item(&executors, user_id).await;

            if let Some(user) = user {
                let new_active = !user.is_active;

                let cmd = ToggleUserActiveCommand {
                    user_id: UserId(user_id),
                    is_active: new_active,
                };

                match executors.commands.toggle_user_active.execute(&cmd).await {
                    Ok(_) => {
                        let reply = if new_active {
                            t!("telegram_bot.dialogues.admin.users.activated")
                        } else {
                            t!("telegram_bot.dialogues.admin.users.deactivated")
                        };

                        bot.send_message(msg.chat().id, reply.to_string()).await?;
                    }

                    Err(e) => {
                        tracing::error!(error = %e, "Failed to toggle user active");

                        bot.send_message(
                            msg.chat().id,
                            t!("telegram_bot.dialogues.admin.users.error").to_string(),
                        )
                        .await?;
                    }
                }
            }

            dialogue.exit().await.ok();
        }

        TelegramBotAdminUserMenuAction::AssignRole => {
            let user = find_user_item(&executors, user_id).await;

            let all_roles = vec![
                TelegramBotChooseRoleAction::QualityAssurance,
                TelegramBotChooseRoleAction::Developer,
            ];

            let user_roles = user.map(|u| u.roles).unwrap_or_default();

            let available: Vec<TelegramBotChooseRoleAction> = all_roles
                .into_iter()
                .filter(|action| {
                    let role: RoleName = action.clone().into();
                    !user_roles.contains(&role)
                })
                .collect();

            if available.is_empty() {
                bot.send_message(
                    msg.chat().id,
                    t!("telegram_bot.dialogues.admin.users.all_roles_assigned").to_string(),
                )
                .await?;

                dialogue.exit().await.ok();
                return Ok(());
            }

            let keyboard = KeyboardBuilder::new()
                .row::<TelegramBotChooseRoleAction>(available)
                .build();

            dialogue
                .update(TelegramBotDialogueState::Admin(
                    TelegramBotDialogueAdminState::UserAssignRole { user_id },
                ))
                .await?;

            bot.send_message(
                msg.chat().id,
                t!("telegram_bot.dialogues.admin.users.select_role_assign").to_string(),
            )
            .reply_markup(keyboard)
            .await?;
        }

        TelegramBotAdminUserMenuAction::RemoveRole => {
            let user = find_user_item(&executors, user_id).await;

            let current_roles = user.map(|u| u.roles).unwrap_or_default();

            if current_roles.is_empty() {
                bot.send_message(
                    msg.chat().id,
                    t!("telegram_bot.dialogues.admin.users.no_roles").to_string(),
                )
                .await?;

                dialogue.exit().await.ok();
                return Ok(());
            }

            let role_actions: Vec<TelegramBotChooseRoleAction> = current_roles
                .iter()
                .filter_map(|r| TelegramBotChooseRoleAction::try_from_role(r))
                .collect();

            let keyboard = KeyboardBuilder::new()
                .row::<TelegramBotChooseRoleAction>(role_actions)
                .build();

            dialogue
                .update(TelegramBotDialogueState::Admin(
                    TelegramBotDialogueAdminState::UserRemoveRole { user_id },
                ))
                .await?;

            bot.send_message(
                msg.chat().id,
                t!("telegram_bot.dialogues.admin.users.select_role_remove").to_string(),
            )
            .reply_markup(keyboard)
            .await?;
        }
    }

    Ok(())
}

async fn handle_assign_role(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
    user_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let data = query.data.as_deref().unwrap_or("");

    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };

    let action = match TelegramBotChooseRoleAction::from_callback_data(data) {
        Ok(a) => a,
        Err(_) => {
            dialogue.exit().await.ok();
            return Ok(());
        }
    };

    let role_name: RoleName = action.into();

    let cmd = AssignUserRoleCommand {
        user_id: UserId(user_id),
        role_name,
    };

    let reply = match executors.commands.assign_user_role.execute(&cmd).await {
        Ok(_) => t!("telegram_bot.dialogues.admin.users.role_assigned").to_string(),

        Err(e) => {
            tracing::error!(error = %e, "Failed to assign role");
            t!("telegram_bot.dialogues.admin.users.error").to_string()
        }
    };

    bot.send_message(msg.chat().id, reply).await?;

    dialogue.exit().await.ok();

    Ok(())
}

async fn handle_remove_role(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
    user_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let data = query.data.as_deref().unwrap_or("");

    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };

    let action = match TelegramBotChooseRoleAction::from_callback_data(data) {
        Ok(a) => a,
        Err(_) => {
            dialogue.exit().await.ok();
            return Ok(());
        }
    };

    let role_name: RoleName = action.into();

    let cmd = RemoveUserRoleCommand {
        user_id: UserId(user_id),
        role_name,
    };

    let reply = match executors.commands.remove_user_role.execute(&cmd).await {
        Ok(_) => t!("telegram_bot.dialogues.admin.users.role_removed").to_string(),

        Err(e) => {
            tracing::error!(error = %e, "Failed to remove role");
            t!("telegram_bot.dialogues.admin.users.error").to_string()
        }
    };

    bot.send_message(msg.chat().id, reply).await?;

    dialogue.exit().await.ok();

    Ok(())
}

async fn show_user_menu(
    bot: &Bot,
    dialogue: &TelegramBotDialogueType,
    executors: &ApplicationBoostrapExecutors,
    chat_id: ChatId,
    user_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user = find_user_item(executors, user_id).await;

    let user = match user {
        Some(u) => u,
        None => {
            bot.send_message(
                chat_id,
                t!("telegram_bot.dialogues.admin.users.not_found").to_string(),
            )
            .await?;

            dialogue.exit().await.ok();
            return Ok(());
        }
    };

    let active_status = if user.is_active {
        t!("telegram_bot.dialogues.admin.users.status_active").to_string()
    } else {
        t!("telegram_bot.dialogues.admin.users.status_inactive").to_string()
    };

    let login = user.social_login.as_deref().unwrap_or("—");

    let roles_str = if user.roles.is_empty() {
        "—".to_string()
    } else {
        user.roles
            .iter()
            .map(|r| role_display_name(r))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let text = MessageBuilder::new()
        .bold(&format!("👤 {} (ID:{})", login, user.user_id.0))
        .empty_line()
        .section(
            &t!("telegram_bot.dialogues.admin.users.field_status").to_string(),
            &active_status,
        )
        .section(
            &t!("telegram_bot.dialogues.admin.users.field_roles").to_string(),
            &roles_str,
        )
        .section(
            &t!("telegram_bot.dialogues.admin.users.field_created").to_string(),
            &user.created_at.format("%d.%m.%Y %H:%M").to_string(),
        )
        .build();

    let keyboard = KeyboardBuilder::new()
        .row::<TelegramBotAdminUserMenuAction>(vec![
            TelegramBotAdminUserMenuAction::Toggle,
        ])
        .row::<TelegramBotAdminUserMenuAction>(vec![
            TelegramBotAdminUserMenuAction::AssignRole,
        ])
        .row::<TelegramBotAdminUserMenuAction>(vec![
            TelegramBotAdminUserMenuAction::RemoveRole,
        ])
        .build();

    dialogue
        .update(TelegramBotDialogueState::Admin(
            TelegramBotDialogueAdminState::UserMenu { user_id },
        ))
        .await?;

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

async fn find_user_item(
    executors: &ApplicationBoostrapExecutors,
    user_id: i32,
) -> Option<UserListItem> {
    executors
        .queries
        .get_all_users
        .execute(&GetAllUsersQuery)
        .await
        .ok()
        .and_then(|r| r.users.into_iter().find(|u| u.user_id.0 == user_id))
}
