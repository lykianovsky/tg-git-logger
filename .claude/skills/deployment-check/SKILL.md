---
name: deployment-check
description: Pre-deployment валидация — все ли готово к продакшену
context: deployment
agent: auto
allowed-tools: Bash(*), Read, Write
priority: high
---

# Deployment Readiness Skill

## PRE-DEPLOYMENT CHECKLIST

### КОД КАЧЕСТВО
- [ ] `npm run test:run` — все тесты проходят
- [ ] `npm run lint` — 0 ошибок
- [ ] `npx tsc --noEmit` — 0 TypeScript ошибок
- [ ] Нет `console.log` / `debugger` в production коде
- [ ] Нет захардкоженных credentials

### ENVIRONMENT VARIABLES
- [ ] Все необходимые переменные есть в `.env.production`
- [ ] `.env.local` не попал в git
- [ ] `NEXT_PUBLIC_` переменные не содержат секретов
- [ ] API endpoints указывают на production

### BUILD
- [ ] `npm run build` завершился без ошибок
- [ ] Bundle size в норме (нет неожиданного роста)
- [ ] Нет broken imports

### БЕЗОПАСНОСТЬ
- [ ] `npm audit --audit-level=high` — 0 critical/high
- [ ] Нет exposed API keys в коде

### API СХЕМЫ
- [ ] GraphQL типы актуальны: `npm run generate:gql:scheme`
- [ ] REST типы актуальны: `npm run generate:rest:scheme`
- [ ] Нет устаревших API вызовов

### DOCKER (если деплой через Docker)
- [ ] `docker-compose.yaml` актуален
- [ ] `makefile` команды работают

## КОМАНДЫ ВАЛИДАЦИИ

```bash
# Полная проверка перед деплоем
npm run test:run && npm run lint && npx tsc --noEmit && npm run build
```

## OUTPUT

```
✅ DEPLOYMENT READY

Tests: 245/245 ✓
Lint: 0 errors ✓
TypeScript: 0 errors ✓
Build: success ✓
Security: 0 critical ✓
Env vars: configured ✓

СТАТУС: Готово к деплою
```

## INVOCATION
`/deployment-check` — полная валидация
`/deploy-ready` — быстрая проверка готовности
`/deploy-build` — только build check
