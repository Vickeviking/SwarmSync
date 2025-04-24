use crate::client::Session;
use once_cell::sync::OnceCell;
use std::sync::OnceLock;

static SESSION: OnceLock<Session> = OnceLock::new();

pub fn set_session(session: Session) -> anyhow::Result<()> {
    SESSION
        .set(session)
        .map_err(|_| anyhow::anyhow!("Session already set"))
}

pub fn get_session() -> &'static Session {
    SESSION.get().expect("Session not initialized")
}
