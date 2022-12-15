use crate::results::QueryResults;
use crate::PrestinoError;
use reqwest::header::HeaderMap;
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;

#[derive(Debug)]
pub(crate) struct ClientConnection {
    pub(crate) headers: HeaderMap,
    pub(crate) http_client: Client,
}

impl ClientConnection {
    /// A convenience function to retrieve all the rows for the statement into a single Vec.
    pub async fn post_statement<T: DeserializeOwned>(
        &mut self,
        base_url: &str,
        statement: String,
    ) -> Result<QueryResults<T>, PrestinoError> {
        let response = self
            .http_client
            .post(format!("{}/v1/statement", base_url))
            .headers(self.headers.clone())
            .body(statement)
            .send()
            .await?;

        self.parse_response(response).await
    }

    pub async fn get_next_results<T: DeserializeOwned>(
        &mut self,
        next_uri: &str,
    ) -> Result<QueryResults<T>, PrestinoError> {
        println!("Getting next results: {}", next_uri);
        let response = self
            .http_client
            .get(next_uri)
            .headers(self.headers.clone())
            .send()
            .await?;
        self.parse_response(response).await
    }

    async fn parse_response<T: DeserializeOwned>(
        &mut self,
        response: Response,
    ) -> Result<QueryResults<T>, PrestinoError> {
        let status = response.status();
        if status != reqwest::StatusCode::OK {
            let message = response.text().await?;
            return Err(PrestinoError::from_status_code(status.as_u16(), message));
        }
        // TODO: Parse response headers to modify client's headers
        // TODO: Make better error messages on json deser.  In particular, if there's a type error,
        // can we print out the row that causes the error?
        Ok(response.json().await?)
    }

    pub async fn cancel(&mut self, next_uri: &str) -> Result<(), PrestinoError> {
        let response = self
            .http_client
            .delete(next_uri)
            .headers(self.headers.clone())
            .send()
            .await?;

        self.parse_response::<()>(response).await.map(|_| ())
    }
}
