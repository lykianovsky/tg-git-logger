use teloxide::types::InlineKeyboardButton;

/// Постраничный список кнопок. Нужен там, где длина списка задаётся не нами: блоки тестов,
/// процессы CI, люди и теги трекера. Раньше такие списки либо рисовались целиком (простыня,
/// которая однажды упрётся в лимит разметки), либо молча обрезались на тридцатом элементе —
/// и человек делал вывод, что нужного варианта просто нет.
///
/// Кнопки элементов строит вызывающий: у каждого списка свои данные в callback (номер,
/// идентификатор). Здесь — только разбиение на страницы и ряд перехода.

/// Страница приходит номером в данных кнопки: `page:2`
pub const PAGE_CALLBACK_PREFIX: &str = "page:";
/// Счётчик страниц — тоже кнопка (пустые данные мессенджер не принимает), но ничего не делает
pub const PAGE_COUNTER_CALLBACK: &str = "page:counter";
/// Сколько вариантов показываем на одной странице: вместе с рядом перехода и «Назад»
/// клавиатура остаётся обозримой на телефоне
pub const PAGE_SIZE: usize = 8;

/// Номер страницы из данных кнопки. Счётчик и чужие данные — `None`: нажатие на него
/// ничего не меняет
pub fn parse_page(data: &str) -> Option<usize> {
    if data == PAGE_COUNTER_CALLBACK {
        return None;
    }

    data.strip_prefix(PAGE_CALLBACK_PREFIX)?
        .parse::<usize>()
        .ok()
}

/// Нажали кнопку списка, а не выбора: экран остаётся тем же, что бы ни было в данных.
/// Отделено от `parse_page`, чтобы нажатие на счётчик не считалось выбором варианта
pub fn is_pagination(data: &str) -> bool {
    data.starts_with(PAGE_CALLBACK_PREFIX)
}

pub fn page_count(total: usize) -> usize {
    total.div_ceil(PAGE_SIZE).max(1)
}

/// Ряды одной страницы плюс ряд перехода. Ряда перехода нет, когда страница одна —
/// не занимаем место в клавиатуре ради неактивных кнопок
pub fn paginated_rows(
    buttons: Vec<InlineKeyboardButton>,
    page: usize,
) -> Vec<Vec<InlineKeyboardButton>> {
    let pages = page_count(buttons.len());
    // Список мог укоротиться, пока человек смотрел на старую клавиатуру
    let current = page.min(pages - 1);

    let mut rows: Vec<Vec<InlineKeyboardButton>> = buttons
        .into_iter()
        .skip(current * PAGE_SIZE)
        .take(PAGE_SIZE)
        .map(|button| vec![button])
        .collect();

    if pages == 1 {
        return rows;
    }

    let mut navigation = Vec::new();

    if current > 0 {
        navigation.push(InlineKeyboardButton::callback(
            rust_i18n::t!("telegram_bot.pagination.previous").to_string(),
            format!("{PAGE_CALLBACK_PREFIX}{}", current - 1),
        ));
    }

    navigation.push(InlineKeyboardButton::callback(
        rust_i18n::t!(
            "telegram_bot.pagination.counter",
            page = current + 1,
            pages = pages
        )
        .to_string(),
        PAGE_COUNTER_CALLBACK.to_string(),
    ));

    if current + 1 < pages {
        navigation.push(InlineKeyboardButton::callback(
            rust_i18n::t!("telegram_bot.pagination.next").to_string(),
            format!("{PAGE_CALLBACK_PREFIX}{}", current + 1),
        ));
    }

    rows.push(navigation);
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buttons(count: usize) -> Vec<InlineKeyboardButton> {
        (0..count)
            .map(|index| InlineKeyboardButton::callback(index.to_string(), index.to_string()))
            .collect()
    }

    #[test]
    fn single_page_has_no_navigation_row() {
        let rows = paginated_rows(buttons(PAGE_SIZE), 0);

        assert_eq!(rows.len(), PAGE_SIZE);
    }

    #[test]
    fn first_page_offers_only_next() {
        let rows = paginated_rows(buttons(PAGE_SIZE * 3), 0);
        let navigation = rows.last().expect("ряд перехода");

        assert_eq!(rows.len(), PAGE_SIZE + 1);
        assert_eq!(navigation.len(), 2);
    }

    #[test]
    fn middle_page_offers_both_directions() {
        let rows = paginated_rows(buttons(PAGE_SIZE * 3), 1);
        let navigation = rows.last().expect("ряд перехода");

        assert_eq!(navigation.len(), 3);
    }

    #[test]
    fn last_page_shows_remainder() {
        let rows = paginated_rows(buttons(PAGE_SIZE * 2 + 3), 2);

        // три элемента последней страницы и ряд перехода
        assert_eq!(rows.len(), 4);
    }

    #[test]
    fn page_beyond_last_falls_back_to_last() {
        let rows = paginated_rows(buttons(PAGE_SIZE + 1), 42);

        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn empty_list_gives_no_rows() {
        assert!(paginated_rows(buttons(0), 0).is_empty());
    }

    #[test]
    fn counter_belongs_to_the_list_but_selects_nothing() {
        assert!(is_pagination(PAGE_COUNTER_CALLBACK));
        assert!(is_pagination("page:2"));
        assert!(!is_pagination("opt:2"));
    }

    #[test]
    fn counter_is_not_a_page() {
        assert_eq!(parse_page(PAGE_COUNTER_CALLBACK), None);
        assert_eq!(parse_page("opt:3"), None);
        assert_eq!(parse_page("page:3"), Some(3));
    }
}
