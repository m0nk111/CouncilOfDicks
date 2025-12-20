use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constitution {
    pub content: String,
    pub last_updated: u64,
    pub signature: Option<String>, // For future admin verification
}

impl Default for Constitution {
    fn default() -> Self {
        Self {
            content: "# The Council Constitution\n\nLoading...".to_string(),
            last_updated: 0,
            signature: None,
        }
    }
}

pub struct ConstitutionManager {
    file_path: PathBuf,
    state: Mutex<Constitution>,
}

impl Default for ConstitutionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstitutionManager {
    pub fn new() -> Self {
        let mut path = PathBuf::from("config/constitution.md");
        if !path.exists() {
            // Try parent directory
            let parent_path = PathBuf::from("../config/constitution.md");
            if parent_path.exists() {
                path = parent_path;
            }
        }

        let state = if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => Constitution {
                    content,
                    last_updated: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    signature: None,
                },
                Err(_) => Constitution::default(),
            }
        } else {
            Constitution::default()
        };

        Self {
            file_path: path,
            state: Mutex::new(state),
        }
    }

    pub fn get_content(&self) -> String {
        self.state.lock().unwrap().content.clone()
    }

    pub fn update_content(&self, new_content: String) -> Result<(), String> {
        // In the future, this will require a signature check
        let mut state = self.state.lock().unwrap();
        state.content = new_content.clone();
        state.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Save to disk
        fs::write(&self.file_path, &state.content).map_err(|e| e.to_string())?;
        
        Ok(())
    }
}
