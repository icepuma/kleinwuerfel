use serde_derive::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ContainerRegistry {
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HelmChartRepo {
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Helmchart {
    pub name: String,
    pub container_registry: String,
    pub helm_chart_repo: String,
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
    pub container_registry: Option<Vec<ContainerRegistry>>,
    pub helm_chart_repo: Option<Vec<HelmChartRepo>>,
    pub helmchart: Option<Vec<Helmchart>>,
}
