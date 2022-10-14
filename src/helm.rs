use std::{
    io::Write,
    process::{Command, Stdio},
};

use tempfile::NamedTempFile;
use which::which;

pub struct Helm {
    username: String,
    password: String,
}

impl Helm {
    pub fn new(username: String, password: String) -> Self {
        Helm { username, password }
    }

    pub fn login(&self, helm_repo_url: &String) -> anyhow::Result<bool> {
        let helm_binary = which("helm")?;

        let status = Command::new(&helm_binary)
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
        let helm_binary = which("helm")?;

        Command::new(&helm_binary)
            .arg("repo")
            .arg("add")
            .arg(&chart_repo)
            .arg(format!("{}/{}", helm_repo_url, &chart_repo))
            .spawn()?
            .wait()?;

        Ok(())
    }

    pub fn upgrade(&self, chart_repo: &String, chart_name: &String) -> anyhow::Result<()> {
        let helm_binary = which("helm")?;

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

        Command::new(&helm_binary)
            .arg("upgrade")
            .arg("--install")
            .arg(&chart_name)
            .arg(format!("{}/{}", &chart_repo, &chart_name))
            .arg("-f")
            .arg(config_file.path())
            .arg("--wait")
            .spawn()?
            .wait()?;

        Ok(())
    }
}
