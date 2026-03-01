//! Thin github api wrapper for the 3 calls I need to do

use reqwest::{
    Client, RequestBuilder,
    header::{AUTHORIZATION, HeaderValue, USER_AGENT},
};
use serde::Deserialize;

#[derive(Deserialize)]
struct Commit {
    sha: String,
}

#[derive(Deserialize)]
struct BranchResponse {
    commit: Commit,
}

pub struct GithubAPI {
    client: Client,
    repo: String,
    token: Option<String>,
    username: String,
    pub sha: String,
}

impl GithubAPI {
    const API_VERSION: &'static str = "2022-11-28";
    pub async fn new(
        repo: String,
        branch: String,
        token: Option<String>,
    ) -> Result<Self, reqwest::Error> {
        let client = Client::new();

        let username = repo
            .split_once('/')
            .map(|(username, _)| username.to_owned())
            .unwrap_or_default();

        let mut req = client
            .get(format!(
                "https://api.github.com/repos/{repo}/branches/{branch}"
            ))
            .header("X-GitHub-Api-Version", GithubAPI::API_VERSION)
            .header(USER_AGENT, &username);

        if let Some(token) = token.as_ref() {
            if let Ok(mut val) = HeaderValue::from_str(&format!("Bearer {token}")) {
                val.set_sensitive(true);
                req = req.header(AUTHORIZATION, val);
            }
        }

        let res = req.send().await?.json::<BranchResponse>().await?;

        Ok(Self {
            client,
            username,
            token,
            repo,
            sha: res.commit.sha,
        })
    }

    fn get(&self, url: String) -> RequestBuilder {
        let req = self
            .client
            .get(url)
            .header(USER_AGENT, &self.username)
            .header("X-GitHub-Api-Version", GithubAPI::API_VERSION);
        if let Some(token) = self.token.as_ref() {
            if let Ok(mut val) = HeaderValue::from_str(&format!("Bearer {token}")) {
                val.set_sensitive(true);
                req.header(AUTHORIZATION, val)
            } else {
                req
            }
        } else {
            req
        }
    }

    pub async fn get_tarball(&self) -> Result<reqwest::Response, reqwest::Error> {
        self.get(format!(
            "https://api.github.com/repos/{}/tarball/{}",
            self.repo, self.sha
        ))
        .send()
        .await
    }
}
