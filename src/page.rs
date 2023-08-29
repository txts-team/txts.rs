use crate::{
    error::{parse_error, Error},
    TxtsClient,
};

use markup5ever::interface::tree_builder::TreeSink;
use scraper::{Html, Selector};
use uuid::Uuid;

#[derive(Debug)]
pub struct Page {
    pub username: String,
    pub verified: bool,
    pub html_content: String,
}

impl Page {
    pub(crate) fn parse(html: &str, username: String) -> Result<Self, Error> {
        let mut html = Html::parse_document(html);

        parse_error(&html)?;

        let verified = html
            .select(&Selector::parse(".verified-icon").unwrap())
            .next()
            .is_some();

        // removes extra elements from the primary container that we don't need (header, edit link, success msgs)
        html.remove_from_parent(
            &html
                .select(&Selector::parse(".primary-container>header").unwrap())
                .next()
                .ok_or(Error::HtmlParse)?
                .id(),
        );

        let mut div_ids = Vec::new();
        for div in html.select(&Selector::parse(".primary-container>div[class]").unwrap()) {
            div_ids.push(div.id());
        }

        for id in div_ids {
            html.remove_from_parent(&id);
        }

        let content = html
            .select(&Selector::parse(".primary-container").unwrap())
            .next()
            .ok_or(Error::HtmlParse)?
            .inner_html();

        Ok(Self {
            username,
            verified,
            html_content: content.trim().to_string(),
        })
    }

    pub async fn get_markdown(&self, client: &TxtsClient) -> Result<String, Error> {
        client.get_markdown(&self.username).await
    }

    pub async fn edit(
        &mut self,
        client: &TxtsClient,
        secret: &Uuid,
        content: &str,
    ) -> Result<(), Error> {
        let page = client.edit(&self.username, secret, content).await?;
        self.html_content = page.html_content;
        self.verified = page.verified;
        Ok(())
    }
}
