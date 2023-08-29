use crate::{
    error::{check_username, parse_error, Error},
    Page,
};
use reqwest::{Client, ClientBuilder, StatusCode};
use scraper::{Html, Selector};
use url::Url;
use uuid::Uuid;

const USER_AGENT: &str = concat!("txts.rs/", env!("CARGO_PKG_VERSION"), " (txts-team/txts.rs)");

#[derive(Debug)]
pub struct TxtsClient {
    client: Client,
    host: Url,
}

impl TxtsClient {
    pub fn new(host: Url) -> Result<Self, reqwest::Error> {
        Ok(Self {
            client: ClientBuilder::new()
                .user_agent(USER_AGENT)
                .cookie_store(true)
                .brotli(true)
                .build()?,
            host,
        })
    }

    pub async fn get(&self, username: &str) -> Result<Page, Error> {
        check_username(username)?;

        let resp = self
            .client
            .get(self.host.join(&format!("@{username}"))?)
            .send()
            .await?;

        if let StatusCode::NOT_FOUND = resp.status() {
            return Err(Error::NotFound);
        }

        let resp = resp.error_for_status()?;

        Page::parse(&resp.text().await?, username.to_string())
    }

    pub async fn get_markdown(&self, username: &str) -> Result<String, Error> {
        check_username(username)?;

        let resp = self
            .client
            .get(self.host.join(&format!("@{username}/edit"))?)
            .send()
            .await?;

        if let StatusCode::NOT_FOUND = resp.status() {
            return Err(Error::NotFound);
        }

        let resp = resp.error_for_status()?;

        let html = Html::parse_document(&resp.text().await?);
        parse_error(&html)?;

        let markdown = html
            .select(&Selector::parse("#content").unwrap())
            .next()
            .ok_or(Error::HtmlParse)?
            .text()
            .next()
            .ok_or(Error::HtmlParse)?;

        Ok(markdown.trim().to_string())
    }

    async fn get_vrf_token(&self, path: &str) -> Result<String, Error> {
        let resp = self.client.get(self.host.join(path)?).send().await?;

        let html = Html::parse_document(&resp.text().await?);

        parse_error(&html)?;

        let vrf_token = html
            .select(&Selector::parse("input[name=__RequestVerificationToken]").unwrap())
            .next()
            .ok_or(Error::HtmlParse)?
            .value()
            .attr("value")
            .ok_or(Error::HtmlParse)?;

        Ok(vrf_token.to_string())
    }

    pub async fn create(&self, username: &str, content: &str) -> Result<(Page, Uuid), Error> {
        check_username(username)?;

        let vrf_token = self.get_vrf_token("").await?;

        let resp = self
            .client
            .post(self.host.as_ref())
            .form(&[
                ("username", username),
                ("content", content),
                ("__RequestVerificationToken", &vrf_token),
            ])
            .send()
            .await?;

        let resp = resp.error_for_status()?;
        // checks for redirect
        if resp.url() != &self.host {
            let secret = resp.url().query_pairs().find(|q| q.0 == "secret");
            let secret = secret.ok_or(Error::SecretNotPresent)?.1;
            let secret = Uuid::parse_str(&secret)?;

            let page = Page::parse(&resp.text().await?, username.to_string())?;

            Ok((page, secret))
        } else {
            parse_error(&Html::parse_document(&resp.text().await?))?;
            Err(Error::UnknownTxtsError)
        }
    }

    pub async fn edit(&self, username: &str, secret: &Uuid, content: &str) -> Result<Page, Error> {
        check_username(username)?;

        let path = format!("@{username}/edit");
        let vrf_token = self.get_vrf_token(&path).await?;

        let req_url = self.host.join(&path)?;
        let resp = self
            .client
            .post(req_url.as_ref())
            .form(&[
                ("content", content),
                ("secret", &secret.to_string()),
                ("__RequestVerificationToken", &vrf_token),
            ])
            .send()
            .await?;

        if let StatusCode::NOT_FOUND = resp.status() {
            return Err(Error::NotFound);
        }

        let resp = resp.error_for_status()?;
        // checks for redirect
        if resp.url() != &req_url {
            let page = Page::parse(&resp.text().await?, username.to_string())?;
            Ok(page)
        } else {
            parse_error(&Html::parse_document(&resp.text().await?))?;
            Err(Error::UnknownTxtsError)
        }
    }
}
