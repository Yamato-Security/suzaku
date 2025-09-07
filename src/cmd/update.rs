use crate::core::rules::load_rules_from_dir;
use crate::core::util::p;
use git2::{ErrorCode, Repository};
use hashbrown::{HashMap, HashSet};
use serde_json::Value;
use std::fs::{self, create_dir};
use std::path::{Path, PathBuf};

use crate::core::color::SuzakuColor::Orange;
use crate::core::log_source::LogSource;
use termcolor::Color;

pub fn start_update_rules(no_color: bool) {
    // エラーが出た場合はインターネット接続がそもそもできないなどの問題点もあるためエラー等の出力は行わない
    let latest_version_data = get_latest_suzaku_version().unwrap_or_default();
    let now_version = &format!("v{}", env!("CARGO_PKG_VERSION"));

    match update_rules() {
        Ok(output) => {
            if output != "You currently have the latest rules." {
                p(Orange.rdg(no_color), "Rules updated successfully.", true);
            }
        }
        Err(e) => {
            if e.message().is_empty() {
                p(None, "Failed to update rules.", true);
            } else {
                p(None, &format!("Failed to update rules. {e:?}  "), true);
            }
        }
    }

    let split_now_version = &now_version
        .replace("-dev", "")
        .replace("v", "")
        .split('.')
        .filter_map(|x| x.parse().ok())
        .collect::<Vec<i8>>();
    let split_latest_version = &latest_version_data
        .as_ref()
        .unwrap_or(now_version)
        .replace('"', "")
        .replace("v", "")
        .split('.')
        .filter_map(|x| x.parse().ok())
        .collect::<Vec<i8>>();
    if split_latest_version > split_now_version {
        p(
            Orange.rdg(no_color),
            &format!(
                "There is a new version of suzaku: {}",
                latest_version_data.unwrap().replace('\"', "")
            ),
            true,
        );
        p(
            Orange.rdg(no_color),
            "You can download it at https://github.com/Yamato-Security/suzaku/releases",
            true,
        );
    }
    println!();
}

/// get latest suzaku version number.
fn get_latest_suzaku_version() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let text = ureq::get("https://api.github.com/repos/Yamato-Security/suzaku/releases/latest")
        .header("User-Agent", "SuzakuUpdateChecker")
        .header("Accept", "application/vnd.github.v3+json")
        .call()?
        .body_mut()
        .read_to_string()?;
    let json_res: Value = serde_json::from_str(&text)?;

    if json_res["tag_name"].is_null() {
        Ok(None)
    } else {
        Ok(Some(json_res["tag_name"].to_string()))
    }
}

/// update rules(suzaku-rules subrepository)
pub fn update_rules() -> Result<String, git2::Error> {
    let mut result;
    let mut prev_modified_rules: HashSet<String> = HashSet::default();
    let suzaku_repo = Repository::open(Path::new("../.."));
    let rule_path = "./rules";
    let rule_path = Path::new(rule_path);
    let suzaku_rule_repo = Repository::open(rule_path);
    if suzaku_repo.is_err() && suzaku_rule_repo.is_err() {
        p(
            None,
            "Attempting to git clone the suzaku-rules repository into the rules folder.",
            true,
        );
        // execution git clone of suzaku-rules repository when failed open suzaku repository.
        result = clone_rules(rule_path);
    } else if suzaku_rule_repo.is_ok() {
        // case of exist suzaku-rules repository
        _repo_main_reset_hard(suzaku_rule_repo.as_ref().unwrap())?;
        // case of failed fetching origin/main, git clone is not executed so network error has occurred possibly.
        prev_modified_rules = get_updated_rules(&rule_path.to_path_buf());
        result = pull_repository(&suzaku_rule_repo.unwrap());
    } else {
        // case of no exist suzaku-rules repository in rules.
        // execute update because submodule information exists if suzaku repository exists submodule information.

        let rules_path = Path::new("../../rules");
        if !rules_path.exists() {
            create_dir(rules_path).ok();
        }
        let suzaku_repo = suzaku_repo.unwrap();
        let submodules = suzaku_repo.submodules()?;
        let mut is_success_submodule_update = true;
        // submodule rules erase path is hard coding to avoid unintentional remove folder.
        fs::remove_dir_all(".git/.submodule/rules").ok();
        for mut submodule in submodules {
            submodule.update(true, None)?;
            let submodule_repo = submodule.open()?;
            if let Err(e) = pull_repository(&submodule_repo) {
                p(None, &format!("[Alert]Failed submodule update. {e}"), true);
                is_success_submodule_update = false;
            }
        }
        if is_success_submodule_update {
            result = Ok("Successed submodule update".to_string());
        } else {
            result = Err(git2::Error::from_str(&String::default()));
        }
    }
    if result.is_ok() {
        let updated_modified_rules = get_updated_rules(&rule_path.to_path_buf());
        result = print_diff_modified_rule_dates(prev_modified_rules, updated_modified_rules);
    }
    result
}

/// hard reset in main branch
fn _repo_main_reset_hard(input_repo: &Repository) -> Result<(), git2::Error> {
    let branch = input_repo
        .find_branch("main", git2::BranchType::Local)
        .unwrap();
    let local_head = branch.get().target().unwrap();
    let object = input_repo.find_object(local_head, None).unwrap();
    match input_repo.reset(&object, git2::ResetType::Hard, None) {
        Ok(()) => Ok(()),
        _ => Err(git2::Error::from_str("Failed reset main branch in rules")),
    }
}

/// Pull(fetch and fast-forward merge) repository to input_repo.
fn pull_repository(input_repo: &Repository) -> Result<String, git2::Error> {
    match input_repo
        .find_remote("origin")?
        .fetch(&["main"], None, None)
    {
        Ok(it) => it,
        Err(e) => {
            p(
                None,
                &format!("Failed git fetch to rules folder. {e}"),
                true,
            );
            return Err(git2::Error::from_str(&String::default()));
        }
    };
    let fetch_head = input_repo.find_reference("FETCH_HEAD")?;
    let fetch_commit = input_repo.reference_to_annotated_commit(&fetch_head)?;
    let analysis = input_repo.merge_analysis(&[&fetch_commit])?;
    if analysis.0.is_up_to_date() {
        Ok("Already up to date".to_string())
    } else if analysis.0.is_fast_forward() {
        let mut reference = input_repo.find_reference("refs/heads/main")?;
        reference.set_target(fetch_commit.id(), "Fast-Forward")?;
        input_repo.set_head("refs/heads/main")?;
        input_repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        Ok("Finished fast forward merge.".to_string())
    } else if analysis.0.is_normal() {
        p(
            None,
            "update-rules option is git Fast-Forward merge only. please check your rules folder.",
            true,
        );
        Err(git2::Error::from_str(&String::default()))
    } else {
        Err(git2::Error::from_str(&String::default()))
    }
}

/// git clone でhauyabusa-rules レポジトリをrulesフォルダにgit cloneする関数
fn clone_rules(rules_path: &Path) -> Result<String, git2::Error> {
    match Repository::clone(
        "https://github.com/Yamato-Security/suzaku-rules.git",
        rules_path,
    ) {
        Ok(_repo) => {
            println!("Finished cloning the suzaku-rules repository.");
            Ok("Finished clone".to_string())
        }
        Err(e) => {
            if e.code() == ErrorCode::Exists {
                p(
                    None,
                    "You need to update the rules as the user that you downloaded suzaku with.\n        You can also move or delete the current rules folder to sync to the latest rules.",
                    true,
                );
            } else {
                p(
                    None,
                    "Failed to git clone into the rules folder. Please rename your rules folder name.",
                    true,
                );
            }
            Err(e)
        }
    }
}

/// Create rules folder files Hashset. Format is "[rule title in yaml]|[filepath]|[filemodified date]|[rule type in yaml]"
fn get_updated_rules(rule_folder_path: &PathBuf) -> HashSet<String> {
    let rulefile_loader = load_rules_from_dir(rule_folder_path, &LogSource::All);

    HashSet::from_iter(rulefile_loader.into_iter().map(|yaml| {
        let yaml_date = yaml.date.unwrap_or("-".to_string());

        format!(
            "{}|{}|{}|{:#?}",
            yaml.logsource.product.unwrap_or("Other".to_string()),
            yaml.title.as_str(),
            yaml.modified.unwrap_or(yaml_date),
            yaml.description
        )
    }))
}

/// print updated rule files.
fn print_diff_modified_rule_dates(
    prev_sets: HashSet<String>,
    updated_sets: HashSet<String>,
) -> Result<String, git2::Error> {
    let diff = updated_sets.into_iter().filter(|k| !prev_sets.contains(k));
    let mut update_count_by_rule_type: HashMap<String, u128> = HashMap::new();
    for diff_key in diff {
        let tmp: Vec<&str> = diff_key.split('|').collect();
        *update_count_by_rule_type
            .entry(tmp[0].to_string())
            .or_insert(0b0) += 1;
        p(None, &format!(" - {} (Modified: {})", tmp[1], tmp[2]), true);
    }
    if !update_count_by_rule_type.is_empty() {
        println!();
    }
    for (key, value) in &update_count_by_rule_type {
        let msg = format!("Updated {key} rules: ");
        p(Some(Color::Rgb(0, 255, 0)), &msg, false);
        p(None, value.to_string().as_str(), true);
    }
    if !&update_count_by_rule_type.is_empty() {
        println!();
        Ok("Rule updated".to_string())
    } else {
        p(
            Some(Color::Rgb(255, 175, 0)),
            "You currently have the latest rules.",
            true,
        );
        Ok("You currently have the latest rules.".to_string())
    }
}
