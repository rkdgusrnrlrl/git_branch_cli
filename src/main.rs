use clap::Arg;
use git::GitClient;
use inquire::Select;
use inquire::{formatter::MultiOptionFormatter, MultiSelect};
use std::str;
mod git;
use chrono::{DateTime, Local};
use clap::{arg, Command};
use std::fmt;

#[derive(Debug)]
enum SelectBranchError {
    UserCanceled,
    OtherError(String),
}

// ✅ `fmt::Display` 구현 (에러 메시지 출력 가능)
impl fmt::Display for SelectBranchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SelectBranchError::UserCanceled => write!(f, "User canceled the operation."),
            SelectBranchError::OtherError(msg) => write!(f, "An error occurred: {}", msg),
        }
    }
}


fn cli() -> Command {
    Command::new("git")
        .about("A fictional versioning CLI")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(Command::new("delete").about("Delete Local Branch"))
        .subcommand(Command::new("revert").about("revert selected files"))
        .subcommand(
            Command::new("recommend")
                .about("recommend stage or release branch name")
                .arg(arg!(<STAGE> "stage"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("branch").about("branch list").arg(
                Arg::new("new_branch")
                    .short('n')
                    .long("new")
                    .help("new branch"),
            ),
        )
}

fn main() {
    let matches = cli().get_matches();
    let git_client = GitClient::new(".");
    match matches.subcommand() {
        Some(("recommend", sub_matches)) => {
            let stage = sub_matches.get_one::<String>("STAGE").expect("required");
            let now: DateTime<Local> = Local::now();
            let yymmdd = now.format("%Y%m%d");
            let filter_format = format!(
                "refs/heads/{stage}/{yymmdd}*",
                stage = stage,
                yymmdd = yymmdd
            );
            let branch_name = git_client.get_remote_last_branch(filter_format.as_str());
            match branch_name {
                Some(branch_name) => {
                    let new_branch_name = branch_name.replace("refs/heads/", "");
                    let version_parts: Vec<&str> = new_branch_name.split('.').collect();
                    let mut sub_ver: i32 = version_parts[1].to_string().parse().unwrap();
                    sub_ver += 1;
                    println!("branch_name: {}.{}", version_parts[0], sub_ver)
                }
                None => {
                    let now: DateTime<Local> = Local::now();
                    let yymmdd = now.format("%Y%m%d");
                    println!("branch_name: {}/{}.1", stage, yymmdd)
                }
            }
        }
        Some(("delete", _sub_matches)) => {
            if !git_client.check_git_exist() {
                panic!("not a git repository")
            }
            let git_branches = git_client.get_branches();
            let selected_branches = multi_select(git_branches).expect("error");
            
            selected_branches.iter().for_each(|branch| {
                git_client.delete_branch(branch.as_str());
            });
        }
        Some(("branch", sub_matches)) => {
            let new = sub_matches.get_one::<String>("new_branch");
            
            match new {
                Some(branch) => {
                    git_client.checkout_new_branch(branch);
                    println!("done")
                },
                None => {
                    let branches = git_client.get_local_branches();
                    let selected_branch = select_branch(branches).unwrap();
                    git_client.checkout(&selected_branch);
                    println!("done")
                }
            }
        }
        Some(("revert", _)) => {
            let files = git_client.get_modified_files();
            let selected_files = multi_select_str(files).unwrap();
            selected_files.iter().for_each(|f| {
                let is_ok = git_client.restore_file(f);
                println!("{}: {}",f, is_ok)
            })
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    }
}

fn multi_select(options: Vec<git::GitBranch>) -> Result<Vec<String>, SelectBranchError> {
    let formatter: MultiOptionFormatter<git::GitBranch> =
        &|a| format!("{} selected branch", a.len());

    let ans = MultiSelect::new("Select branch list to delete:", options)
        .with_formatter(formatter)
        .prompt();

    match ans {
        Ok(selected_branches) => {
            Ok(selected_branches.iter().map(|branch| branch.name.clone()).collect())
        }
        Err(e) => match e {
            inquire::InquireError::OperationInterrupted => {
                Err(SelectBranchError::UserCanceled)
            }
            _ => Err(SelectBranchError::OtherError(e.to_string()))
        },
    }
}

fn multi_select_str(options: Vec<String>) -> Result<Vec<String>, SelectBranchError> {
    let ans = MultiSelect::new("Select file list to restore:", options)
        .prompt();

    match ans {
        Ok(selected_files) => {
            Ok(selected_files)
        }
        Err(e) => match e {
            inquire::InquireError::OperationInterrupted => {
                Err(SelectBranchError::UserCanceled)
            }
            _ => Err(SelectBranchError::OtherError(e.to_string()))
        },
    }
}

// 체크 아웃 하기 위해 브랜치 선택 하는 코드
fn select_branch(branch_names: Vec<String>) -> Result<String, SelectBranchError> {
    let ans = Select::new("Please select the branch to check out.", branch_names).prompt();

    match ans {
        Ok(choice) => {
            Ok(choice)
        },
        Err(e) => match e {
            inquire::InquireError::OperationInterrupted => {
                Err(SelectBranchError::UserCanceled)
            }
            _ => Err(SelectBranchError::OtherError(e.to_string()))
        },
    }
}
