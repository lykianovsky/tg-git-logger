---
name: performance-analyzer
description: Анализ производительности — профилирование, поиск узких мест, оптимизация
context: performance
agent: auto
allowed-tools: Bash(*), Read, Write
priority: medium
---

# Performance Analysis Skill

## МЕТРИКИ ФРОНТЕНДА (Next.js)

### Core Web Vitals (цели)
- LCP (Largest Contentful Paint): < 2.5s
- FID (First Input Delay): < 100ms
- CLS (Cumulative Layout Shift): < 0.1

### Bundle Size
- Анализ: `ANALYZE=true npm run build`
- Цели: первоначальная загрузка < 200KB gzipped

## ТИПИЧНЫЕ ПРОБЛЕМЫ ЭТОГО ПРОЕКТА

### 1. ИМПОРТЫ (высокий impact)
```typescript
// ❌ Плохо - весь Ramda bundle
import * as R from 'ramda'

// ✅ Хорошо - только нужное
import { filter, map } from 'ramda'
```
То же для `date-fns`, `antd`, lodash

### 2. MobX RE-RENDERS
- Компонент без `observer()` — не обновляется
- Компонент с лишними computed в render — лишние подписки
- Создание объектов в render — дестабилизирует MobX

### 3. NEXT.JS SSR
- Лишние `getServerSideProps` вместо `getStaticProps`
- Большие payload в `pageProps`
- Не используется Image компонент Next.js

### 4. REACT
- Отсутствие `React.memo` для тяжёлых компонентов
- Inline function definitions в JSX
- Большие списки без виртуализации (есть `react-virtuoso` в проекте)

## КОМАНДЫ

### /performance-analyze
- Запусти `ANALYZE=true npm run build`
- Найди крупнейшие чанки
- Предложи оптимизации

### /perf-components
- Найди компоненты с потенциальными re-render проблемами
- Проверь правильность observer() применения

### /perf-bundle
- Анализ bundle size
- Список тяжёлых зависимостей
- Возможности code-splitting

## OUTPUT FORMAT

```
📊 Performance Report

Bundle Analysis:
- Total: 850KB (цель: < 500KB)
- Самые большие чанки:
  1. vendor.js: 400KB
  2. antd.js: 200KB

Bottlenecks:
1. AntD импорт без tree-shaking [antd bundle: 200KB]
2. Ramda * import в 5 файлах [+80KB]
3. Список из 1000 элементов без виртуализации

Рекомендации:
1. Настроить tree-shaking для AntD (уже есть transpilePackages)
2. Заменить * imports на named imports
3. Добавить react-virtuoso (уже установлен)

Ожидаемый эффект: -40% bundle size
```

## INVOCATION
`/performance-analyze` — полный анализ
`/perf-bundle` — только bundle
`/perf-components` — Re-render анализ
