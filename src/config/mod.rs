use errors::AppError;
use serde_json;
use slog::Logger;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
  pub workspace: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
  pub name: String,
  pub git: String,
  pub after_clone: Option<String>,
  pub after_workon: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
  pub projects: HashMap<String, Project>,
  pub settings: Settings,
}

fn read_config<R>(reader: Result<R, AppError>) -> Result<Config, AppError>
  where R: Read
{
  reader.and_then(|r| serde_json::de::from_reader(r).map_err(|error| AppError::BadJson(error)))
}

pub fn config_path() -> Result<PathBuf, AppError> {
  let mut home: PathBuf = env::home_dir()
    .ok_or(AppError::UserError("$HOME not set".to_owned()))?;
  home.push(".fw.json");
  Ok(home)
}

fn determine_config() -> Result<File, AppError> {
  let config_file_path = config_path()?;
  let path = config_file_path.to_str()
                             .ok_or(AppError::UserError("$HOME is not valid utf8".to_owned()));
  path.and_then(|path| File::open(path).map_err(|err| AppError::IO(err)))
}

pub fn get_config() -> Result<Config, AppError> {
  let config_file = determine_config();
  let reader = config_file.map(|f| BufReader::new(f));
  read_config(reader)
}

pub fn add_entry(maybe_config: Result<Config, AppError>, name: &str, url: &str, logger: &Logger) -> Result<(), AppError> {
  let mut config: Config = maybe_config?;
  info!(logger, "Prepare new project entry"; "name" => name, "url" => url);
  if name.starts_with("http") || name.starts_with("git@") {
    Err(AppError::UserError(format!("{} looks like a repo URL and not like a project name, please fix",
                                    name)))
  } else if config.projects.contains_key(name) {
    Err(AppError::UserError(format!("Project key {} already exists, not gonna overwrite it for you",
                                    name)))
  } else {
    config.projects
          .insert(name.to_owned(),
                  Project {
                    git: url.to_owned(),
                    name: name.to_owned(),
                    after_clone: None,
                    after_workon: None,
                  });
    info!(logger, "Updated config"; "config" => format!("{:?}", config));
    write_config(config, logger)
  }
}

pub fn write_config(config: Config, logger: &Logger) -> Result<(), AppError> {
  let config_path = config_path()?;
  info!(logger, "Writing config"; "path" => format!("{:?}", config_path));
  let mut buffer = File::create(config_path)?;
  serde_json::ser::to_writer_pretty(&mut buffer, &config).map_err(|e| AppError::BadJson(e))
}