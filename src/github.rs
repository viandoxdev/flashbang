//! Thin github api wrapper for the 3 calls I need to do

use reqwest::{
    header::{ACCEPT, AUTHORIZATION, USER_AGENT},
    Client, IntoUrl, RequestBuilder, Url,
};
use serde::Deserialize;
use std::error::Error;

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
    pub async fn new(
        repo: String,
        branch: String,
        token: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        if !is_valid_github_repo(&repo) {
            return Err("Invalid GitHub repository format. Expected 'owner/repo'.".into());
        }
        if !is_valid_github_branch(&branch) {
            return Err("Invalid GitHub branch name.".into());
        }

        let client = Client::new();

        let (owner, repo_name) = repo.split_once('/').unwrap();
        let username = owner.to_owned();

        let mut url = Url::parse("https://api.github.com/repos/").unwrap();
        {
            let mut segments = url.path_segments_mut().unwrap();
            segments.push(owner);
            segments.push(repo_name);
            segments.push("branches");
            segments.push(&branch);
        }

        let mut req = client
            .get(url)
            .header("X-GitHub-Api-Version", GithubAPI::API_VERSION)
            .header(USER_AGENT, &username);

        if let Some(token) = token.as_ref() {
            req = req.header(AUTHORIZATION, format!("Bearer {token}"));
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

    fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        let req = self
            .client
            .get(url)
            .header(USER_AGENT, &self.username)
            .header("X-GitHub-Api-Version", GithubAPI::API_VERSION);
        if let Some(token) = self.token.as_ref() {
            req.header(AUTHORIZATION, format!("Bearer {token}"))
        } else {
            req
        }
    }

    pub async fn get_items(&self) -> Result<Vec<TreeItem>, reqwest::Error> {
        let mut url = Url::parse("https://api.github.com/repos/").unwrap();
        {
            let (owner, repo_name) = self.repo.split_once('/').unwrap();
            let mut segments = url.path_segments_mut().unwrap();
            segments.push(owner);
            segments.push(repo_name);
            segments.push("git");
            segments.push("trees");
            segments.push(&self.sha);
        }
        url.query_pairs_mut().append_pair("recursive", "1");

        Ok(self
            .get(url)
            .send()
            .await?
            .json::<TreeResponse>()
            .await?
            .tree)
    }

    pub async fn get_blob(&self, sha: impl AsRef<str>) -> Result<String, reqwest::Error> {
        let sha = sha.as_ref();
        let mut url = Url::parse("https://api.github.com/repos/").unwrap();
        {
            let (owner, repo_name) = self.repo.split_once('/').unwrap();
            let mut segments = url.path_segments_mut().unwrap();
            segments.push(owner);
            segments.push(repo_name);
            segments.push("git");
            segments.push("blobs");
            segments.push(sha);
        }

        self.get(url)
            .header(ACCEPT, "application/vnd.github.raw+json")
            .send()
            .await?
            .text()
            .await
    }
}

fn is_valid_github_repo(repo: &str) -> bool {
    let Some((owner, name)) = repo.split_once('/') else {
        return false;
    };

    if owner.is_empty()
        || owner.len() > 39
        || !owner.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        || owner.starts_with('-')
        || owner.ends_with('-')
        || owner.contains("--")
    {
        return false;
    }

    if name.is_empty()
        || name.len() > 100
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        || name == "."
        || name == ".."
    {
        return false;
    }

    true
}

fn is_valid_github_branch(branch: &str) -> bool {
    if branch.is_empty() || branch.len() > 255 || branch.contains("..") || branch.starts_with('/') {
        return false;
    }

    // A bit more permissive for branches but still safe for URL construction
    branch.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/' || c == '+'
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_validation() {
        assert!(is_valid_github_repo("owner/repo"));
        assert!(is_valid_github_repo("owner-name/repo.name_123"));
        assert!(!is_valid_github_repo("owner"));
        assert!(!is_valid_github_repo("owner/repo/extra"));
        assert!(!is_valid_github_repo("-owner/repo"));
        assert!(!is_valid_github_repo("owner-/repo"));
        assert!(!is_valid_github_repo("owner/.."));
        assert!(!is_valid_github_repo("owner/."));
        assert!(!is_valid_github_repo("owner/repo@bad.com"));
    }

    #[test]
    fn test_branch_validation() {
        assert!(is_valid_github_branch("main"));
        assert!(is_valid_github_branch("feature/new-stuff"));
        assert!(is_valid_github_branch("v1.0.0"));
        assert!(is_valid_github_branch("branch+name"));
        assert!(!is_valid_github_branch(""));
        assert!(!is_valid_github_branch("/leading-slash"));
        assert!(!is_valid_github_branch("double..dot"));
        assert!(!is_valid_github_branch("branch with spaces"));
    }
}
