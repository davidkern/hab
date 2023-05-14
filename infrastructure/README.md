# Hab infrastructure

## First time setup

* [Install poetry](https://python-poetry.org/docs/#installation).
* Run `poetry install` in this directory.

## Usage

To plan (check) changes: `just plan`.
To apply changes: `just apply`.

## Notes

This stack assumes that:
* the Hab computer has Ubuntu 22.04 Server already installed;
* a `hab` user exists with passwordless-sudo rights;
* that the ansible-playbook running machine can ssh to the hab account without a password
  (e.g. it has ssh keys setup).
