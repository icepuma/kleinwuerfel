use serde_derive::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct HelmChartRepo {
    pub name: String,
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,

    pub values: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Helmchart {
    pub helm_chart_repo: String,
    pub name: String,

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
    pub helm_chart_repo: Option<Vec<HelmChartRepo>>,
    pub helmchart: Option<Vec<Helmchart>>,
}
