use crate::client::Session;
use once_cell::sync::Lazy;
use std::sync::Mutex;

/// Global mutable store for the current session (authenticated client and user).
static SESSION: Lazy<Mutex<Option<Session>>> = Lazy::new(|| Mutex::new(None));

/// Set or replace the global session.
pub fn set_session(session: Session) -> anyhow::Result<()> {
    let mut guard = SESSION.lock().unwrap();
    *guard = Some(session);
    Ok(())
}

/// Retrieve a clone of the current session.
/// Panics if called before the session is set (i.e., before login/registration).
pub fn get_session() -> Session {
    let guard = SESSION.lock().unwrap();
    guard.clone().expect("Session not initialized")
}
