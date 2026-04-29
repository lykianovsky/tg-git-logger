# GitEye Bot — Roadmap

## Этап 1: DND + Code Review Loop

Базовый функционал: тихий режим по умолчанию + расширение цикла код-ревью.

**Дефолт DND:** 20:00–10:00 МСК для всех юзеров (настраивается в `.env`).

### Веха 1 — Фундамент DND и дефолтные тихие часы

**Цель:** глобально работают дефолтные тихие часы. Уведомления в DND дропаются (буфер появится в Вехе 2).

1. Конфиг (`.env` + `src/config/application/mod.rs`):
   ```
   NOTIFICATIONS_DEFAULT_DND_START=20:00
   NOTIFICATIONS_DEFAULT_DND_END=10:00
   NOTIFICATIONS_DEFAULT_TIMEZONE=Europe/Moscow
   ```
   Новая sub-config struct `ApplicationNotificationsConfig`.

2. Миграция `create_user_preferences`:
   ```
   user_preferences
     ├── id (PK)
     ├── user_id (FK → users, unique)
     ├── timezone           (nullable, default null = из конфига)
     ├── dnd_start          (TIME, nullable)
     ├── dnd_end            (TIME, nullable)
     ├── vacation_until     (DATETIME, nullable)
     ├── snooze_until       (DATETIME, nullable)
     ├── enabled_events     (JSON: ["pr","review","comment","ci","release"])
     ├── priority_only      (BOOL)
     └── created_at, updated_at
   ```
   Запуск `make generate-database-entity`.

3. Domain (`src/domain/user_preferences/`):
   - Entity `UserPreferences`
   - Port `UserPreferencesRepository` (find_by_user_id, upsert)
   - Service `QuietHoursResolver` — чистая логика «сейчас тихий час для юзера X?»

4. Infrastructure (`src/infrastructure/database/mysql/user_preferences/`):
   - Реализация `UserPreferencesRepository`

5. Application — middleware:
   - В `SendSocialNotifyExecutor` перед отправкой: `if QuietHoursResolver.is_quiet(user_id, now) { return Ok(skipped) }`
   - Логировать `tracing::debug!` со причиной скипа.

6. Bootstrap — зарегистрировать новые зависимости в `ApplicationSharedDependency`.

**Выход вехи:** ничего не приходит ночью никому.

### Веха 2 — Буфер отложенных уведомлений + утренний дайджест

**Цель:** ночные события не теряются, утром в 10:00 приходит одним сообщением.

1. Миграция `pending_notifications`:
   ```
   pending_notifications
     ├── id, user_id (FK)
     ├── payload (JSON — готовое сообщение + метаданные события)
     ├── event_type (для группировки)
     ├── created_at
     └── deliver_after (DATETIME — когда отдать)
   ```

2. Domain — `PendingNotification` entity + repository port.

3. Application:
   - `BufferNotificationCommand` — кладёт в pending вместо drop
   - `FlushPendingNotificationsCommand` — забирает все где `deliver_after <= now`, склеивает по юзеру в один MessageBuilder, отправляет, удаляет из БД

4. Middleware Вехи 1 поправить: вместо drop — вызываем `BufferNotificationCommand` с `deliver_after = next_active_window(user)`.

5. Cron (`src/delivery/scheduler/`) — каждую минуту вызывать `FlushPendingNotificationsCommand`.

**Выход вехи:** ночные события приходят утром одним сообщением «За ночь произошло: …».

### Веха 3 — Команда `/notifications`

**Цель:** юзер сам управляет своим DND через бот.

1. Dialogue (`src/delivery/bot/telegram/dialogues/notifications/`) — state machine:
   ```
   /notifications →
     ├── 🕘 Рабочие часы       → ввод HH:MM–HH:MM
     ├── 🌐 Часовой пояс        → выбор из списка (или ввод)
     ├── 🏖 Vacation            → 1d / 3d / 7d / custom
     ├── 😴 Snooze              → 2ч / 4ч / до утра
     ├── 🔕 Фильтры событий     → toggle inline-кнопками
     ├── 🚨 Priority-only       → on/off
     └── 🔄 Сбросить к дефолту
   ```

2. Application commands:
   - `UpdateUserPreferencesCommand` (с разными вариантами patch)
   - `GetUserPreferencesQuery`

3. Команда показывает текущие значения с пометкой «(default)» если не переопределено.

**Выход вехи:** каждый разраб сам себе настраивает время.

### Веха 4 — `review_requested` → ЛС ревьюеру

**Цель:** когда тебя назначают на ревью — сразу узнаёшь.

1. Webhook parser — расширить обработку `pull_request` action — добавить кейс `review_requested`. Извлечь `requested_reviewer.login`.

2. Domain event `WebhookReviewRequestedEvent`.

3. Listener (`src/delivery/events/`) — `OnReviewRequested`:
   - find user by github_login
   - если найден → publish `SendSocialNotifyJob` с шаблоном
   - проверить `vacation_until` — если в отпуске, ничего не делаем (Веха 7 расширит)

4. MessageBuilder шаблон:
   ```
   👀 Вы назначены на ревью
   PR: #123 — Refactor auth middleware
   Автор: @ivan
   [Открыть PR]
   ```

5. i18n ключи в `locales/ru.json`.

**Выход вехи:** ревьюеры перестают пропускать назначения.

### Веха 5 — `@-tag` привязанных юзеров при открытии PR

**Цель:** упомянули в PR description — узнал в TG.

1. Parser util (`src/utils/`) — `extract_github_mentions(text) -> Vec<String>` (regex `@[\w-]+`).

2. Listener `OnPullRequestOpened` — парсим `pr.body + pr.title`, делаем `find_users_by_github_logins`, для каждого найденного — `SendSocialNotifyJob`.

3. В групповой чат (если репо привязан к чату) — добавить пометку с `@tg_username`-ами для тех, кто привязан.

**Выход вехи:** меньше «эй, посмотри пожалуйста, я тебя в PR упомянул» в личке.

### Веха 6 — Re-review нудж

**Цель:** автор пушит коммиты после ревью — старый ревьюер видит апдейт.

1. Миграция `pr_reviews`:
   ```
   pr_reviews (id, repository_id, pr_number, reviewer_login, last_reviewed_at, last_review_state)
   ```
   Заполняется на каждое `pull_request_review submitted`.

2. Listener `OnPullRequestSynchronize` (action=`synchronize`):
   - select reviewers из `pr_reviews` для этого PR
   - для каждого, кто привязан, отправить «🔄 Автор обновил PR #X — посмотрите ещё раз»
   - dedup через `notification_log` — не слать чаще раза в N часов

3. Миграция `notification_log`:
   ```
   notification_log (user_id, kind, key, sent_at)
   ```

**Выход вехи:** ревью-цикл замыкается без ручных пинаний.

### Веха 7 — Vacation mode + автокомментарий в PR

**Цель:** человек в отпуске не получает спам и команда об этом знает.

1. `/vacation 5d` или `/vacation until 10.05` — простая команда, ставит `vacation_until` в `user_preferences`.

2. В Вехе 4 listener `OnReviewRequested` — если ревьюер в отпуске:
   - не слать ему ЛС
   - через GitHub REST API оставить комментарий ботом в PR: `❄️ @ivan в отпуске до 04.05.2026 — переназначьте ревью`
   - уведомить лида (роль Admin)

3. Расширить GitHub REST adapter — `post_pr_comment(repo, pr, body)` (новый метод).

**Выход вехи:** отпуска не ломают ревью-флоу.

### Веха 8 — Полный onboarding через `/register`

**Цель:** после `/register` юзер уже полностью настроен — не нужно бегать по командам `/bind_repository`, `/notifications` и т.д.

**Расширение текущего `/register` flow:**

После успешного OAuth (текущая логика остаётся), бот ведёт по чек-листу одним непрерывным диалогом:

1. ✅ **GitHub привязан** (текущий шаг)
2. **Выбор репозиториев** — UI как в `/bind_repository`: чекбоксы + опция «привязать все доступные»
3. **Часовой пояс** — список популярных или ручной ввод. Дефолт из `NOTIFICATIONS_DEFAULT_TIMEZONE`
4. **Рабочие часы** — предложить дефолт `10:00–20:00 МСК` (из конфига) или ввести свои; реюзает логику Вехи 1
5. **Типы уведомлений** — inline-toggles: `[✓] PR  [✓] Reviews  [✓] Comments  [ ] CI  [ ] Releases`
6. **Финал** — `🎉 Готово! Вы привязали N репозиториев и настроены. Команды: /my_prs, /notifications, /vacation`

Каждый шаг можно скипнуть кнопкой `Пропустить, оставить дефолты` — для быстрых юзеров.

**Off-boarding (расширение `/unregister`):**
- Освободить все pending review-назначения юзера в open PR через GitHub API + автокомментарий «X деактивирован, переназначьте»
- Уведомить авторов тех PR в TG
- Архивировать `user_preferences` (не удалять — на случай повторной регистрации)

**Сложность:** M (новый составной диалог, переиспользует Вехи 1, 3 и логику `bind_repository`).
**Зависит от:** Веха 1, 2, 3.

---

## Этап 2 — Автоматизация ревью в групповом чате

**Сценарий из практики:** сейчас разрабы вручную кидают в чат «PR на ревью», тегают людей, через день ретегают если ревью нет. Бот делает это сам.

### Веха 9 — Авто-тегание ревьюеров при открытии PR

**Триггер:** webhook `pull_request` action=`opened` или `ready_for_review`.

**Действие:** в групповой чат репозитория (привязанный через `/setup_webhook`) бот постит:
```
🆕 PR открыт — ждёт ревью
#123 — Refactor auth middleware
Автор: @ivan
Ревьюеры: @petr_tg, @maria_tg
[Открыть PR]
```

Логика тегания:
- Если у ревьюера есть `social_user_login` в `user_social_accounts` → тег `@telegram_username` (реальное упоминание с пушем)
- Если не привязан → просто GitHub login текстом (без тега)
- Если PR открыт без ревьюеров → пометка `⚠️ ревьюеры не указаны`

Источник данных: поле `pull_request.requested_reviewers[]` уже приходит в webhook payload.

**Сложность:** S.

### Веха 10 — Re-tag stale PR в чате

**Триггер:** cron каждый час.

**Логика:**
1. GraphQL `GithubPullRequests` забирает все open PR в привязанных репо
2. Для каждого: смотрим `last_review_activity_at` (последний review/review-comment); если нет — `created_at`
3. Если активности нет >`REVIEW_STALE_THRESHOLD_HOURS` (дефолт 24ч) — в групповой чат:
   ```
   ⏰ Напоминаю про ревью
   #123 — Refactor auth middleware
   Висит без ревью: 26 часов
   Ревьюеры: @petr_tg, @maria_tg
   [Открыть PR]
   ```
4. Если >`REVIEW_STALE_ESCALATION_HOURS` (дефолт 48ч) — дополнительно тегнуть юзеров с ролью Lead/Admin
5. Дедуп через `notification_log`: re-tag не чаще раза в `REVIEW_STALE_RETAG_INTERVAL_HOURS` (дефолт 12ч) на один PR

**Конфиг (`.env`):**
```
REVIEW_STALE_THRESHOLD_HOURS=24
REVIEW_STALE_ESCALATION_HOURS=48
REVIEW_STALE_RETAG_INTERVAL_HOURS=12
```

**Сложность:** M (новый cron + GraphQL запрос + использует `notification_log` из Вехи 6).

### Веха 11 — Уведомление в чат о результате ревью

**Триггер:** webhook `pull_request_review` action=`submitted`.

**Действие в групповом чате:**
- `approved` → `✅ #123 одобрен @petr_tg`
- `changes_requested` → `🔁 #123 — запрошены правки от @petr_tg`
- `commented` → НЕ постим в чат (шумно), только в ЛС автору (см. Веху 4)

**Сложность:** S — расширяет существующий handler `WebhookPullRequestReviewEvent`.

### Веха 12 — Conflict detected → ЛС автору

**Триггер:** cron каждые N минут проверяет open PR на `mergeable_state == 'dirty'`.

**Действие:** ЛС автору `⚠️ PR #123 конфликтует с base. Подтяните main и разрешите конфликты.`

**Дедуп:** `notification_log` — не чаще раза на каждое появление конфликта (после resolve и нового конфликта — снова).

**Сложность:** S–M.

---

## Этап 3 — Утилиты для разрабов и лида

Точечные команды и алерты на основе существующих данных. Все вехи независимы, можно мерджить по одной.

### Веха 13 — `/my_prs`

**Цель:** разраб одной командой видит все свои открытые PR со статусами ревью.

```
/my_prs
📋 Ваши открытые PR (4):
  ✅ #123 Refactor auth — approved (2/2)
  🔁 #124 Fix payments — changes requested (@maria)
  ⏳ #125 Add metrics — без ревью 8ч
  🔴 #126 Migrate db — без ревью 32ч ⚠️
```

Источник: GraphQL `GithubPullRequests` фильтр по `author = ваш github_login`.
**Сложность:** S.

### Веха 14 — `/pending_reviews`

**Цель:** ревьюер не забывает что у него на ревью.

```
/pending_reviews
👀 Ждут вашего ревью (3):
  🔴 #100 Iv — 36ч (срочно!)
  🟡 #115 Petr — 8ч
  🟢 #120 Maria — 1ч
```

Сортировка по возрасту, маркеры по порогам (24ч/8ч).
**Сложность:** S.

### Веха 15 — CI fail → ЛС автору

**Цель:** автор PR сразу узнаёт что его коммит сломал CI, не ища в общем чате.

**Триггер:** webhook `workflow_run` action=`completed`, conclusion=`failure`.
**Действие:**
- Найти автора последнего коммита в ветке
- Если он привязан → ЛС: `🔴 Workflow "Tests" упал в вашем PR #123. [Открыть прогон]`

**Не отменяет** существующее уведомление в групповой чат — добавляется к нему.
**Сложность:** S.

### Веха 16 — `/review_load` + алерт перегруза

**Цель:** лид видит балансировку и получает алерт когда кто-то перегружен.

`/review_load` — таблица за последние 7 дней:
```
@ivan_lead  — 12 ревью получено / 4 сделано
@petr       — 2 / 8
@maria      — 1 / 5
...
```

**Алерт лиду:** если у одного юзера >`REVIEW_OVERLOAD_THRESHOLD` (дефолт 5) PR одновременно ждут его ревью — ЛС лиду: `⚠️ @petr перегружен ревью (7 PR)`. Дедуп через `notification_log` — не чаще раза в день.

**Конфиг:**
```
REVIEW_OVERLOAD_THRESHOLD=5
```

**Сложность:** M.

---

## Этап 4 — Релиз-менеджмент (роли PM/QA + планирование)

**Сценарий:** в команду приходят роли которые не пишут код (PM, QA), но управляют релизами и тестированием. Сейчас ритуал релиза («когда катим? что катим? кого зовём на созвон?») идёт в чате руками.

### Веха 17 — Роли PM и QA через `/register`

**Цель:** регистрация для PM/QA с правильными правами без расширения ролевой модели на TG-only flow.

**Решение:** все роли регистрируются через GitHub OAuth (так проще архитектурно). На первом шаге `/register` добавляется выбор:

```
Кто вы?
[Developer] [QA] [PR-менеджер]
```

- **Developer** — текущий flow, получает все code-review уведомления
- **QA** — после OAuth: не получает уведомления о коммитах/CI, но получает алерты «карточка пришла в Тестирование»
- **PR-менеджер (PM)** — после OAuth: получает доступ к командам релиз-планирования, не получает code-review уведомления

В БД: добавляются роли `pm` и `qa` в таблицу `roles`. Логика `WhichEventsToReceive` смотрит на роль.

**Зависит от:** Веха 8 (расширенный onboarding — там же добавляется выбор роли).

### Веха 18 — Команда `/release_plan`

**Цель:** PM или Lead создаёт план релиза одной командой.

**Диалог:**
```
/release_plan
→ Выберите репозитории (multi-select):
   [✓] backend  [✓] frontend  [ ] mobile
→ Дата релиза: 15.05.2026
→ Созвон: 14.05.2026 18:00 (или Пропустить)
→ Ссылка на встречу: https://meet.google.com/abc-defg (или Пропустить)
→ Заметка: "Релиз 1.4 — auth + payments"
→ Опубликовать в чат: [Выберите чат]
→ ✅ Создать план
```

**Multi-repo:** один план может включать несколько репо (синхронный релиз). Один PM может создавать несколько параллельных планов.

**Кто может:** роли `pm`, `lead` (admin). Проверка прав в command guard.

**Анонс в выбранный чат:**
```
📅 План релиза
Репо: backend, frontend
Дата: 15.05.2026
Созвон: 14.05 в 18:00 МСК
🔗 Встреча: https://meet.google.com/abc-defg
📝 Релиз 1.4 — auth + payments
В релизе: 12 PR
Создал: @ivan_pm
```

**Сложность:** L (большой диалог + права + анонс + БД).

### Веха 19 — `/releases`, `/release_edit`, `/release_cancel`

- `/releases` — список текущих и предстоящих планов (свои + чужие в репах юзера)
- `/release_edit <id>` — изменить дату/время/ссылку/заметку (только PM создатель или Lead)
- `/release_cancel <id>` — отменить (с подтверждением + анонсом «❌ Релиз 15.05 отменён» в тот же чат)

**Сложность:** M.

### Веха 20 — Авто-уведомления по релизу

**Cron каждые 15 минут:** проверяет все активные `release_plans`.

| Триггер | Действие |
|---|---|
| `now() == call_datetime - 1h` | в чат: `🔔 Релизный созвон через час: [ссылка]` |
| `now() == planned_date - 24h` | в чат: `⏰ Релиз завтра. Эти PR ещё открыты: #X, #Y, #Z`. Лиду ЛС: «4 PR не успеют, точно ОК?» |
| `now() == planned_date 09:00` | в чат: `🚀 Сегодня релиз! В релизе: [список merged PR с last_tag]` |

После релиза `status` → `done` либо вручную PM закрывает.

**Дедуп:** в `release_plans` поля `notified_24h_at`, `notified_call_at`, `notified_release_day_at` — чтобы не слать дважды.

**Сложность:** M.

### Веха 21 — `/test_queue` для QA

**Цель:** QA не лезет в Kaiten — видит свою очередь в боте.

```
/test_queue
🧪 На тестировании сейчас (5):
  • T-456 Auth flow refactor — пришло 2ч назад
  • T-450 Payments fix — 1 день
  • T-440 Migration — 3 дня ⚠️
```

ЛС-нотификация QA при попадании карточки в колонку «Тестирование» (бот уже двигает её при merge — добавляем послемувный пинг).

**Сложность:** S.

### Что в БД (миграции)

```
release_plans
  ├── id (PK)
  ├── planned_date (DATE)
  ├── call_datetime (DATETIME, nullable)
  ├── meeting_url (VARCHAR, nullable)
  ├── note (TEXT, nullable)
  ├── status (ENUM: planned | cancelled | done)
  ├── announce_chat_id (BIGINT — куда постили анонс)
  ├── notified_24h_at (DATETIME, nullable)
  ├── notified_call_at (DATETIME, nullable)
  ├── notified_release_day_at (DATETIME, nullable)
  ├── created_by_user_id (FK → users)
  └── created_at, updated_at

release_plan_repositories  -- M:N релиз ↔ репозитории
  ├── release_plan_id (FK)
  └── repository_id (FK)

roles  -- добавляются записи 'pm', 'qa' (если ещё нет)
```

---

## Этап 5+: TBD

По мере появления новых процессов в команде.
