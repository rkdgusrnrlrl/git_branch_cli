use chrono::NaiveDateTime;
use std::fmt::Display;
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

pub fn check_git_exist(path: &str) -> bool {
    let output = Command::new("git")
        .current_dir(path)
        .arg("status")
        .output()
        .expect("failed to execute process");
    output.status.success()
}

pub fn delete_git_branch(path: &str, branch: &str) -> bool {
    let output = Command::new("git")
        .current_dir(path)
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


pub fn get_branches(path: &str) -> Vec<GitBranch> {
    let ref_format =
        "{\"name\": \"%(refname:short)\", \"committerdate\": \"%(committerdate:iso)\"}";
    let format_arg = format!("--format='{ref_format}'", ref_format = ref_format);
    let output = Command::new("git")
        .current_dir(path)
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


pub fn get_remote_last_branch(path: &str, filter_format: &str) -> Option<String> {
    
    let output = Command::new("git")
        .current_dir(path)
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