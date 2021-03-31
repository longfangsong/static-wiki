use async_trait::async_trait;
use baipiao_bot_rust::{
    Bot, Dispatcher, IssueCreatedEvent, IssueReopenedEvent, PullRequestCreatedEvent, Repository,
    RunningInfo,
};
use chrono::SecondsFormat;
use log::info;
use octocrab::{models, params, Octocrab, OctocrabBuilder};
use rand::rngs::OsRng;
use rand::Rng;
use regex::Regex;
use simpler_git::{
    git2,
    git2::{Cred, Signature},
    GitHubRepository,
};
use std::path::Path;
use std::{env, fs, fs::File, io::Write, time::Duration};
use tokio::time;

fn collect_changed_files(diff: &str) -> Vec<&str> {
    let re = Regex::new(r"(\s*)diff --git a/(.+)").unwrap();
    re.find_iter(diff)
        .map(|m| {
            m.as_str()
                .splitn(3, ' ')
                .last()
                .unwrap()
                .trim()
                .trim_start_matches("diff --git ")
        })
        .collect()
}

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

        let filename = if Path::new(&format!(
            "./{}/data/{}/{}/{}",
            repo_info.name, language, answer, filename
        ))
        .exists()
        {
            let digest = md5::compute(content);
            format!("{}-{:?}.md", filename.trim_end_matches(".md"), digest)
        } else {
            filename.to_string()
        };

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

    fn update_file_contents(content: &str, author: &str) -> String {
        let mut splitted = content.split("---");
        let nothing = splitted.next();
        assert_eq!(nothing, Some(""));
        let mut meta = splitted.next().unwrap().to_string();
        let now = chrono::Utc::now();
        meta += &format!(
            "author: {}\nlast_update: {}\n",
            author,
            now.to_rfc3339_opts(SecondsFormat::Secs, true)
        );
        let content = splitted.next().unwrap();
        assert_eq!(splitted.next(), None);
        format!("---\n{}---\n{}", meta, content)
    }

    async fn handle_contribute_issue(
        &self,
        repo: &Repository,
        id: usize,
        title: &str,
        body: &str,
        author: &str,
        locker_id: usize,
    ) {
        info!("Contribute issue created with title {}", title);
        let title = title.strip_prefix("[Contribute] ").unwrap();
        let content_start = body.find("---").unwrap();
        let mut meta = body[..content_start].split('\n').map(|it| it.trim());
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
        self.acquire_lock_with_issue(&repo, locker_id).await;
        let content = Self::update_file_contents(content, author);
        self.save_file_and_push(
            &repo,
            "main",
            language,
            answer,
            &format!("{}.md", title.replace(".", "-")),
            &content,
        )
        .unwrap();
        info!("File saved and pushed {}", title);
        self.comment(&repo, id, "Merged. Thank you for contribution!")
            .await;
        self.close_issue(&repo, id).await;
        // we won't unlock here, build site action will do the unlock job
    }

    async fn acquire_lock_with_issue(&self, repo: &Repository, locker_id: usize) {
        loop {
            let current_holder = self.current_lock_holder(repo).await;
            match current_holder {
                None => {
                    info!("Seems no body is holding the lock, try to acquire it...");
                    self.github_client
                        .issues(&repo.owner, &repo.name)
                        .update(1)
                        .body(&format!("{}", locker_id))
                        .send()
                        .await
                        .map_err(|_| info!("Error raised by GitHub, retry ..."));
                }
                Some(x) if x == locker_id => {
                    info!("Seems I got the lock successfully, waiting for any possible concurrent running locker...");
                    time::sleep(Duration::from_secs(10)).await;
                    let current_holder_after_wait = self.current_lock_holder(&repo).await;
                    match current_holder_after_wait {
                        Some(x) if x == locker_id => {
                            info!("I still have the lock! Acquire success!");
                            break;
                        }
                        _ => info!("I lost the lock TAT, I'll retry ..."),
                    }
                }
                Some(current_holder) => {
                    info!("Blocked by {}, wait and retry ...", current_holder);
                    time::sleep(Duration::from_secs(10)).await;
                    time::sleep(Duration::from_millis(OsRng.gen_range(0..10000))).await;
                }
            }
        }
    }

    async fn current_lock_holder(&self, repo: &Repository) -> Option<usize> {
        self.github_client
            .issues(&repo.owner, &repo.name)
            .get(1)
            .await
            .unwrap()
            .body
            .and_then(|holder| usize::from_str_radix(&holder, 10).ok())
    }

    async fn handle_contribute_pr(&self, repo: &Repository, id: usize, locker_id: usize) {
        info!("Contribute pr created with id {}", id);
        let diff = self
            .github_client
            .pulls(&repo.owner, &repo.name)
            .get_patch(id as _)
            .await
            .unwrap();
        info!("diff: {}", diff);
        let changed_files = collect_changed_files(&diff);
        println!("changed files: {:?}", changed_files);
        let all_files_valid = collect_changed_files(&diff)
            .iter()
            .all(|it| it.starts_with("a/data") && it.ends_with(".md"));
        if all_files_valid {
            self.acquire_lock_with_issue(&repo, locker_id).await;
            self.merge_pr(&repo, id).await;
        } else {
            self.comment(
                repo,
                id,
                "Sorry I cannot make sure your PR is safe to merge.\n@longfangsong PTAL.",
            )
            .await;
        }
    }
}

#[async_trait]
impl Bot for StaticWikiBot {
    async fn on_issue_created(
        &self,
        repo: Repository,
        running_info: RunningInfo,
        event: IssueCreatedEvent,
    ) {
        if event.title.starts_with("[Contribute]") {
            self.handle_contribute_issue(
                &repo,
                event.id,
                &event.title,
                &event.body,
                &event.user,
                running_info.run_id,
            )
            .await;
        }
    }

    async fn on_issue_reopened(
        &self,
        repo: Repository,
        running_info: RunningInfo,
        event: IssueReopenedEvent,
    ) {
        if event.title.starts_with("[Contribute]") {
            self.handle_contribute_issue(
                &repo,
                event.id,
                &event.title,
                &event.body,
                &event.user,
                running_info.run_id,
            )
            .await;
        }
    }

    async fn on_pull_request_created(
        &self,
        repo: Repository,
        running_info: RunningInfo,
        event: PullRequestCreatedEvent,
    ) {
        self.handle_contribute_pr(&repo, event.id, running_info.run_id)
            .await;
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
