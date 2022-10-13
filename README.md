<div align="center">

# kleinwuerfel

![https://crates.io/crates/kleinwuerfel](https://img.shields.io/crates/v/kleinwuerfel)
![https://github.com/icepuma/kleinwuerfel/actions/workflows/ci.yaml](https://github.com/icepuma/kleinwuerfel/actions/workflows/ci.yaml/badge.svg)

Interact with track.toggl.com via terminal.

[Installation](#installation) •
[Usage](#usage)

</div>

## Installation
* cargo
  ```bash
  cargo install kleinwuerfel
  ```
* Precompiled binary

## Usage

### Config file
```toml
[minikube]
cpus = 4
memory = 8192

[[registry]]
name = "registry-1"
url = "some.registry.url"
helm_repo_url = "some.registry.url/chartrepo"
username = "${env.HARBOR_USERNAME}"
password = "${env.HARBOR_SECRET}"

[[helmchart]]
name = "helm-chart-1"
registry = "registry-1"
repo = "chart-repo"

[[helmchart]]
name = "helm-chart-2"
registry = "registry-1"
repo = "chart-repo"

[[helmchart]]
name = "helm-chart-3"
registry = "registry-1"
repo = "some-different-chart-repo"
```

### Start (start minikube and deploy helm charts)
```bash
kleinwuerfel start
```

### Stop (stops minikube)
```bash
kleinwuerfel stop
```
