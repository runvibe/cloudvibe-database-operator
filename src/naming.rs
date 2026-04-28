use regex::Regex;
use std::sync::LazyLock;
use thiserror::Error;

static IDENTIFIER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]{0,62}$").expect("valid regex"));

#[derive(Debug, Error)]
pub enum NamingError {
    #[error("invalid identifier `{0}`")]
    InvalidIdentifier(String),
    #[error("secret name `{name}` must stay under prefix `{prefix}`")]
    SecretOutsidePrefix { name: String, prefix: String },
}

pub fn validate_identifier(value: &str) -> Result<(), NamingError> {
    if IDENTIFIER.is_match(value) {
        Ok(())
    } else {
        Err(NamingError::InvalidIdentifier(value.to_string()))
    }
}

pub fn quote_identifier(value: &str) -> Result<String, NamingError> {
    validate_identifier(value)?;
    Ok(format!(r#""{value}""#))
}

pub fn default_secret_name(
    prefix: &str,
    namespace: &str,
    database: &str,
    username: &str,
) -> Result<String, NamingError> {
    validate_identifier(database)?;
    validate_identifier(username)?;
    let name = format!(
        "{}/{}/{}/{}",
        prefix.trim_end_matches('/'),
        namespace,
        database,
        username
    );
    ensure_secret_prefix(prefix, &name)?;
    Ok(name)
}

pub fn ensure_secret_prefix(prefix: &str, name: &str) -> Result<(), NamingError> {
    let normalized = prefix.trim_end_matches('/');
    if name == normalized || name.starts_with(&format!("{normalized}/")) {
        Ok(())
    } else {
        Err(NamingError::SecretOutsidePrefix {
            name: name.to_string(),
            prefix: prefix.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_identifiers() {
        assert!(validate_identifier("orders_api").is_ok());
        assert!(validate_identifier("1orders").is_err());
    }

    #[test]
    fn builds_secret_name_under_prefix() {
        let name = default_secret_name("rds/prod", "orders", "orders", "orders_api").unwrap();
        assert_eq!(name, "rds/prod/orders/orders/orders_api");
    }

    #[test]
    fn quotes_identifiers() {
        assert_eq!(quote_identifier("orders_api").unwrap(), r#""orders_api""#);
        assert!(quote_identifier("orders-api").is_err());
    }
}
