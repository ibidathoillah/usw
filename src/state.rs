use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use crate::error::UswitchError;

const STATE_DIR: &str = "/var/lib/usw";
const STATE_FILE: &str = "/var/lib/usw/state.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub version: u8,
    pub users: HashMap<String, UserState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserState {
    pub created_at: DateTime<Utc>,
    pub home: PathBuf,
    pub projects: Vec<String>,
    pub status: RuntimeStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeStatus {
    Running,
    Stopped,
    Unknown,
}

impl State {
    pub fn load() -> Result<Self, UswitchError> {
        let path = Path::new(STATE_FILE);
        if !path.exists() {
            return Ok(State {
                version: 1,
                users: HashMap::new(),
            });
        }

        let mut file = File::open(path)
            .map_err(|e| UswitchError::State(format!("failed to open state: {e}")))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| UswitchError::State(format!("failed to read state: {e}")))?;

        serde_json::from_str(&contents)
            .map_err(|e| UswitchError::State(format!("failed to parse state: {e}")))
    }

    pub fn save(&self) -> Result<(), UswitchError> {
        let dir = Path::new(STATE_DIR);
        fs::create_dir_all(dir)
            .map_err(|e| UswitchError::State(format!("failed to create state dir: {e}")))?;

        let path = Path::new(STATE_FILE);
        let tmp_path = path.with_extension("tmp");

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| UswitchError::State(format!("failed to serialize state: {e}")))?;

        {
            let mut file = File::create(&tmp_path)
                .map_err(|e| UswitchError::State(format!("failed to write temp state: {e}")))?;
            file.write_all(json.as_bytes())
                .map_err(|e| UswitchError::State(format!("failed to write state data: {e}")))?;
            file.flush()
                .map_err(|e| UswitchError::State(format!("failed to flush state: {e}")))?;
        }

        fs::rename(&tmp_path, path)
            .map_err(|e| UswitchError::State(format!("failed to rename state file: {e}")))?;

        use std::os::unix::fs::PermissionsExt;
        let perms = PermissionsExt::from_mode(0o600);
        fs::set_permissions(path, perms)
            .map_err(|e| UswitchError::State(format!("failed to set state permissions: {e}")))?;

        Ok(())
    }

    pub fn with_lock<F, T>(f: F) -> Result<T, UswitchError>
    where
        F: FnOnce(&mut State) -> Result<T, UswitchError>,
    {
        let lock_path = Path::new(STATE_DIR).join("state.lock");
        let lock_file = File::create(&lock_path).map_err(|e| {
            UswitchError::State(format!("failed to create lock file: {e}"))
        })?;
        fs2::FileExt::lock_exclusive(&lock_file).map_err(|e| {
            UswitchError::State(format!("failed to acquire state lock: {e}"))
        })?;

        let result = (|| {
            let mut state = State::load()?;
            let result = f(&mut state)?;
            state.save()?;
            Ok(result)
        })();

        let _ = fs2::FileExt::unlock(&lock_file);
        result
    }

    pub fn get_user(&self, username: &str) -> Result<&UserState, UswitchError> {
        self.users
            .get(username)
            .ok_or_else(|| UswitchError::UserNotFound(username.to_string()))
    }

    pub fn get_user_mut(&mut self, username: &str) -> Result<&mut UserState, UswitchError> {
        self.users
            .get_mut(username)
            .ok_or_else(|| UswitchError::UserNotFound(username.to_string()))
    }

    pub fn ensure_user_not_exists(&self, username: &str) -> Result<(), UswitchError> {
        if self.users.contains_key(username) {
            return Err(UswitchError::UserExists(username.to_string()));
        }
        Ok(())
    }
}
