use core;
use git2;
use github_gql;
use regex;
use serde_json;
use std::error::Error;
use std::fmt;
use std::io;
use std::time::SystemTimeError;

#[derive(Debug)]
pub enum AppError {
  IO(io::Error),
  UserError(String),
  RuntimeError(String),
  BadJson(serde_json::Error),
  InternalError(&'static str),
  ClockError(SystemTimeError),
  GitError(git2::Error),
  Regex(regex::Error),
  GithubApiError(github_gql::errors::Error),
}

macro_rules! app_error_from {
  ($error: ty, $app_error: ident) => {
    impl From<$error> for AppError {
      fn from(err: $error) -> AppError {
        AppError::$app_error(err)
      }
    }
  };
}

impl AppError {
  pub fn require<T>(option: Option<T>, app_error: AppError) -> Result<T, AppError> {
    if let Some(value) = option {
      Result::Ok(value)
    } else {
      Result::Err(app_error)
    }
  }
}

impl fmt::Display for AppError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      AppError::IO(ref err) => write!(f, "IO error: {}", err),
      AppError::UserError(ref str) => write!(f, "User error: {}", str),
      AppError::RuntimeError(ref str) => write!(f, "Runtime error: {}", str),
      AppError::BadJson(ref err) => write!(f, "JSON error: {}", err),
      AppError::InternalError(str) => write!(f, "Internal error: {}", str),
      AppError::ClockError(ref err) => write!(f, "System clock error: {}", err),
      AppError::GitError(ref err) => write!(f, "Git error: {}", err),
      AppError::Regex(ref err) => write!(f, "Regex error: {}", err),
      AppError::GithubApiError(ref err) => write!(f, "GitHub API error: {}", err),
    }
  }
}

impl Error for AppError {
  fn description(&self) -> &str {
    match *self {
      AppError::IO(ref err) => err.description(),
      AppError::UserError(ref str) | AppError::RuntimeError(ref str) => str.as_ref(),
      AppError::BadJson(ref err) => err.description(),
      AppError::InternalError(str) => str,
      AppError::ClockError(ref err) => err.description(),
      AppError::GitError(ref err) => err.description(),
      AppError::Regex(ref err) => err.description(),
      AppError::GithubApiError(ref err) => err.description(),
    }
  }

  fn cause(&self) -> Option<&Error> {
    match *self {
      AppError::IO(ref err) => Some(err),
      AppError::UserError(_) | AppError::RuntimeError(_) | AppError::InternalError(_) => None,
      AppError::BadJson(ref err) => Some(err),
      AppError::ClockError(ref err) => Some(err),
      AppError::GitError(ref err) => Some(err),
      AppError::Regex(ref err) => Some(err),
      AppError::GithubApiError(ref err) => Some(err),
    }
  }
}

impl From<core::num::ParseIntError> for AppError {
  fn from(err: core::num::ParseIntError) -> AppError {
    AppError::UserError(format!("Type error: {}", err.description()))
  }
}

app_error_from!(github_gql::errors::Error, GithubApiError);
app_error_from!(git2::Error, GitError);
app_error_from!(io::Error, IO);
app_error_from!(serde_json::Error, BadJson);
app_error_from!(regex::Error, Regex);
