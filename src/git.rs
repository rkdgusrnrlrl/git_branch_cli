use chrono::NaiveDateTime;
use std::fmt::Display;
use std::path::PathBuf;
use std::process::Command;



pub struct GitBranch {
    pub name: String,
    pub committerdate: NaiveDateTime,
}

impl Display for GitBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let date = self.committerdate.format("%Y-%m-%d %H:%M:%S").to_string();
        write!(f, "[{}] {}", date, self.name)
    }
}

pub struct GitClient {
    working_directory: PathBuf,
}

impl GitClient {
    pub fn new(path: &str) -> Self {
        GitClient { working_directory: PathBuf::from(path) }
    }

    pub fn checkout_new_branch(&self, branch_name:&str) -> bool {
        let output = Command::new("git")
            .current_dir(&self.working_directory)
            .arg("checkout")
            .arg("-b")
            .arg(branch_name)
            .output()
            .expect("failed to execute process");
        output.status.success()
    }

    pub fn get_local_branches(&self) -> Vec<String> {
        let output = Command::new("git")
            .current_dir(&self.working_directory)
            .arg("branch")
            .output()
            .expect("failed to execute process");
        let git_branches: Vec<String> = String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|line| line[2..].to_string())
            .collect();
        git_branches
    }

    pub fn checkout(&self, branch_name:&str) -> bool {
        let output = Command::new("git")
            .current_dir(&self.working_directory)
            .arg("checkout")
            .arg(branch_name)
            .output()
            .expect("failed to execute process");
        output.status.success()
    }

    pub fn check_git_exist(&self) -> bool {
        let output = Command::new("git")
            .current_dir(&self.working_directory)
            .arg("status")
            .output()
            .expect("failed to execute process");
        output.status.success()
    }

    pub fn delete_branch(&self, branch: &str) -> bool {
        let output = Command::new("git")
            .current_dir(&self.working_directory)
            .arg("branch")
            .arg("-D")
            .arg(branch)
            .output()
            .expect("failed to execute process");
        if !output.stderr.is_empty() {
            println!("\x1b[93m{}\x1b[0m", String::from_utf8(output.stderr).unwrap());
        }
        
        output.status.success()
    }

    pub fn get_branches(&self) -> Vec<GitBranch> {
        let ref_format =
            "{\"name\": \"%(refname:short)\", \"committerdate\": \"%(committerdate:iso)\"}";
        let format_arg = format!("--format='{ref_format}'", ref_format = ref_format);
        let output = Command::new("git")
            .current_dir(&self.working_directory)
            .arg("for-each-ref")
            .arg("refs/heads/")
            .arg(format_arg)
            .arg("--sort=committerdate")
            .output()
            .expect("failed to execute process");

        let git_branches: Vec<GitBranch> = String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|line| line.trim_start_matches('\'').trim_end_matches('\''))
            .map(|line| json::parse(line).unwrap())
            .map(|value| GitBranch {
                name: value["name"].to_string(),
                committerdate: NaiveDateTime::parse_from_str(
                    &value["committerdate"].to_string(),
                    "%Y-%m-%d %H:%M:%S %z",
                )
                .expect("failed to parse date"),
            })
            .collect();
        git_branches
    }

    pub fn get_remote_last_branch(&self, filter_format: &str) -> Option<String> {
        let output = Command::new("git")
            .current_dir(&self.working_directory)
            .arg("ls-remote")
            .arg("--sort=-v:refname")
            .arg("origin")
            .arg(filter_format)
            .output()
            .expect("failed to execute process");
        let out_str = String::from_utf8(output.stdout).unwrap();
        let out_first = out_str.lines().next();
        let split_some:Vec<&str> = out_first.unwrap_or("").split('\t').collect();
        if split_some.len() >= 2 {
            Some(split_some[1].to_string())
        } else {
            None
        }
    }

    // 추가: 수정된 파일 목록 조회 (git ls-files -m)
    pub fn get_modified_files(&self) -> Vec<String> {
        let output = Command::new("git")
            .current_dir(&self.working_directory)
            .args(["ls-files", "-m"])
            .output()
            .expect("failed to execute process");
        if !output.status.success() {
            return vec![];
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.lines().map(|s| s.to_string()).collect()
    }

    // 추가: 선택한 파일을 원래대로 복원 (git restore)
    pub fn restore_file(&self, file: &str) -> bool {
        let output = Command::new("git")
            .current_dir(&self.working_directory)
            .args(["restore", file])
            .output()
            .expect("failed to execute process");
        output.status.success()
    }
}