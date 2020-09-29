use std::path::Path;
use std::process::Command;

use structopt::StructOpt;

use rayon::prelude::*;

#[derive(StructOpt)]
#[structopt(name = "git-mirror")]
enum Opt {
    Add { url: String, path: Option<String> },
    Update { path: Option<String> },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match Opt::from_args() {
        Opt::Add { url, path } => {
            if let Some(path) = path.or_else(|| url.split('/').last().map(|s| s.to_owned())) {
                println!("Mirroring {} to {}", url, path);

                let path = Path::new(&path);
                let path_valid = if !path.exists() {
                    true
                } else {
                    path.read_dir()
                        .map(|mut dir| dir.next().is_none())
                        .unwrap_or(false)
                };

                if path_valid {
                    let output = Command::new("git")
                        .args(&[
                            "clone",
                            "--mirror",
                            &url,
                            &path.as_os_str().to_string_lossy(),
                        ])
                        .output();

                    if let Ok(output) = output {
                        if output.status.success() {
                            println!("Success!")
                        } else {
                            println!("Failure: {:?}", output.stderr);
                        }
                    }
                } else {
                    println!("The provided path is not valid");
                }
            }
        }
        Opt::Update { path } => {
            let path = path.unwrap_or_else(|| ".".to_owned());
            Path::new(&path)
                .read_dir()?
                .par_bridge()
                .filter_map(Result::ok)
                .filter(|e| {
                    let mut git_config_path = e.path();
                    git_config_path.push("config");
                    git_config_path.exists()
                })
                .for_each(|dir_entry| {
                    let output = Command::new("git")
                        .current_dir(dir_entry.path())
                        .args(&["remote", "update"])
                        .output();
                    match output {
                        Ok(output) => {
                            if output.status.success() {
                                println!("{:?} - Success!", dir_entry.path());
                            } else {
                                println!("{:?} - Failure: {:?}", dir_entry.path(), output.stderr);
                            }
                        }
                        Err(output) => {
                            println!("{:?} - Error: {:?}", dir_entry.path(), output);
                        }
                    }
                });
        }
    }

    Ok(())
}
