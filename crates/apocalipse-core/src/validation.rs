use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadExpectation {
    Any,
    Zip,
    Media,
    Binary,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PayloadError {
    #[error("the server returned an HTML page instead of the requested file")]
    HtmlInsteadOfFile,
    #[error("the response does not have a valid ZIP signature")]
    InvalidZipSignature,
    #[error("the server returned JSON instead of downloadable content")]
    JsonInsteadOfFile,
}

pub fn validate_payload(
    expectation: PayloadExpectation,
    content_type: Option<&str>,
    prefix: &[u8],
) -> Result<(), PayloadError> {
    let trimmed = prefix
        .iter()
        .copied()
        .skip_while(u8::is_ascii_whitespace)
        .take(16)
        .collect::<Vec<_>>();
    let lowered_type = content_type.unwrap_or_default().to_ascii_lowercase();
    let html = lowered_type.contains("text/html")
        || trimmed.starts_with(b"<!DOCTYPE")
        || trimmed.starts_with(b"<html");
    if expectation != PayloadExpectation::Any && html {
        return Err(PayloadError::HtmlInsteadOfFile);
    }
    if expectation != PayloadExpectation::Any
        && (lowered_type.contains("application/json") || trimmed.starts_with(b"{"))
    {
        return Err(PayloadError::JsonInsteadOfFile);
    }
    if expectation == PayloadExpectation::Zip
        && !prefix.starts_with(b"PK\x03\x04")
        && !prefix.starts_with(b"PK\x05\x06")
        && !prefix.starts_with(b"PK\x07\x08")
    {
        return Err(PayloadError::InvalidZipSignature);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_login_or_error_page_disguised_as_zip() {
        assert_eq!(
            validate_payload(
                PayloadExpectation::Zip,
                Some("text/html"),
                b"<!DOCTYPE html>"
            ),
            Err(PayloadError::HtmlInsteadOfFile)
        );
    }

    #[test]
    fn accepts_zip_magic_even_without_content_type() {
        assert_eq!(
            validate_payload(PayloadExpectation::Zip, None, b"PK\x03\x04rest"),
            Ok(())
        );
    }
}
