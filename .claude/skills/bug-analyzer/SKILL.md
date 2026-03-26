---
name: bug-analyzer
description: Анализ ошибок, определение root cause методом 5 Whys, предложение исправлений
context: debugging
agent: auto
allowed-tools: Bash(*), Read, Write
priority: high
---

# Bug Analysis & Root Cause Skill

## МЕТОДОЛОГИЯ: 5 Whys

## ИНФОРМАЦИЯ ДЛЯ АНАЛИЗА
- Stack trace
- Шаги воспроизведения
- Expected vs actual поведение
- Браузер / окружение
- Последние изменения в коде

## ТИПИЧНЫЕ ПАТТЕРНЫ В ЭТОМ ПРОЕКТЕ

**MobX:**
- Компонент не обёрнут в `observer()` — не реагирует на изменения
- `enableStaticRendering` не вызван — утечки в SSR

**Next.js SSR:**
- Обращение к `window`/`document` вне `useEffect`
- Несовпадение hydration (server vs client render)

**GraphQL (URQL):**
- Устаревшие types после изменения схемы — запусти `npm run generate:gql:scheme`
- Неправильные variables в query/mutation

**i18n:**
- Ключ перевода не найден — проверь `locales/ru/`
- Namespace не подключён в `i18n.js` для данной страницы

## ВЫХОДНОЙ ФОРМАТ

```
🐛 BUG ANALYSIS

ERROR: [Сообщение об ошибке]
SEVERITY: [Critical/High/Medium/Low]
ЗАТРОНУТО: [Страницы/функции]

ROOT CAUSE (5 Whys):
1. СИМПТОМ: ...
   ПОЧЕМУ? → ...
2. ПРИЧИНА 1: ...
   ПОЧЕМУ? → ...
3. ПРИЧИНА 2: ...
   ПОЧЕМУ? → ...

НЕМЕДЛЕННОЕ ИСПРАВЛЕНИЕ:
- [Файл: строка] — что исправить

ДОЛГОСРОЧНО:
1. ...

ТЕСТ ДЛЯ ВОСПРОИЗВЕДЕНИЯ:
[Код теста, который падает до фикса]
```

## INVOCATION
`/bug-analyzer` — интерактивная сессия
`/analyze-error "[error message]"` — быстрый анализ
