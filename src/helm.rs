use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    process::{Command, Stdio},
};

use lazy_static::lazy_static;
use regex::Regex;
use which::which;

use crate::model::{HelmChartRepo, Helmchart};

lazy_static! {
    static ref ENV_VAR_REGEX: Regex = Regex::new(r"\$\{env\.(?P<env_var>[a-zA-Z0-9_]+)\}").unwrap();
}

pub struct Helm {
    helm_chart_repo: HelmChartRepo,
    default_values: BTreeMap<String, String>,
    helm_binary_path: PathBuf,
}

impl Helm {
    pub fn new(
        helm_chart_repo: &HelmChartRepo,
        default_values: &BTreeMap<String, String>,
        helm_binary_path: &PathBuf,
    ) -> Self {
        Helm {
            helm_chart_repo: helm_chart_repo.to_owned(),
            default_values: default_values.to_owned(),
            helm_binary_path: helm_binary_path.to_owned(),
        }
    }

    fn extract_env_var_name(&self, input: &str) -> Option<String> {
        ENV_VAR_REGEX.captures(input).and_then(|capture| {
            capture
                .name("env_var")
                .map(|env_var| env_var.as_str().to_string())
        })
    }

    fn replace_with_env_var(&self, input: &str) -> anyhow::Result<Option<String>> {
        if let Some(env_var_name) = self.extract_env_var_name(input) {
            let env_var = std::env::var(&env_var_name).map_err(|_| {
                anyhow::anyhow!(
                    "The env var '{}' is not defined, please provide it via 'export {}=\"<your-desired-value>\"'",
                    &env_var_name,
                    &env_var_name
                )
            })?;

            Ok(Some(env_var))
        } else {
            Ok(None)
        }
    }

    pub fn initial_arguments(&self) -> anyhow::Result<Vec<String>> {
        let mut arguments = vec![];

        if let (Some(username), Some(password)) = (
            self.helm_chart_repo.username.to_owned(),
            self.helm_chart_repo.password.to_owned(),
        ) {
            let username = self.replace_with_env_var(&username)?.unwrap_or(username);
            let password = self.replace_with_env_var(&password)?.unwrap_or(password);

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

    pub fn add_repo(&self, helm_chart_repo: &HelmChartRepo) -> anyhow::Result<()> {
        let mut arguments = self.initial_arguments()?;

        arguments.push("repo".to_string());
        arguments.push("add".to_string());
        arguments.push("--force-update".to_string());
        arguments.push(helm_chart_repo.name.to_string());
        arguments.push(helm_chart_repo.url.to_string());

        Command::new(self.helm_binary_path.as_path())
            .args(&arguments)
            .spawn()?
            .wait()?;

        Ok(())
    }

    pub fn upgrade(&self, chart_repo: &String, helmchart: &Helmchart) -> anyhow::Result<()> {
        let mut arguments = self.initial_arguments()?;

        arguments.push("upgrade".to_string());
        arguments.push("--install".to_string());

        let mut all_values = HashMap::new();

        for (key, value) in &self.default_values {
            all_values.insert(key, value);
        }

        for (key, value) in &helmchart.values {
            all_values.insert(key, value);
        }

        let mut values = HashMap::new();

        for (key, value) in all_values {
            let value = self
                .replace_with_env_var(value)?
                .unwrap_or_else(|| value.to_owned());

            values.insert(key, value);
        }

        if !values.is_empty() {
            for (key, value) in values {
                arguments.push("--set".to_string());
                arguments.push(format!("{}={}", key, value.replace(',', r"\,")));
            }
        }

        arguments.push(helmchart.name.to_string());
        arguments.push(format!("{}/{}", &chart_repo, &helmchart.name));
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
