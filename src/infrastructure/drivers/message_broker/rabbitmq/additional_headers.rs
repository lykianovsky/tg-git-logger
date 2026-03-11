use std::fmt;

pub enum RabbitMQMessageBrokerAdditionalHeader {
    ErrorHistory,
    DeadReason,
    DeadAt,
}

impl fmt::Display for RabbitMQMessageBrokerAdditionalHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            RabbitMQMessageBrokerAdditionalHeader::ErrorHistory => "x-error-history",
            RabbitMQMessageBrokerAdditionalHeader::DeadReason => "x-dead-reason",
            RabbitMQMessageBrokerAdditionalHeader::DeadAt => "x-dead-at",
        };
        write!(f, "{}", s)
    }
}
