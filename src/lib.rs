mod client;
mod page;
mod error;

pub use client::TxtsClient;
pub use page::Page;
pub use error::Error;

#[cfg(test)]
mod tests {
    use url::Url;
    use uuid::Uuid;

    use super::*;
    const HOST: &'static str = "http://localhost:8001/";

    #[tokio::test]
    async fn test_page_verified() -> Result<(), Error> {
        let client = TxtsClient::new(Url::parse(HOST).unwrap())?;
        let page = client.get("verifiedtest").await?;
        assert_eq!(page.verified, true);
        Ok(())
    }

    #[tokio::test]
    async fn test_page_unverified() -> Result<(), Error> {
        let client = TxtsClient::new(Url::parse(HOST).unwrap())?;
        let page = client.get("test").await?;
        assert_eq!(page.verified, false);
        Ok(())
    }

    #[tokio::test]
    async fn test_markdown_get() -> Result<(), Error> {
        let client = TxtsClient::new(Url::parse(HOST).unwrap())?;
        client.get_markdown("test").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_create_ok() -> Result<(), Error> {
        let client = TxtsClient::new(Url::parse(HOST).unwrap())?;
        client.create("test", "content").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_create_err() -> Result<(), Error> {
        let client = TxtsClient::new(Url::parse(HOST).unwrap())?;
        let resp = client.create("test", "content").await;
        match resp {
            Ok(_) => panic!(),
            Err(e) => match e {
                Error::TxtsError { .. } => Ok(()),
                _ => panic!(),
            },
        }
    }

    #[tokio::test]
    async fn test_edit_ok() -> Result<(), Error> {
        let client = TxtsClient::new(Url::parse(HOST).unwrap())?;
        let secret = Uuid::parse_str("35934ccc-d791-4c05-befd-5e92e91c9339").unwrap();
        client.edit("test", &secret, "epic content").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_edit_err() -> Result<(), Error> {
        let client = TxtsClient::new(Url::parse(HOST).unwrap())?;
        let wrong_secret = Uuid::parse_str("ab6f8341-080f-4ef3-82b7-ec6a3c37a0f2").unwrap();
        let resp = client.edit("test", &wrong_secret, "epic content").await;
        match resp {
            Ok(_) => panic!(),
            Err(e) => match e {
                Error::TxtsError { .. } => Ok(()),
                _ => panic!(),
            },
        }
    }
}
