<div align="center">

# kleinwuerfel

![https://crates.io/crates/kleinwuerfel](https://img.shields.io/crates/v/kleinwuerfel)
![https://github.com/icepuma/kleinwuerfel/actions/workflows/ci.yaml](https://github.com/icepuma/kleinwuerfel/actions/workflows/ci.yaml/badge.svg)

Opinionated command line tool to interact with [minikube](https://github.com/kubernetes/minikube). An easy way to deploy a given set of helm charts.

"kleinwuerfel" means more or less "minikube" in German.

[Installation](#installation) •
[Usage](#usage)

</div>

## Prerequisites
As `kleinwuerfel` interacts with other command line tools, you have install:
* `minikube`
* `helm`
* `kubectl`

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
# Amount of CPUs
cpus = 4

# Memory in MB
memory = 8192

[[helm_chart_repo]]
# Name to be referenced in [[helmchart]] blocks
name = "helm-chart-repo-1"

# URL for "helm repo add ..." and "helm login" when "username" and "password" are both set
url = "some.registry.url/chartrepo"

# Optional
username = "${env.HARBOR_USERNAME}"

# Optional
password = "${env.HARBOR_SECRET}"

# Optional - will be piped to "helm upgrade ... -f <values>"
values = """
imageRegistry:
  username: '${env.HARBOR_USERNAME}'
  password: '${env.HARBOR_SECRET}'
"""

[[helmchart]]
# Reference to name of [[helm_chart_repo]] block
helm_chart_repo = "helm-chart-repo-1"
# Is combined for "helm upgrade ... helm-chart-1 helm-chart-repo-1/helm-chart-1"
name = "helm-chart-1"
# Port fowarding
ports = [8080, 9999]

[[helmchart]]
# Reference to name of [[helm_chart_repo]] block
helm_chart_repo = "helm-chart-repo-1"
# Is combined for "helm upgrade ... helm-chart-1 helm-chart-repo-1/helm-chart-2"
name = "helm-chart-2"
```

### Up (start minikube and deploy helm charts)
* If `minikube status` exits with `0`, we assume that it is already running and skip the `minikube start` part

```bash
kleinwuerfel up
```

### Down
* Calls `minikube delete` right now

```bash
kleinwuerfel down
```

## Ideas
* Better error handling
* More testing
