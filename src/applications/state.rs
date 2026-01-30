use sea_orm::DatabaseConnection;

pub struct ApplicationState {
    pub db: DatabaseConnection,
}