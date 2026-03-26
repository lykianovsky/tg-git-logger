---
name: security-audit
description: Аудит безопасности — OWASP Top 10, уязвимости зависимостей, обработка данных
context: security
agent: auto
allowed-tools: Bash(*), Read, Write
priority: high
---

# Security Audit Skill

## ПРОВЕРКИ ДЛЯ ЭТОГО ПРОЕКТА

### 1. XSS (приоритет высокий — React + AntD)
- Использование `dangerouslySetInnerHTML` без DOMPurify
- Прямая вставка user input в DOM
- Убедись что DOMPurify (`library/`) применяется везде где нужно

### 2. АУТЕНТИФИКАЦИЯ / АВТОРИЗАЦИЯ
- Authorization provider (`library/providers/`) проверяет токены?
- Защищённые роуты недоступны без авторизации?
- JWT хранится в httpOnly cookie, не в localStorage?

### 3. API БЕЗОПАСНОСТЬ
- Sensitive данные не логируются
- API keys не в коде (`library/api/gql/`, `library/api/rest/`)
- URQL exchanges не раскрывают credentials

### 4. ENVIRONMENT VARIABLES
- Секреты в `.env.*` файлах, не в коде
- `.env.local` в `.gitignore`
- Проверь что нет `NEXT_PUBLIC_` переменных с секретами

### 5. ЗАВИСИМОСТИ
```bash
npm audit --audit-level=moderate
```

### 6. COOKIES
- `js-cookie` использует secure + sameSite флаги?
- Session tokens правильно expire?

## OUTPUT FORMAT

```
🔒 SECURITY AUDIT

━━━ CRITICAL (немедленно исправить) ━━━
[Список с расположением файлов]

━━━ HIGH ━━━
[Список]

━━━ DEPENDENCY VULNERABILITIES ━━━
[npm audit output]

━━━ РЕКОМЕНДАЦИИ ━━━
1. [По приоритету]

ИТОГОВЫЙ СЧЁТ: [X/10]
```

## INVOCATION
`/security-audit` — полный аудит проекта
`/security-check [file]` — конкретный файл
