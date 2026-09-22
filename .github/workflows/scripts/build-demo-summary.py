"""Собирает отчёт в формате json-репортёра Playwright.

Нужен только для проверки управления тестами из бота: блок `demo/broken` всегда даёт
упавшие тесты, остальные — зелёные. Настоящие тесты этот прогон не запускает.
"""

import json
import os
import random

FAILING_BLOCK = "demo/broken"
DEMO_SUITES = {
    "demo/smoke": [
        ("вход по паролю", "e2e/tests/demo/smoke/login.spec.ts"),
        ("главная открывается", "e2e/tests/demo/smoke/home.spec.ts"),
        ("поиск находит отель", "e2e/tests/demo/smoke/search.spec.ts"),
    ],
    "demo/broken": [
        ("бронирование сохраняется", "e2e/tests/demo/broken/booking.spec.ts"),
        ("цена пересчитывается", "e2e/tests/demo/broken/price.spec.ts"),
    ],
    "api/health": [
        ("сервис отвечает", "e2e/tests/api/health/ping.spec.ts"),
    ],
}
ERRORS = [
    "Error: expect(locator).toBeVisible() failed\n\nLocator: getByRole('dialog')",
    "TimeoutError: locator.click: Timeout 30000ms exceeded.",
]


def selected_blocks(scope: str) -> list:
    """Аргументы прогона — это путь блока; пусто значит «все тесты»."""
    if not scope.strip():
        return list(DEMO_SUITES)

    return [block for block in DEMO_SUITES if block in scope] or list(DEMO_SUITES)


def build_report(blocks: list) -> dict:
    suites = []
    passed = 0
    failed = 0

    for block in blocks:
        specs = []

        for index, (title, file) in enumerate(DEMO_SUITES[block]):
            is_failed = block == FAILING_BLOCK
            status = "unexpected" if is_failed else "expected"
            results = [{"error": {"message": random.choice(ERRORS)}}] if is_failed else []

            if is_failed:
                failed += 1
            else:
                passed += 1

            specs.append(
                {
                    "title": title,
                    "ok": not is_failed,
                    "file": file,
                    "tests": [
                        {
                            "projectName": block.split("/")[0],
                            "status": status,
                            "results": results,
                        }
                    ],
                }
            )

        suites.append({"title": block, "file": specs[0]["file"], "specs": specs})

    return {
        "stats": {
            "duration": round(random.uniform(20.0, 90.0), 1),
            "expected": passed,
            "unexpected": failed,
            "flaky": 0,
            "skipped": 0,
        },
        "suites": suites,
    }


if __name__ == "__main__":
    print(json.dumps(build_report(selected_blocks(os.environ.get("E2E", ""))), ensure_ascii=False))
