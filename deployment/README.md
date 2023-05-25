# Hab infrastructure

## First time setup

Install nomad-pack nightly build:
  * [amd64 .deb](https://github.com/hashicorp/nomad-pack/releases/download/nightly/nomad-pack_0.0.1.techpreview.4-1_amd64.deb)
  * [arm64 .deb](https://github.com/hashicorp/nomad-pack/releases/download/nightly/nomad-pack_0.0.1.techpreview.4-1_arm64.deb)

## Usage

* `export NOMAD_ADDR=http://habctl.hab.mju.io:4646`
* `export NOMAD_REGION=hab`
* `nomad-pack run ./packs/[pack-name]`
