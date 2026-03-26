---
name: test-suite
description: Управление тестами — запуск, анализ покрытия, генерация тестов
context: testing
agent: auto
allowed-tools: Bash(npm), Read, Write
priority: high
---

# Test Suite Management Skill

## СТЕК ТЕСТИРОВАНИЯ
- **Runner:** Vitest (не Jest)
- **Environment:** jsdom
- **Config:** `vitest.config.ts`
- **TypeScript config для тестов:** `tsconfig.test.json`
- **Path aliases:** те же, что в tsconfig.json (`@library/*`, `@slices/*` и т.д.)

## КОМАНДЫ

### `npm test` — watch mode
### `npm run test:run` — однократный запуск
### `npm run test:coverage` — с отчётом покрытия

### /test-run
Запусти все тесты, покажи:
- Passed / Failed / Skipped
- Slow tests (>3 секунды)
- Список упавших тестов с файлом и строкой

### /test-coverage
- Запусти с `npm run test:coverage`
- Покажи файлы с покрытием ниже 70%
- Укажи непокрытые функции

### /test-generate [file_path]
- Проанализируй файл
- Сгенерируй unit тесты для непокрытых функций
- Включи edge cases и negative tests
- Следи за правильными path aliases

### /test-watch
- Запусти `npm test` (watch mode)
- Показывай только failed тесты

## EXPECTED OUTPUT

```
✅ Test Report

Total: 245 tests
✓ Passed: 242 (98.8%)
✗ Failed: 2 (0.8%)
⊘ Skipped: 1 (0.4%)

⚠️ FAILURES:
- src/slices/auth/lib/helpers.test.ts:45
- src/library/utils/date.test.ts:123

📊 COVERAGE: 82.3%
Файлы ниже 70%:
- library/api/rest/models/Payment.ts: 45%

⏱️ SLOW TESTS (>3s):
- Integration suite: 8.2s
```
