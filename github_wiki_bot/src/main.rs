use async_trait::async_trait;
use baipiao_bot_rust::{Bot, Dispatcher, IssueCreatedEvent, IssueReopenedEvent, Repository};
use log::info;
use octocrab::{models, params, Octocrab, OctocrabBuilder};
use simpler_git::{
    git2,
    git2::{Cred, Signature},
    GitHubRepository,
};
use std::{env, fs, fs::File, io::Write};

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
        language: &str,
        answer: &str,
        filename: &str,
        content: &str,
    ) -> Result<(), git2::Error> {
        info!("try to add file {}/{}/{}", language, answer, content);
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
        fs::create_dir_all(format!(
            "./{}/data/{}/{}/",
            repo_info.name, language, answer
        ))
        .unwrap();
        let mut file = File::create(format!(
            "./{}/data/{}/{}/{}",
            repo_info.name, language, answer, filename
        ))
        .unwrap();
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

    async fn handle_contribute_issue(&self, repo: Repository, id: usize, title: &str, body: &str) {
        info!("Contribute issue created with title {}", title);
        let title = title.strip_prefix("[Contribute] ").unwrap();
        let content_start = body.find("---").unwrap();
        let mut meta = body[..content_start].split("\n").map(|it| it.trim());
        let language = meta
            .clone()
            .find(|it| it.starts_with("language:"))
            .map(|it| it.trim_start_matches("language:").trim())
            .unwrap();
        let answer = meta
            .find(|it| it.starts_with("answer:"))
            .map(|it| it.trim_start_matches("answer:").trim())
            .unwrap();
        let content = &body[content_start..];
        self.save_file_and_push(
            &repo,
            "main",
            language,
            answer,
            &format!("{}.md", title),
            content,
        )
        .unwrap();
        info!("File saved and pushed {}", title);
        self.comment(&repo, id, "Merged").await;
        self.close_issue(&repo, id).await;
    }
}

#[async_trait]
impl Bot for StaticWikiBot {
    async fn on_issue_created(&self, repo: Repository, event: IssueCreatedEvent) {
        if event.title.starts_with("[Contribute]") {
            self.handle_contribute_issue(repo, event.id, &event.title, &event.body)
                .await;
        }
    }

    async fn on_issue_reopened(&self, repo: Repository, event: IssueReopenedEvent) {
        if event.title.starts_with("[Contribute]") {
            self.handle_contribute_issue(repo, event.id, &event.title, &event.body)
                .await;
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
