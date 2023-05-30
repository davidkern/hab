# Hab infrastructure

## First time setup

Install nomad-pack nightly build

### Debian/Ubuntu

  * [amd64 .deb](https://github.com/hashicorp/nomad-pack/releases/download/nightly/nomad-pack_0.0.1.techpreview.4-1_amd64.deb)
  * [arm64 .deb](https://github.com/hashicorp/nomad-pack/releases/download/nightly/nomad-pack_0.0.1.techpreview.4-1_arm64.deb)

### Fedora

  * [amd64 .rpm](https://github.com/hashicorp/nomad-pack/releases/download/nightly/nomad-pack-0.0.1.techpreview.4-1.x86_64.rpm)
  * [arm64 .rpm](https://github.com/hashicorp/nomad-pack/releases/download/nightly/nomad-pack-0.0.1.techpreview.4-1.aarch64.rpm)

## Usage

* `export NOMAD_ADDR=http://habctl.hab.mju.io:4646`
* `export NOMAD_REGION=hab`
* `nomad-pack run ./packs/[pack-name]`
