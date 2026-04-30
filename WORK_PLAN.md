# План работы: расширение релиз-менеджмента + `/whoami` + мониторинг

**Статус:** согласован, к реализации.
**Формат поставки:** один общий PR.
**Связанный roadmap:** Вехи 18-20 (`ROADMAP.md`) — этот план их конкретизирует с принятыми решениями.

---

## Часть 1. Расширение релиз-менеджмента

### 1.1 Команда `/releases`

- **Доступ:** только зарегистрированные пользователи (любая роль).
- **Содержимое:** релизы со `status = planned` и `planned_date >= today_msk`, отсортированные по дате.
- **Формат вывода:** для каждого плана — дата (`DD.MM.YYYY` + лейбл "сегодня"/"завтра"/"через N дн"), список репозиториев, время и ссылка созвона (если есть), заметка (если есть), создатель.
- **Inline-кнопки под каждым релизом:**
  - 📝 Редактировать — открывает меню редактирования (доступно только Admin / ProductManager).
  - ❌ Отменить — открывает диалог ввода причины (доступно только Admin / ProductManager).
  - ✅ Закрыть — переводит в статус `done` с подтверждением (доступно только Admin / ProductManager).
- Если у юзера нет прав на действие — на нажатие кнопки бот отвечает "нет прав".

### 1.2 Расширение диалога `/release_plan` (создание)

Сейчас диалог: дата → репозитории. Расширяется так:

1. **Дата релиза** (как сейчас, обязательное).
2. **Выбор репозиториев** (как сейчас, multi-select из существующих).
3. **Дата созвона** (опционально):
   - Кнопка "Использовать дефолт" → подставляется ближайший `RELEASE_PLAN_DEFAULT_CALL_WEEKDAY` до даты релиза.
   - Кнопка "Ввести вручную" → ввод даты в формате `DD.MM.YYYY`.
4. **Время созвона** (опционально):
   - Кнопка "Использовать дефолт" → `RELEASE_PLAN_DEFAULT_CALL_TIME`.
   - Кнопка "Ввести вручную" → ввод в формате `HH:MM` МСК.
5. **Ссылка на встречу** (опционально, можно пропустить).
6. **Заметка** (опционально, можно пропустить).

**Важное правило:** созвон создаётся всегда. Если PM не менял время/дату созвона, подставляются дефолты из `.env`. Полностью убрать созвон можно только через меню редактирования (см. ниже).

### 1.3 Меню редактирования (`/releases` → 📝)

Inline-меню с пунктами:
- Изменить дату релиза
- Изменить дату созвона
- Изменить время созвона
- Удалить созвон (отдельная кнопка — обнуляет `call_datetime` и `meeting_url`)
- Изменить ссылку на встречу
- Изменить заметку
- Изменить список репозиториев (multi-select из существующих)
- ↩️ Назад

Доступ: только `Admin` или `ProductManager`. Часовой пояс ввода времени — фиксированно МСК.

### 1.4 Отмена релиза (`/releases` → ❌)

1. Бот: "Введите причину отмены".
2. PM вводит текст.
3. Бот: подтверждение "Отменить релиз DD.MM по причине '...'? [Да/Нет]".
4. После "Да": `status = cancelled`.
5. В `announce_chat_id` уходит сообщение:
   > ❌ **Релиз DD.MM.YYYY отменён**
   > Отменил: @user
   > Причина: ...

### 1.5 Закрытие релиза (`/releases` → ✅)

1. Подтверждение "Закрыть релиз DD.MM? [Да/Нет]".
2. После "Да": `status = done`.
3. **Без анонса** в чат.

### 1.6 Утреннее напоминание о релизе

- **Cron:** `0 0 7 * * *` (07:00 UTC = 10:00 МСК).
- **Условия отправки:** `status = planned` AND `planned_date == today_msk` AND `notified_release_day_at IS NULL`.
- **Канал:** `announce_chat_id` плана.
- **Содержимое:** "🚀 Сегодня релиз!" + репо + заметка. **Созвон НЕ упоминается** (для него — отдельное напоминание за 1 час, см. 1.7).
- После отправки выставляется `notified_release_day_at = now()`.

### 1.7 Напоминание о созвоне за 1 час

- **Cron:** `0 */15 * * * *` (раз в 15 минут — окно проверки).
- **Условия:** `status = planned` AND `call_datetime` between `now()` и `now() + 1h` AND `notified_call_at IS NULL`.
- **Канал:** `announce_chat_id`.
- **Содержимое:** "🔔 Релизный созвон через час", ссылка на встречу (если есть).
- После отправки выставляется `notified_call_at = now()`.

### 1.8 Конфигурация (`.env`)

```env
RELEASE_PLAN_DEFAULT_CALL_WEEKDAY=monday   # понедельник
RELEASE_PLAN_DEFAULT_CALL_TIME=16:00       # 16:00 по МСК
```

Часовой пояс созвона жёстко МСК (`Europe/Moscow`).

### 1.9 Затрагиваемые файлы (Часть 1)

- `src/config/application/mod.rs` — новая sub-config `ApplicationReleasePlanConfig`.
- `src/domain/release_plan/repositories/release_plan_repository.rs` — методы `find_upcoming`, `find_due_for_release_day_reminder`, `find_due_for_call_reminder`, `update_*`, `cancel`, `complete`, `set_repositories`.
- `src/infrastructure/repositories/mysql/release_plan.rs` — реализации.
- `src/application/release_plan/queries/get_upcoming_release_plans/{query,executor,response,error,mod}.rs` (новый).
- `src/application/release_plan/commands/{update_release_plan, cancel_release_plan, complete_release_plan, send_release_day_reminders, send_call_reminders}/...` — 5 новых executors.
  - `update_release_plan` — единая команда с `ReleasePlanPatch` enum (по аналогии с `UpdateUserPreferencesExecutor`):
    ```rust
    pub enum ReleasePlanPatch {
        SetPlannedDate { date: NaiveDate },
        SetCallDateTime { datetime: DateTime<Utc> },
        ClearCallDateTime,
        SetMeetingUrl { url: String },
        ClearMeetingUrl,
        SetNote { text: String },
        ClearNote,
        SetRepositories { ids: Vec<RepositoryId> },
    }
    ```
- `src/delivery/bot/telegram/commands/releases.rs` — handler.
- `src/delivery/bot/telegram/commands/builder.rs`, `commands/mod.rs` — регистрация команды.
- `src/delivery/bot/telegram/dialogues/release_plan/mod.rs` — расширение state machine для шагов созвона/ссылки/заметки.
- `src/delivery/bot/telegram/dialogues/release_plan_settings/mod.rs` (новый) — диспетчер меню редактирования.
- `src/delivery/bot/telegram/dialogues/cancel_release_plan/mod.rs` (новый) — диспетчер отмены.
- `src/delivery/bot/telegram/keyboards/actions/release_plan.rs` — расширение action enum.
- `src/delivery/bot/telegram/dialogues/mod.rs`, `src/delivery/bot/telegram/mod.rs` — регистрация диспетчеров.
- `src/bootstrap/executors.rs` — регистрация новых executors.
- `src/delivery/scheduler/mod.rs` — два новых cron-job.
- `locales/ru.json` — новые ключи.
- `.env.example` — новые переменные.

**Оценка:** ~50 файлов.

---

## Часть 2. Команда `/whoami`

### 2.1 Поведение

- **Доступ:** только зарегистрированные пользователи.
- **Что показывает (один экран):**
  - GitHub: логин + имя.
  - Роли (Admin / Developer / ProductManager / QualityAssurance).
  - DND: окно тихих часов (или "по умолчанию").
  - Часовой пояс: `Europe/Moscow` (или кастомный).
  - Vacation: до даты, либо "выкл".
  - Snooze: до времени, либо "выкл".
  - Привязанные репозитории — список `owner/name`.

### 2.2 Затрагиваемые файлы

- `src/application/user/queries/get_user_overview/{query,executor,response,error,mod}.rs` (новый, агрегирует данные из существующих repo).
- `src/application/user/queries/mod.rs`.
- `src/delivery/bot/telegram/commands/whoami.rs` (новый handler).
- `src/delivery/bot/telegram/commands/builder.rs`, `commands/mod.rs`.
- `src/bootstrap/executors.rs`.
- `locales/ru.json`.

**Оценка:** ~7 файлов.

---

## Часть 3. Мониторинг (Prometheus + Grafana + Loki + Promtail)

### 3.1 Архитектура

- **Метрики:** Prometheus периодически скрапит `/metrics` бота.
- **Логи:** Promtail вытаскивает stdout контейнера → пишет в Loki.
- **Визуализация:** Grafana с готовым дашбордом "GitEye Overview" + datasources Prometheus и Loki.
- **Доступ:** только локально (`127.0.0.1:3000`), наружу не публикуется.

### 3.2 Метрики (полная инструментация)

**HTTP (axum middleware):**
- `http_requests_total{method, route, status}`
- `http_request_duration_seconds{method, route}` (histogram, P50/P95/P99)

**GitHub webhooks:**
- `webhook_received_total{event_type}` (pull_request, release, push, workflow_run, review, ...)
- `webhook_processing_duration_seconds{event_type}`
- `webhook_signature_invalid_total`

**RabbitMQ jobs:**
- `job_processed_total{queue, status}` (success, retry, fail)
- `job_processing_duration_seconds{queue}`
- `job_queue_size{queue, priority}` (gauge — длина каждой приоритетной очереди)
- `job_retry_total{queue, reason}`

**Telegram client:**
- `telegram_send_total{status}` (sent, failed, blocked)
- `telegram_send_duration_seconds`

**GitHub API client:**
- `github_api_requests_total{method, status}`
- `github_api_request_duration_seconds{method}`
- `github_api_rate_limit_remaining` (gauge)

**Kaiten client:**
- `kaiten_api_requests_total{status}`
- `kaiten_api_request_duration_seconds`

**MySQL repositories:**
- `db_query_total{repo, op, status}`
- `db_query_duration_seconds{repo, op}` (histogram)

**Health pings:**
- `health_ping_status{service}` (1=up, 0=down)
- `health_ping_response_time_seconds{service}`

**Бизнес-метрики (gauges, обновляются периодически):**
- `users_total{status}` (active, inactive)
- `users_on_vacation_total`
- `repositories_total`
- `release_plans_active_total`
- `release_plans_today_total`

**Errors:**
- `errors_total{module, kind}`

### 3.3 Дашборд "GitEye Overview"

Панели:
1. **Общая активность:** webhook events / минуту по типам (stacked).
2. **Telegram отправки:** success/fail rate, P95 latency.
3. **RabbitMQ очереди:** длина по приоритетам, retry rate, processing time P95.
4. **HTTP latency:** P50/P95/P99 по роутам.
5. **GitHub API:** rate-limit remaining, latency, ошибки.
6. **Kaiten API:** доступность, latency.
7. **MySQL:** медленные запросы (P95 > порога), errors.
8. **Health pings:** статус всех сервисов.
9. **Бизнес-метрики:** пользователи / отпуска / релизы.
10. **Ошибки:** total errors по модулям.
11. **Логи:** панель Loki с фильтром по уровню (errors последние 1h).

### 3.4 Логи

- Формат: JSON (через `tracing-subscriber` с JSON-форматтером).
- Лейблы Loki: `level`, `module`, `event`, `request_id`.
- **Срок хранения: 30 дней.**
- Дашборд "Logs Explorer" с фильтрами level + module + текстовый поиск.

### 3.5 Затрагиваемые файлы

**Инфраструктура мониторинга (новые файлы):**
- `docker-compose.monitoring.yml`
- `monitoring/prometheus/prometheus.yml`
- `monitoring/loki/loki-config.yml`
- `monitoring/promtail/promtail-config.yml`
- `monitoring/grafana/provisioning/datasources/datasources.yml`
- `monitoring/grafana/provisioning/dashboards/dashboards.yml`
- `monitoring/grafana/dashboards/giteye-overview.json`
- `monitoring/grafana/dashboards/logs-explorer.json`

**Makefile:**
```
monitoring-up:        запуск стека
monitoring-down:      остановка
monitoring-logs:      просмотр логов стека
monitoring-restart:   рестарт
```

**Cargo.toml:**
- `prometheus-client` или `axum-prometheus`
- `tracing-subscriber` с feature `json` (если ещё нет)

**Код (полная инструментация — `~30 файлов`):**
- `src/infrastructure/metrics/{mod, registry, http_middleware, job_wrapper}.rs` (новый модуль).
- `src/delivery/http/axum/mod.rs` — добавить `/metrics` endpoint и middleware.
- `src/delivery/jobs/consumers/*` — обернуть consume в job-метрики.
- `src/infrastructure/integrations/version_control/github/client.rs` — обернуть HTTP-клиент.
- `src/infrastructure/integrations/task_tracker/kaiten/*.rs` — обернуть.
- `src/infrastructure/integrations/health_check/*.rs` — экспорт метрик пингов.
- `src/infrastructure/services/notification/*.rs` — обернуть Telegram-отправку.
- `src/infrastructure/repositories/mysql/*.rs` — макрос/обёртка для DB query metrics.
- `src/main.rs` или `src/bootstrap/mod.rs` — настроить tracing JSON output, инициализировать registry.
- `src/delivery/scheduler/mod.rs` — отдельный job для обновления business-метрик (раз в минуту).

**Конфиг:**
- `.env.example` — `METRICS_ENABLED`, `LOG_FORMAT=json`.
- `Dockerfile.prod` / `docker-compose.prod.yml` — пробросить порт 9090 (метрики) во внутреннюю сеть.

**Оценка:** ~30-40 файлов.

---

## Итого

**Объём:** ~90 файлов (~50 + 7 + 35).
**Поставка:** один общий PR.

---

## Идеи на будущее (не входят в этот PR)

Зафиксировано во время анализа. Расположено по убыванию ценности.

### Близко к текущему скоупу
1. **Команда `/release_history`** — список последних done/cancelled релизов за месяц. Сейчас закрытые релизы пропадают из `/releases`.
2. **Постмортем релиза** — после `done` бот шлёт PM ЛС: "Релиз закрыт за N часов от запланированной даты, X PR, Y инцидентов". Требует Веху 22 (метрики).
3. **Авто-добавление PR в release_plan** — если PR закрыт между `prev_release.done_at` и `next_release.planned_date`, он автоматически попадает в "что будет в релизе".

### Из roadmap, требуют отдельного PR
4. **Вехи 9-12** — авто-теги ревьюеров, re-tag stale в чате, уведомление о результате ревью, conflict detection.
5. **Веха 13 — `/my_prs`**, **Веха 14 — `/pending_reviews`**.
6. **Веха 15 — CI fail → ЛС автору**.
7. **Веха 16 — `/review_load`** + алерт перегруза.
8. **Веха 21 — `/test_queue`** для QA.

### Командные утилиты
9. **`/status`** — пинг всех healh-pings + Kaiten + GitHub API на лету. Полезно когда "что-то молчит".
10. **`/digest_now`** — отправить дайджест прямо сейчас, не ждать расписания.
11. **`/admin → метрики`** — недельная сводка по уведомлениям, retry rate, среднее время доставки.
12. **`/admin → debug webhook`** — последние 10 webhook payload-ов с timestamp/result для дебага.
13. **Slash-кнопка "🐛 Bug report" в `/start`** — отправляет в админ-чат описание + chat-id.

### Технические улучшения
14. **`repository_id` в `notification_log.kind`** — per-repo дедуп вместо глобального.
15. **Тесты `MessageBuilder`** — нет ни одного теста на edge-cases (HTML-escape, max_length, юникод).
16. **Идемпотентность scheduler-job** — паттерн "флаг `notified_*_at`" из release_plan распространить на digest и pending notifications, чтобы перезапуск приложения в момент cron-tick не давал дубликат.
17. **Sentry** — облачный сбор ошибок с дедупом и алертами. Альтернатива/дополнение к Loki+Grafana. (Пользователь явно отказался — фиксирую как опцию на будущее.)

### Расширения мониторинга
18. **Tempo + OpenTelemetry трассировка** — сквозной путь webhook → job → Telegram. В текущем PR закладывается базовая metrics+logs архитектура; трассировку добавим точечно когда понадобится дебажить latency.
19. **Алерты в Grafana** — отдельно после стабилизации дашбордов (нужно понять пороги).
