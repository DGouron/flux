use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecretsError {
    #[error("credential non trouvé pour {provider}")]
    NotFound { provider: String },

    #[error("erreur de lecture du fichier secrets: {source}")]
    Read {
        #[from]
        source: std::io::Error,
    },

    #[error("erreur de parsing secrets.toml: {source}")]
    Parse {
        #[from]
        source: toml::de::Error,
    },
}

#[derive(Debug, Clone)]
pub struct ProviderCredentials {
    pub token: String,
    pub user_id: u64,
}

#[derive(Debug, serde::Deserialize)]
struct SecretsFile {
    gitlab: Option<ProviderSecrets>,
    github: Option<ProviderSecrets>,
}

#[derive(Debug, serde::Deserialize)]
struct ProviderSecrets {
    token: String,
    user_id: u64,
}

pub fn resolve_gitlab_credentials() -> Result<ProviderCredentials, SecretsError> {
    resolve_credentials("gitlab", "FLUX_GITLAB_TOKEN", "FLUX_GITLAB_USER_ID")
}

pub fn resolve_github_credentials() -> Result<ProviderCredentials, SecretsError> {
    resolve_credentials("github", "FLUX_GITHUB_TOKEN", "FLUX_GITHUB_USER_ID")
}

fn resolve_credentials(
    provider: &str,
    token_env: &str,
    user_id_env: &str,
) -> Result<ProviderCredentials, SecretsError> {
    if let (Ok(token), Ok(user_id_str)) = (std::env::var(token_env), std::env::var(user_id_env)) {
        if let Ok(user_id) = user_id_str.parse::<u64>() {
            return Ok(ProviderCredentials { token, user_id });
        }
    }

    load_from_secrets_file(provider)
}

fn load_from_secrets_file(provider: &str) -> Result<ProviderCredentials, SecretsError> {
    let path = secrets_path();

    if !path.exists() {
        return Err(SecretsError::NotFound {
            provider: provider.to_string(),
        });
    }

    let content = std::fs::read_to_string(&path)?;
    let secrets: SecretsFile = toml::from_str(&content)?;

    let provider_secrets = match provider {
        "gitlab" => secrets.gitlab,
        "github" => secrets.github,
        _ => None,
    };

    provider_secrets
        .map(|secrets| ProviderCredentials {
            token: secrets.token,
            user_id: secrets.user_id,
        })
        .ok_or_else(|| SecretsError::NotFound {
            provider: provider.to_string(),
        })
}

fn secrets_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("flux")
        .join("secrets.toml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn resolve_from_environment() {
        env::set_var("FLUX_GITLAB_TOKEN", "test-token-123");
        env::set_var("FLUX_GITLAB_USER_ID", "42");

        let credentials = resolve_gitlab_credentials().unwrap();

        assert_eq!(credentials.token, "test-token-123");
        assert_eq!(credentials.user_id, 42);

        env::remove_var("FLUX_GITLAB_TOKEN");
        env::remove_var("FLUX_GITLAB_USER_ID");
    }

    #[test]
    fn missing_credentials_returns_error() {
        env::remove_var("FLUX_GITHUB_TOKEN");
        env::remove_var("FLUX_GITHUB_USER_ID");

        let result = resolve_github_credentials();

        assert!(result.is_err());
    }
}
