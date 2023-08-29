use regex::Regex;
use scraper::Html;
use scraper::Selector;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("request error")]
    Request(#[from] reqwest::Error),
    #[error("page not found")]
    NotFound,
    #[error("invalid username")]
    InvalidUsername,
    #[error("error from txts (error {error:?}, message {message:?})")]
    TxtsError { error: String, message: String },
    #[error("unknown error from txts")]
    UnknownTxtsError,
    #[error("error while parsing html")]
    HtmlParse,
    #[error("error while parsing url")]
    UrlParse(#[from] url::ParseError),
    #[error("secret not present in redirect url")]
    SecretNotPresent,
    #[error("error while parsing secret uuid")]
    UuidParse(#[from] uuid::Error),
}

pub(crate) fn parse_error(html: &Html) -> Result<(), Error> {
    let error = html
        .select(&Selector::parse(".error-message").unwrap())
        .next();

    if error.is_some() {
        let err_title = html
            .select(&Selector::parse(".error-message :nth-child(1)").unwrap())
            .next()
            .ok_or(Error::HtmlParse)?
            .text()
            .find(|s| !s.trim().is_empty())
            .ok_or(Error::HtmlParse)?;
        let err_msg = html
            .select(&Selector::parse(".error-message :nth-child(2)").unwrap())
            .next()
            .ok_or(Error::HtmlParse)?
            .text()
            .find(|s| !s.trim().is_empty())
            .ok_or(Error::HtmlParse)?;

        return Err(Error::TxtsError {
            error: err_title.trim().to_string(),
            message: err_msg.trim().to_string(),
        });
    }

    Ok(())
}

pub(crate) fn check_username(username: &str) -> Result<(), Error> {
    let re = Regex::new(r"^[A-Za-z0-9_.]{3,16}$").unwrap();
    match re.is_match(username) {
        true => Ok(()),
        false => Err(Error::InvalidUsername),
    }
}
