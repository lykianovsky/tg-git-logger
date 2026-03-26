---
name: refactor-safe
description: Безопасный рефакторинг с гарантией сохранения функциональности через тесты
context: refactoring
agent: auto
allowed-tools: Bash(*), Read, Write
priority: medium
---

# Safe Refactoring Skill

## ПРИНЦИПЫ FSD ПРИ РЕФАКТОРИНГЕ
- Slice не должен импортировать из другого slice
- `index.tsx` — только реэкспорты, никакой логики
- `ui/` — только .tsx, `lib/` — только .ts
- Используй slice-level stores, не global store (deprecated)

## ПРОЦЕСС

### PHASE 1: BASELINE
1. Запусти `npm run test:run` — должны пройти (baseline)
2. Если тестов нет — напиши их для текущего поведения
3. Зафиксируй покрытие

### PHASE 2: INCREMENTAL CHANGES
1. Маленькие атомарные изменения
2. После каждого изменения — `npm run test:run`
3. Если тест упал — откати изменение
4. Только зелёные тесты!

### PHASE 3: VALIDATE
1. Все тесты должны пройти
2. Coverage не должно упасть
3. TypeScript без ошибок: `npx tsc --noEmit`
4. Lint чистый: `npm run lint`

## REFACTORING CHECKLIST

- [ ] Тесты для исходного кода exist/created
- [ ] Baseline тесты green
- [ ] Каждый step минимальный
- [ ] Тесты запускались после каждого шага
- [ ] TypeScript errors = 0
- [ ] Lint errors = 0
- [ ] FSD слои соблюдены

## INVOCATION
`/refactor [file_path]` — guided refactoring session
`/refactor-pattern [pattern]` — применить конкретный паттерн
