use thiserror::Error;

#[derive(Error, Debug)]
pub enum UswitchError {
    #[error("must run as root (current euid: {0})")]
    NotRoot(u32),

    #[error("user '{0}' already exists")]
    UserExists(String),

    #[error("user '{0}' not found")]
    UserNotFound(String),

    #[error("runtime '{0}' is already running")]
    AlreadyRunning(String),

    #[error("runtime '{0}' is not running")]
    NotRunning(String),

    #[error("runtime '{0}' not found")]
    RuntimeNotFound(String),

    #[error("project '{0}' not found")]
    ProjectNotFound(String),

    #[error("project '{0}' is already attached to user '{1}'")]
    ProjectAlreadyAttached(String, String),

    #[error("project '{0}' is not attached to user '{1}'")]
    ProjectNotAttached(String, String),

    #[error("state file error: {0}")]
    State(String),

    #[error("systemd error: {0}")]
    Systemd(String),

    #[error("system command '{0}' failed: {1}")]
    CommandFailed(String, String),

    #[error("invalid username '{0}': must match [a-z][a-z0-9_-]{{1,30}}")]
    InvalidUsername(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
