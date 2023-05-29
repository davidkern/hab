# Hab infrastructure

## First time setup

* Install just.
* [Install poetry](https://python-poetry.org/docs/#installation).
* Run `poetry install` in this directory.
* [Install bitwarden cli](https://vault.bitwarden.com/download/?app=cli&platform=linux)
  extract the bw binary to a location on your path, such as ~/.local/bin.

## Usage

The ansible vault password is stored in bitwarden. Before running other commands,
run `just login` to retrieve the ansible vault password (stored to .ansible-vault-password).

To plan (check) changes: `just plan`.
To apply changes: `just apply`.

## Notes

This stack assumes that:
* the Hab computer has Ubuntu 22.04 Server already installed;
* a `hab` user exists with passwordless-sudo rights;
* that the ansible-playbook running machine can ssh to the hab account without a password
  (e.g. it has ssh keys setup).
