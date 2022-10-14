use serde_derive::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Registry {
    pub name: String,
    pub url: String,
    pub helm_repo_url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Helmchart {
    pub name: String,
    pub registry: String,
    pub repo: String,

    #[serde(default)]
    pub ports: Vec<u16>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Minikube {
    pub cpus: Option<u8>,
    pub memory: Option<u16>,

    #[serde(default)]
    pub addons: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Configuration {
    pub minikube: Option<Minikube>,
    pub registry: Option<Vec<Registry>>,
    pub helmchart: Option<Vec<Helmchart>>,
}
