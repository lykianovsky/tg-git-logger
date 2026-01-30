#[macro_export]
macro_rules! unwrap_or_return_status {
    ($result:expr, $error_status:expr) => {
        match $result {
            Ok(value) => value,
            Err(e) => {
                tracing::error!("{:?}", e);
                return $error_status;
            }
        }
    };
}