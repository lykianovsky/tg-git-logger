---
name: docs-generator
description: Автоматическая генерация и обновление документации из кода
context: documentation
agent: auto
allowed-tools: Bash(*), Read, Write
priority: medium
---

# Documentation Generation Skill

## ДОКУМЕНТАЦИЯ В ПРОЕКТЕ
Существующая документация:
- `README.md` — запуск проекта
- `architecture.md` — FSD архитектура (RU)
- `formatting.md` — правила форматирования (RU)
- `pull-request-and-commit-rules.md` — git workflow (RU)
- `CLAUDE.md` — руководство для Claude Code

## ТИПЫ ГЕНЕРИРУЕМОЙ ДОКУМЕНТАЦИИ

### /docs-api
- GraphQL queries/mutations из `library/api/gql/`
- REST endpoints из `library/api/rest/`
- Параметры запросов и response типы

### /docs-slice [slice-name]
- Документация для конкретного slice
- Экспортируемые компоненты и их props
- Используемые stores и их структура
- Зависимости на library/

### /docs-store
- Все MobX stores и их поля
- Actions и computed значения
- Зависимости между stores

### /docs-update
- Обнови устаревшую документацию
- Синхронизируй с текущим состоянием кода

## OUTPUT
- Документация на **русском языке** (язык проекта)
- Markdown формат
- Включать примеры использования

## INVOCATION
`/docs-generate` — полная генерация
`/docs-api` — только API документация
`/docs-slice [name]` — slice документация
`/docs-update` — обновить существующую
