use clap::Arg;
use inquire::Select;
use inquire::{formatter::MultiOptionFormatter, MultiSelect};
use std::env;
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
    match matches.subcommand() {
        Some(("recommend", sub_matches)) => {
            let path = &String::from(".");
            let stage = sub_matches.get_one::<String>("STAGE").expect("required");
            let absolut_path = get_absolute_path(path);
            let now: DateTime<Local> = Local::now();
            let yymmdd = now.format("%Y%m%d");
            let filter_format = format!(
                "refs/heads/{stage}/{yymmdd}*",
                stage = stage,
                yymmdd = yymmdd
            );
            let branch_name = git::get_remote_last_branch(&absolut_path, filter_format.as_str());
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
            let path = &String::from(".");
            let absolut_path = get_absolute_path(path);
            if !git::check_git_exist(&absolut_path) {
                panic!("{} not a git repository", path)
            }
            let git_branches = git::get_branches(&absolut_path);
            multi_select(git_branches, &absolut_path);
        }
        Some(("branch", sub_matches)) => {
            let new = sub_matches.get_one::<String>("new_branch");

            println!("branche!!");
            match new {
                Some(branch) => println!("New branch name: {}", branch),
                None => {
                    let path = &String::from(".");
                    let branches = git::get_local_branches(path);
                    let selected_branch = select_branch(path, branches).unwrap();
                    git::checkout(path, &selected_branch);
                    println!("done")
                }
            }
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    }
}

fn get_absolute_path(path: &String) -> String {
    let mut dir = env::current_dir().unwrap();
    dir.push(path);
    let absolut_path = String::from(dir.canonicalize().unwrap().to_str().unwrap());
    absolut_path
}

fn multi_select(options: Vec<git::GitBranch>, path: &str) {
    let formatter: MultiOptionFormatter<git::GitBranch> =
        &|a| format!("{} selected branch", a.len());

    let ans = MultiSelect::new("Select branch list to delete:", options)
        .with_formatter(formatter)
        .prompt();

    match ans {
        Ok(selected_branches) => {
            selected_branches.iter().for_each(|branch| {
                let branch_name = branch.name.as_str();
                git::delete_git_branch(path, branch_name);
            });
        }
        Err(e) => match e {
            inquire::InquireError::OperationInterrupted => {
                println!("user canceled");
            }
            _ => println!("Error: {:?}", e),
        },
    }
}

// 체크 아웃 하기 위해 브랜치 선택 하는 코드
fn select_branch(path: &str, branch_names: Vec<String>) -> Result<String, SelectBranchError> {
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
