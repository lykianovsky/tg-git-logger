---
name: git-workflow
description: Управление git workflow — branches, commits, PR согласно стандартам проекта
context: version-control
agent: auto
allowed-tools: Bash(git), Read, Write
priority: high
---

# Git Workflow Management Skill

## СТАНДАРТЫ ПРОЕКТА

### Формат коммита (ОБЯЗАТЕЛЬНО)
```
<task-id> <tag>: <описание>
```

**Task ID** (один из):
- `ZB-<ID>` — основные задачи
- `ZBH-<ID>` — задачи гостиничной части
- `PMS-<ID>` — PMS задачи

**Tags:** `feat`, `fix`, `hotfix`, `test`, `refactor`, `docs`

**Примеры:**
```
ZB-1234 feat: Добавил форму обратной связи
ZB-5678 fix: Исправил ошибку при загрузке бронирований
ZBH-9999 refactor: Переработал store для отелей
```

### Формат PR
```
<tag>(<project>-<task-id>)-<short-description>
```
Пример: `feat(ZB-1234)-add-feedback-form`

**В описании PR обязательно:** ссылка на задачу в формате `(ZB-1234)`

## КОМАНДЫ

### /git-create-branch [type]/[name]
```bash
git checkout -b feat/ZB-1234-feature-name
```

### /git-commit-check
- Проверь что commit message соответствует формату
- Наличие task ID
- Правильный tag

### /git-prepare-pr
- Сформируй название PR
- Сгенерируй описание с ссылкой на задачу
- Проверь checklist перед merge

### /git-status
- `git log --oneline -10` — последние коммиты
- `git diff --stat` — что изменилось
- Текущая ветка и её статус

## PRE-PUSH CHECKLIST
- [ ] Все тесты проходят: `npm run test:run`
- [ ] Lint чистый: `npm run lint`
- [ ] TypeScript OK: `npx tsc --noEmit`
- [ ] Commit message в правильном формате
- [ ] Task ID указан

## INVOCATION
`/git-create-branch [type/name]` — создать ветку
`/git-commit-check` — проверить коммиты
`/git-prepare-pr` — подготовить PR
