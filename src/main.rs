use inquire::{formatter::MultiOptionFormatter, MultiSelect};
use std::env;
use std::str;
mod git;
use clap::{arg, Command};
use chrono::{DateTime, Local};

fn cli() -> Command {
    Command::new("git")
        .about("A fictional versioning CLI")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("delete")
                .about("Delete Local Branch"),
        )
        .subcommand(
            Command::new("recommend")
                .about("recommend stage or release branch name")
                .arg(arg!(<STAGE> "stage"))
                .arg_required_else_help(true),
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
            let branch_name = git::get_remote_last_branch(
                &absolut_path, 
                filter_format.as_str()
            );
            match branch_name {
                Some(branch_name) => {
                    let new_branch_name = branch_name.replace("refs/heads/", "");
                    let version_parts: Vec<&str> = new_branch_name.split('.').collect();
                    let mut sub_ver: i32 = version_parts[1].to_string().parse().unwrap();
                    sub_ver += 1;
                    println!("branch_name: {}.{}", version_parts[0], sub_ver)
                },
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
    let formatter: MultiOptionFormatter<git::GitBranch> = &|a| format!("{} selected branch", a.len());

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
        Err(e) => {
            match e {
                inquire::InquireError::OperationInterrupted => {
                    println!("user canceled");
                }
                _ => println!("Error: {:?}", e),
            }
        }
    }
}
