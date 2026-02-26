//! Thin github api wrapper for the 3 calls I need to do

use reqwest::{
    blocking::{Client, RequestBuilder},
    header::{ACCEPT, AUTHORIZATION, HeaderValue, USER_AGENT},
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

#[derive(Deserialize)]
pub struct TreeItem {
    pub path: String,
    pub sha: String,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Deserialize)]
struct TreeResponse {
    tree: Vec<TreeItem>,
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
    pub fn new(
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
            let mut val = HeaderValue::from_str(&format!("Bearer {token}")).unwrap();
            val.set_sensitive(true);
            req = req.header(AUTHORIZATION, val);
        }

        let res = req.send()?.json::<BranchResponse>()?;

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
            let mut val = HeaderValue::from_str(&format!("Bearer {token}")).unwrap();
            val.set_sensitive(true);
            req.header(AUTHORIZATION, val)
        } else {
            req
        }
    }

    pub fn get_items(&self) -> Result<Vec<TreeItem>, reqwest::Error> {
        Ok(self
            .get(format!(
                "https://api.github.com/repos/{}/git/trees/{}?recursive=1",
                self.repo, self.sha
            ))
            .send()?
            .json::<TreeResponse>()?
            .tree)
    }

    pub fn get_blob(&self, sha: impl AsRef<str>) -> Result<String, reqwest::Error> {
        let sha = sha.as_ref();
        self.get(format!(
            "https://api.github.com/repos/{}/git/blobs/{sha}",
            self.repo
        ))
        .header(ACCEPT, "application/vnd.github.raw+json")
        .send()?
        .text()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorization_header_is_sensitive() {
        let client = Client::new();
        let api = GithubAPI {
            client,
            repo: "owner/repo".to_string(),
            token: Some("secret_token".to_string()),
            username: "owner".to_string(),
            sha: "dummy_sha".to_string(),
        };

        let req = api
            .get("https://example.com".to_string())
            .build()
            .expect("Failed to build request");

        if let Some(auth_header) = req.headers().get(AUTHORIZATION) {
            assert!(
                auth_header.is_sensitive(),
                "Authorization header should be marked sensitive"
            );
        } else {
            panic!("Authorization header missing");
        }
    }
}
