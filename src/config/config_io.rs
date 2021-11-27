use std::env::home_dir;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use directories::BaseDirs;

use crate::config::config::Config;

const CONFIG_FILE_PATH: &str = "rmapi";
const CONFIG_FILE_NAME: &str = "rmapi.conf";
pub const CONFIG_PATH: [&str; 2] = [CONFIG_FILE_PATH, CONFIG_FILE_NAME];

/// Resolves the config path relative
/// by the home directory.
pub fn resolve_config_path() -> Result<PathBuf, String> {
    let dirs = BaseDirs::new();

    if let Some(dir) = dirs {
        let home_path = dir.config_dir();
        let config_path = CONFIG_PATH;

        let path = Path::new(home_path).join(config_path.iter().collect::<PathBuf>());

        return Ok(path);
    }

    Err("Could not locate home path".into())
}

/// Writes a Config struct to a file.
pub fn write_config(_conf: &Config, _path: &Path) -> Result<(), String> {
    #[cfg(not(test))]
    {
        let mut file = File::create(_path).map_err(|e| e.to_string())?;

        if let Some(content) = _conf.create_config_content() {
            file.write_all(content.as_bytes())
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Reads the config file and returns a Config object
pub fn load_config_from_file(path: &str) -> Result<Config, String> {
    let file = fs::read_to_string(path).map_err(|e| e.to_string())?;

    let mut config = Config::new();
    config.load(file.as_str()).map_err(|e| e.to_string())?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::*;

    #[test]
    fn test_dir_ok() {
        let path: PathBuf = CONFIG_PATH.iter().collect();
        assert_eq!(path, OsStr::new("rmapi/rmapi.conf"))
    }
}
