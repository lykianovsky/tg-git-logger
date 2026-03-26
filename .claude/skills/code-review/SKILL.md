---
name: code-review
description: Комплексный анализ кода на производительность, безопасность, качество и соответствие стандартам проекта
context: reviewing
agent: auto
allowed-tools: Bash(*), Read, Write
priority: high
---

# Comprehensive Code Review Skill

## ЗАДАЧА
Выполни глубокий анализ кода по всем направлениям.

## СТАНДАРТЫ ПРОЕКТА
- TypeScript strict mode, no implicit any
- ESLint как единственный форматтер для JS/TS/TSX
- 2-space indent, single quotes, no semicolons
- FSD архитектура: слои не импортируют из вышележащих слоёв
- MobX stores через `createSingletonStore` (не global store)
- Slice структура: `ui/` (tsx), `lib/` (ts), `content.tsx`, `index.tsx`

## ОБЛАСТИ ПРОВЕРКИ

### 1. БЕЗОПАСНОСТЬ
- Input validation и санитизация (DOMPurify для HTML)
- XSS/CSRF уязвимости
- Утечки sensitive данных (API keys в коде)
- Dependency vulnerabilities

### 2. ПРОИЗВОДИТЕЛЬНОСТЬ (React/Next.js)
- Лишние re-renders (missing memo, неправильные deps в useEffect)
- N+1 GraphQL/REST запросы
- Bundle size — импорты из AntD, date-fns, Ramda (используй named imports)
- Memory leaks (unsubscribed MobX reactions, event listeners)

### 3. КОД КАЧЕСТВО
- Code duplication
- Соответствие FSD слоёв (slice не импортирует из другого slice)
- Function length >50 lines — флаг
- TypeScript типизация (избегать `any`)
- Обработка ошибок (try/catch, загрузочные состояния)

### 4. REACT/NEXT.JS СПЕЦИФИКА
- `observer()` обёртка на компонентах, читающих MobX
- Правильный список зависимостей хуков
- SSR совместимость (`enableStaticRendering` в store)
- i18n через `useTranslation` / `next-translate`

## ВЫХОДНОЙ ФОРМАТ

```
# Code Review: [FILE_NAME]

## 🔴 КРИТИЧЕСКИЕ ПРОБЛЕМЫ (обязательно исправить)
- [Проблема + объяснение]

## 🟠 ВАЖНЫЕ ПРОБЛЕМЫ (исправить желательно)
- [Проблема + объяснение]

## 🟡 НЕЗНАЧИТЕЛЬНЫЕ (nice to have)
- [Проблема + объяснение]

## ✅ ХОРОШО СДЕЛАНО
- [Что сделано правильно]

## РЕКОМЕНДАЦИИ
1. [По приоритету]
```

## INVOCATION
`/code-review` — интерактивный режим (укажи файл)
`/code-review [path]` — конкретный файл или директория
