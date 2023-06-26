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
            Command::new("delete_branch")
                .about("Delete Local Branch")
                .arg(arg!(<PATH> "local repo path"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("recommend")
                .about("recommend stage or release branch name")
                .arg(arg!(<PATH> "local repo path"))
                .arg(arg!(<STAGE> "stage"))
                .arg_required_else_help(true),
        )
}

fn main() {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("recommend", sub_matches)) => {
            let path = sub_matches.get_one::<String>("PATH").expect("required");
            let stage = sub_matches.get_one::<String>("STAGE").expect("required");
            let absolut_path = get_absolute_path(path);
            let branch_name = git::get_remote_last_branch(&absolut_path, stage);
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
        Some(("delete_branch", sub_matches)) => {
            let path = sub_matches.get_one::<String>("PATH").expect("required");
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
    let formatter: MultiOptionFormatter<git::GitBranch> = &|a| format!("{} different fruits", a.len());

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
