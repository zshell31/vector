---
title: Nix
weight: 6
---

[Nix] is a cross-platform package manager implemented on a functional deployment model where software is installed into unique directories generated through cryptographic hashes, it is also the name of the programming language. This page will cover installing and managing Vector through the Nix package repository.

{{< warning title="Nix releases are delayed" >}}
Because Nix releases for Vector must be manually updated, expect delays between official Vector releases and release of the Nix package. New Vector packages for Nix are typically available within a few days.
{{< /warning >}}

## Installation

```shell
nix-env --install \
  --file https://github.com/NixOS/nixpkgs/archive/master.tar.gz \
  --attr vector
```

## Deployment

Vector is an end-to-end observability data pipeline designed to deploy under various roles. You mix and match these roles to create topologies. The intent is to make Vector as flexible as possible, allowing you to fluidly integrate Vector into your infrastructure over time. The deployment section demonstrates common Vector pipelines:

{{< jump "/docs/setup/deployment/topologies" >}}

## Administration

### Start

```shell
vector --config /etc/vector/vector.toml
```

### Reload

```shell
killall -s SIGHUP vector
```

### Upgrade

```shell
nix-env --upgrade vector \
  --file https://github.com/NixOS/nixpkgs/archive/master.tar.gz
```

### Uninstall

```shell
nix-env --uninstall vector
```

[nix]: https://nixos.org