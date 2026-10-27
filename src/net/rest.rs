use crate::error::{NodeLinkError, NodeResult};
use crate::model::payload::{UpdatePlayerPayload, UpdateSessionPayload};
use crate::model::track::LoadResult;
use reqwest::{Client as RawHttp, StatusCode};
use std::sync::Arc;
use tokio::sync::RwLock;

pub(crate) struct RestClient {
    base_url: String,
    http: RawHttp,
    pub(crate) session_id: Arc<RwLock<Option<String>>>,
}

impl RestClient {
    pub fn new(base_url: String, http: RawHttp, session_id: Arc<RwLock<Option<String>>>) -> Self {
        Self {
            base_url,
            http,
            session_id,
        }
    }

    async fn get_session_id(&self) -> NodeResult<String> {
        self.session_id
            .read()
            .await
            .as_ref()
            .cloned()
            .ok_or(NodeLinkError::SessionNotReady)
    }

    async fn ensure_success(&self, res: reqwest::Response) -> NodeResult<reqwest::Response> {
        if res.status().is_success() {
            Ok(res)
        } else {
            let status = res.status().as_u16();
            let message = res.text().await.unwrap_or_default();
            Err(NodeLinkError::Api { status, message })
        }
    }

    pub async fn load_tracks(&self, identifier: &str) -> NodeResult<LoadResult> {
        let url = format!("{}/v4/loadtracks", self.base_url);
        let res = self
            .http
            .get(url)
            .query(&[("identifier", identifier)])
            .send()
            .await?;

        let res = self.ensure_success(res).await?;
        Ok(res.json::<LoadResult>().await?)
    }

    pub async fn update_player(
        &self,
        guild_id: u64,
        payload: &UpdatePlayerPayload,
        no_replace: bool,
    ) -> NodeResult<()> {
        let session = self.get_session_id().await?;
        let url = format!(
            "{}/v4/sessions/{}/players/{}?noReplace={}",
            self.base_url, session, guild_id, no_replace
        );

        let res = self.http.patch(url).json(payload).send().await?;

        self.ensure_success(res).await?;
        Ok(())
    }

    pub async fn destroy_player(&self, guild_id: u64) -> NodeResult<()> {
        let session = self.get_session_id().await?;
        let url = format!(
            "{}/v4/sessions/{}/players/{}",
            self.base_url, session, guild_id
        );

        let res = self.http.delete(url).send().await?;

        if res.status() == StatusCode::NOT_FOUND {
            return Ok(());
        }

        self.ensure_success(res).await?;
        Ok(())
    }

    pub async fn update_session(&self, payload: &UpdateSessionPayload) -> NodeResult<()> {
        let session = self.get_session_id().await?;
        let url = format!("{}/v4/sessions/{}", self.base_url, session);

        let res = self.http.patch(url).json(payload).send().await?;
        self.ensure_success(res).await?;
        Ok(())
    }
}
