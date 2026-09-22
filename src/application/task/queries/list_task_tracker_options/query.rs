/// Что запрашиваем у трекера: людей и теги — для формы карточки,
/// спейсы, доски и колонки — чтобы знать, куда карточки класть
pub enum ListTaskTrackerOptionsQuery {
    Users,
    Tags,
    Spaces,
    Boards { space_id: i32 },
    Columns { board_id: i32 },
}
