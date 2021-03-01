use async_trait::async_trait;
use baipiao_bot_rust::{Bot, Dispatcher, IssueCreatedEvent, Repository};
use log::info;
use octocrab::{models, params, Octocrab, OctocrabBuilder};
use simpler_git::git2::{Cred, Signature};
use simpler_git::{git2, GitHubRepository};
use std::fs::File;
use std::io::Write;
use std::{env, fs};

struct StaticWikiBot {
    github_client: Octocrab,
}

impl StaticWikiBot {
    fn new(token: String) -> Self {
        Self {
            github_client: OctocrabBuilder::new()
                .personal_token(token)
                .build()
                .unwrap(),
        }
    }

    fn save_file_and_push(
        &self,
        repo_info: &Repository,
        branch_name: &str,
        filename: &str,
        content: &str,
    ) -> Result<(), git2::Error> {
        let github_repo = GitHubRepository {
            owner: repo_info.owner.clone(),
            name: repo_info.name.clone(),
        };
        let mut repo = github_repo.clone(
            format!("./{}", &repo_info.name),
            Some(|| {
                Cred::userpass_plaintext("baipiao-bot", &env::var("BAIPIAO_BOT_TOKEN").unwrap())
            }),
        )?;
        if branch_name != "main" {
            repo.pull(
                branch_name,
                Some(|| {
                    Cred::userpass_plaintext("baipiao-bot", &env::var("BAIPIAO_BOT_TOKEN").unwrap())
                }),
            )?;
        }
        fs::create_dir_all(format!("./{}/data/zh/what/", repo_info.name)).unwrap();
        let mut file =
            File::create(format!("./{}/data/zh/what/{}", repo_info.name, filename)).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        drop(file);
        let tree_id = repo.add_all()?;
        let signature = Signature::now("baipiao-bot", "moss_the_bot@163.com")?;
        repo.commit(
            tree_id,
            "contribute: merge contribution from issue",
            signature,
        )?;
        repo.push(
            || Cred::userpass_plaintext("baipiao-bot", &env::var("BAIPIAO_BOT_TOKEN").unwrap()),
            branch_name,
        )
    }

    async fn merge_pr(&self, repo: &Repository, pr_id: usize) {
        let pr = self
            .github_client
            .pulls(&repo.owner, &repo.name)
            .get(pr_id as _)
            .await
            .unwrap();
        self.github_client
            .pulls(&repo.owner, &repo.name)
            .merge(pr_id as _)
            .title(&pr.title)
            .message(&pr.title)
            .method(params::pulls::MergeMethod::Squash)
            .send()
            .await
            .unwrap();
    }

    async fn comment(&self, repo: &Repository, issue_id: usize, content: &str) {
        self.github_client
            .issues(&repo.owner, &repo.name)
            .create_comment(issue_id as u64, content)
            .await
            .unwrap();
        info!("commented on {}/{}#{}", repo.owner, repo.name, issue_id);
    }

    async fn close_issue(&self, repo: &Repository, issue_id: usize) {
        self.github_client
            .issues(&repo.owner, &repo.name)
            .update(issue_id as _)
            .state(models::IssueState::Closed)
            .send()
            .await
            .unwrap();
    }
}

#[async_trait]
impl Bot for StaticWikiBot {
    async fn on_issue_created(&self, repo: Repository, event: IssueCreatedEvent) {
        if event.title.starts_with("[Contribute]") {
            info!("Contribute issue created with title {}", event.title);
            let title = event.title.strip_prefix("[Contribute] ").unwrap();
            let content_start = event.body.find("---").unwrap();
            let content = &event.body[content_start..];
            self.save_file_and_push(&repo, "main", &format!("{}.md", title), content)
                .unwrap();
            info!("File saved and pushed {}", event.title);
            self.comment(&repo, event.id, "Merged").await;
            self.close_issue(&repo, event.id).await;
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let token = env::var("BAIPIAO_BOT_TOKEN").unwrap();

    let bot = StaticWikiBot::new(token);
    let dispatcher = Dispatcher::new(bot);
    let content = env::var("JSON").unwrap();
    let input: serde_json::Value = serde_json::from_str(&content).unwrap();
    dispatcher.dispatch_event(input).await;
}
