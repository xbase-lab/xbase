use process_stream::ProcessItem;
use serde::{Deserialize, Serialize};

/// Representation of Messages that clients needs to process
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Message {
    /// Notify use with a message
    Notify { msg: String, level: MessageLevel },
    /// Log a message
    Log { msg: String, level: MessageLevel },
    /// Execute an task
    Execute(Task),
}

impl Message {
    pub fn notify_error<S: AsRef<str>>(value: S) -> Self {
        Self::Notify {
            msg: value.as_ref().to_string(),
            level: MessageLevel::Error,
        }
    }

    pub fn notify_warn<S: AsRef<str>>(value: S) -> Self {
        Self::Notify {
            msg: value.as_ref().to_string(),
            level: MessageLevel::Warn,
        }
    }

    pub fn notify_trace<S: AsRef<str>>(value: S) -> Self {
        Self::Notify {
            msg: value.as_ref().to_string(),
            level: MessageLevel::Trace,
        }
    }

    pub fn notify_debug<S: AsRef<str>>(value: S) -> Self {
        Self::Notify {
            msg: value.as_ref().to_string(),
            level: MessageLevel::Debug,
        }
    }

    pub fn log_error<S: AsRef<str>>(value: S) -> Self {
        Self::Log {
            msg: value.as_ref().to_string(),
            level: MessageLevel::Error,
        }
    }

    pub fn log_info<S: AsRef<str>>(value: S) -> Self {
        Self::Log {
            msg: value.as_ref().to_string(),
            level: MessageLevel::Error,
        }
    }

    pub fn log_warn<S: AsRef<str>>(value: S) -> Self {
        Self::Log {
            msg: value.as_ref().to_string(),
            level: MessageLevel::Warn,
        }
    }

    pub fn log_trace<S: AsRef<str>>(value: S) -> Self {
        Self::Log {
            msg: value.as_ref().to_string(),
            level: MessageLevel::Trace,
        }
    }

    pub fn log_debug<S: AsRef<str>>(value: S) -> Self {
        Self::Log {
            msg: value.as_ref().to_string(),
            level: MessageLevel::Debug,
        }
    }
}

/// Statusline state
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StatuslineState {
    /// Last task was successful
    Success,
    /// Last task failed
    Failure,
    /// A Request is being processed.
    Processing,
    /// Something is being watched.
    Watching,
    ///  that is currently running.
    Running,
}

/// Tasks that the clients should execute
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Task {
    UpdateStatusline(StatuslineState),
}

/// Message Kind
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum MessageLevel {
    /// Trace Message
    Trace = 0,
    /// Debug Message
    Debug = 1,
    /// Info Message
    Info = 2,
    /// Warn Message
    Warn = 3,
    /// Error Message
    Error = 4,
}

impl From<ProcessItem> for Message {
    fn from(item: ProcessItem) -> Self {
        let is_success = item.is_success();
        match item {
            ProcessItem::Output(value) => {
                if value.to_lowercase().contains("error") {
                    Self::Log {
                        msg: value,
                        level: MessageLevel::Error,
                    }
                } else if value.to_lowercase().contains("warn") {
                    Self::Log {
                        msg: value,
                        level: MessageLevel::Warn,
                    }
                } else {
                    Self::Log {
                        msg: value,
                        level: MessageLevel::Info,
                    }
                }
            }
            ProcessItem::Error(value) => Self::Log {
                msg: value,
                level: MessageLevel::Error,
            },
            ProcessItem::Exit(code) => {
                if is_success.unwrap() {
                    Self::Log {
                        msg: "Success".into(),
                        level: MessageLevel::Info,
                    }
                } else {
                    Self::Log {
                        msg: format!("Exit {code}"),
                        level: MessageLevel::Error,
                    }
                }
            }
        }
    }
}

impl From<String> for Message {
    fn from(value: String) -> Self {
        Self::Notify {
            msg: value,
            level: MessageLevel::Info,
        }
    }
}

impl From<&str> for Message {
    fn from(value: &str) -> Self {
        Self::Notify {
            msg: value.to_string(),
            level: MessageLevel::Info,
        }
    }
}
