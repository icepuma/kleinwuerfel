use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use lazy_static::lazy_static;
use regex::Regex;
use tempfile::NamedTempFile;
use which::which;

use crate::model::HelmChartRepo;

lazy_static! {
    static ref ENV_VAR_REGEX: Regex = Regex::new(r"\$\{env\.(?P<env_var>[a-zA-Z0-9_]+)\}").unwrap();
}

pub struct Helm {
    helm_chart_repo: HelmChartRepo,
    helm_binary_path: PathBuf,
}

impl Helm {
    pub fn new(helm_chart_repo: &HelmChartRepo, helm_binary_path: &PathBuf) -> Self {
        Helm {
            helm_chart_repo: helm_chart_repo.to_owned(),
            helm_binary_path: helm_binary_path.to_owned(),
        }
    }

    fn extract_env_var_name(&self, input: &str) -> anyhow::Result<String> {
        let env_var_name = ENV_VAR_REGEX
            .captures(input)
            .and_then(|capture| {
                capture
                    .name("env_var")
                    .map(|env_var| env_var.as_str().to_string())
            })
            .ok_or_else(|| {
                anyhow::anyhow!("Reading env var name from input='{}' didn't work.", input)
            })?;

        Ok(env_var_name)
    }

    fn extract_env_var_names(&self, input: &str) -> anyhow::Result<Vec<String>> {
        Ok(ENV_VAR_REGEX
            .captures_iter(input)
            .map(|capture| {
                capture
                    .name("env_var")
                    .map(|env_var| env_var.as_str().to_string())
                    .unwrap_or_default()
            })
            .collect::<Vec<String>>())
    }

    fn replace_with_env_var(&self, input: &str) -> anyhow::Result<String> {
        let env_var_name = self.extract_env_var_name(input)?;

        let env_var = std::env::var(&env_var_name).map_err(|_| {
            anyhow::anyhow!(
                "The env var '{}' is not defined, please provide it via 'export {}=\"<your-desired-value>\"'",
                &env_var_name,
                &env_var_name
            )
        })?;

        Ok(env_var)
    }

    pub fn initial_arguments(&self) -> anyhow::Result<Vec<String>> {
        let mut arguments = vec![];

        if let (Some(username), Some(password)) = (
            self.helm_chart_repo.username.to_owned(),
            self.helm_chart_repo.password.to_owned(),
        ) {
            let username = self.replace_with_env_var(&username)?;
            let password = self.replace_with_env_var(&password)?;

            arguments.push("--username".to_string());
            arguments.push(username);
            arguments.push("--password".to_string());
            arguments.push(password);
        }

        Ok(arguments)
    }

    pub fn login(&self, helm_repo_url: &String) -> anyhow::Result<bool> {
        // if no username and password is set, we mark the login successful to jump into the next code branch
        if self.helm_chart_repo.username.is_none() && self.helm_chart_repo.password.is_none() {
            return Ok(true);
        }

        let mut arguments = self.initial_arguments()?;

        arguments.push("registry".to_string());
        arguments.push("login".to_string());
        arguments.push(helm_repo_url.to_string());

        let status = Command::new(self.helm_binary_path.as_path())
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .args(&arguments)
            .spawn()?
            .wait_with_output()?;

        Ok(status.status.success())
    }

    pub fn add_repo(&self, chart_repo: &String, helm_repo_url: &String) -> anyhow::Result<()> {
        let mut arguments = self.initial_arguments()?;

        arguments.push("repo".to_string());
        arguments.push("add".to_string());
        arguments.push("--force-update".to_string());
        arguments.push(chart_repo.to_string());
        arguments.push(helm_repo_url.to_string());

        Command::new(self.helm_binary_path.as_path())
            .args(&arguments)
            .spawn()?
            .wait()?;

        Ok(())
    }

    pub fn upgrade(&self, chart_repo: &String, chart_name: &String) -> anyhow::Result<()> {
        let values = if let Some(values) = self.helm_chart_repo.values.to_owned() {
            let env_var_names = self.extract_env_var_names(&values)?;
            let replacements = env_var_names
                .iter()
                .map(|env_var_name| {
                    std::env::var(&env_var_name)
                        .map(|value| (format!("${{env.{}}}", &env_var_name), value))
                        .unwrap_or_else(|_| {
                            (format!("${{env.{}}}", &env_var_name), "lel".to_string())
                        })
                })
                .collect::<Vec<(String, String)>>();

            let mut values = values;

            for (env_var, value) in &replacements {
                values = values.replace(env_var, value);
            }

            Some(values)
        } else {
            None
        };

        let mut arguments = self.initial_arguments()?;

        arguments.push("upgrade".to_string());
        arguments.push("--install".to_string());
        arguments.push(chart_name.to_string());
        arguments.push(format!("{}/{}", &chart_repo, &chart_name));

        let mut values_file = NamedTempFile::new()?;

        if let Some(values) = values {
            values_file.write_all(values.as_bytes())?;

            arguments.push("-f".to_string());
            arguments.push(values_file.path().display().to_string());
        }

        arguments.push("--wait".to_string());

        Command::new(self.helm_binary_path.as_path())
            .args(&arguments)
            .spawn()?
            .wait()?;

        Ok(())
    }

    pub fn list() -> anyhow::Result<()> {
        let helm_binary = which("helm")?;

        Command::new(helm_binary).arg("list").spawn()?.wait()?;

        Ok(())
    }
}
