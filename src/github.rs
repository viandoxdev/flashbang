//! Thin github api wrapper for the 3 calls I need to do

use reqwest::{header::ACCEPT, Client};
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
    pub sha: String,
}

impl GithubAPI {
    pub async fn new(repo: String, branch: String) -> Result<Self, reqwest::Error> {
        let client = Client::new();

        let res = client
            .get(format!(
                "https://api.github.com/repos/{repo}/branches/{branch}"
            ))
            .send()
            .await?
            .json::<BranchResponse>()
            .await?;

        Ok(Self {
            client,
            repo,
            sha: res.commit.sha,
        })
    }

    pub async fn get_items(&self) -> Result<Vec<TreeItem>, reqwest::Error> {
        Ok(self
            .client
            .get(format!(
                "https://api.github.com/repos/{}/git/trees/{}?recursive=1",
                self.repo, self.sha
            ))
            .send()
            .await?
            .json::<TreeResponse>()
            .await?
            .tree)
    }

    pub async fn get_blob(&self, sha: impl AsRef<str>) -> Result<String, reqwest::Error> {
        let sha = sha.as_ref();
        self.client
            .get(format!(
                "https://api.github.com/repos/{}/git/blobs/{sha}",
                self.repo
            ))
            .header(ACCEPT, "application/vnd.github.raw+json")
            .send()
            .await?
            .text()
            .await
    }
}
