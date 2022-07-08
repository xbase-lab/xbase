#![allow(dead_code)]
use super::{Message, MessageLevel, StatuslineState, Task};
use std::path::PathBuf;

impl super::Broadcast {
    /// Explicitly Abort/Consume logger
    pub fn abort(&self) {
        self.abort.notify_waiters();
    }

    /// Get a reference to the logger's project root.
    #[must_use]
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Get a reference to the logger's log path.
    #[must_use]
    pub fn address(&self) -> &PathBuf {
        &self.address
    }

    pub fn log_step<S: AsRef<str>>(&self, msg: S) {
        let sep = ".".repeat(73);
        self.tx.send(Message::log_info(msg)).ok();
        self.tx.send(Message::log_info(&sep)).ok();
    }

    pub fn log_separator(&self) {
        let sep = ".".repeat(73);
        tracing::info!("{sep}");
        self.tx.send(Message::log_info(&sep)).ok();
    }

    pub fn success<S: AsRef<str>>(&self, msg: S) {
        tracing::info!("{}", msg.as_ref());
        self.tx
            .send(Message::Notify {
                msg: msg.as_ref().into(),
                level: MessageLevel::Success,
            })
            .ok();
    }

    pub fn info<S: AsRef<str>>(&self, msg: S) {
        tracing::info!("{}", msg.as_ref());
        self.tx.send(msg.as_ref().into()).ok();
    }

    pub fn error<S: AsRef<str>>(&self, msg: S) {
        tracing::error!("{}", msg.as_ref());
        self.tx.send(Message::notify_error(msg)).ok();
    }

    pub fn warn<S: AsRef<str>>(&self, msg: S) {
        tracing::warn!("{}", msg.as_ref());
        self.tx.send(Message::notify_warn(msg)).ok();
    }

    pub fn trace<S: AsRef<str>>(&self, msg: S) {
        tracing::trace!("{}", msg.as_ref());
        self.tx.send(Message::notify_trace(msg)).ok();
    }

    pub fn debug<S: AsRef<str>>(&self, msg: S) {
        tracing::debug!("{}", msg.as_ref());
        self.tx.send(Message::notify_debug(msg)).ok();
    }

    pub fn log_info<S: AsRef<str>>(&self, msg: S) {
        self.tx.send(Message::log_info(msg)).ok();
    }

    pub fn log_error<S: AsRef<str>>(&self, msg: S) {
        tracing::error!("{}", msg.as_ref());
        self.tx.send(Message::log_error(msg)).ok();
    }

    pub fn log_warn<S: AsRef<str>>(&self, msg: S) {
        tracing::warn!("{}", msg.as_ref());
        self.tx.send(Message::log_warn(msg)).ok();
    }

    pub fn log_trace<S: AsRef<str>>(&self, msg: S) {
        tracing::trace!("{}", msg.as_ref());
        self.tx.send(Message::log_trace(msg)).ok();
    }

    pub fn log_debug<S: AsRef<str>>(&self, msg: S) {
        tracing::debug!("{}", msg.as_ref());
        self.tx.send(Message::log_debug(msg)).ok();
    }

    pub fn update_statusline(&self, state: StatuslineState) {
        self.tx
            .send(Message::Execute(Task::UpdateStatusline(state)))
            .ok();
    }

    pub fn open_logger(&self) {
        self.tx.send(Message::Execute(Task::OpenLogger)).ok();
    }

    pub fn reload_lsp_server(&self) {
        self.tx.send(Message::Execute(Task::ReloadLspServer)).ok();
    }
}
