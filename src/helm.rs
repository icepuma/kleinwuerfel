use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use tempfile::NamedTempFile;
use which::which;

pub struct Helm {
    username: String,
    password: String,
    helm_binary_path: PathBuf,
}

impl Helm {
    pub fn new(username: String, password: String, helm_binary_path: &PathBuf) -> Self {
        Helm {
            username,
            password,
            helm_binary_path: helm_binary_path.to_owned(),
        }
    }

    pub fn login(&self, helm_repo_url: &String) -> anyhow::Result<bool> {
        let status = Command::new(self.helm_binary_path.as_path())
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .arg("registry")
            .arg("login")
            .arg(&helm_repo_url)
            .arg("--username")
            .arg(&self.username)
            .arg("--password")
            .arg(&self.password)
            .spawn()?
            .wait_with_output()?;

        Ok(status.status.success())
    }

    pub fn add_repo(&self, chart_repo: &String, helm_repo_url: &String) -> anyhow::Result<()> {
        Command::new(self.helm_binary_path.as_path())
            .arg("repo")
            .arg("add")
            .arg("--username")
            .arg(&self.username)
            .arg("--password")
            .arg(&self.password)
            .arg(&chart_repo)
            .arg(format!("{}/{}", &helm_repo_url, &chart_repo))
            .spawn()?
            .wait()?;

        Ok(())
    }

    pub fn upgrade(&self, chart_repo: &String, chart_name: &String) -> anyhow::Result<()> {
        let mut config_file = NamedTempFile::new()?;

        let config_file_content = format!(
            r###"
environment:
    local: true
imageRegistry:
    username: '{}'
    password: '{}'"###,
            &self.username, &self.password
        );

        config_file.write_all(config_file_content.as_bytes())?;

        Command::new(self.helm_binary_path.as_path())
            .arg("upgrade")
            .arg("--install")
            .arg("--username")
            .arg(&self.username)
            .arg("--password")
            .arg(&self.password)
            .arg(&chart_name)
            .arg(format!("{}/{}", &chart_repo, &chart_name))
            .arg("-f")
            .arg(config_file.path())
            .arg("--wait")
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
