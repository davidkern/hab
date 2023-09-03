terraform {
  cloud {
    organization = "hab"

    workspaces {
        name = "01-network"
    }
  }

  required_providers {
    routeros = {
      source = "terraform-routeros/routeros"
      version = "1.13.2"
    }
  }
}

provider "routeros" {
  alias = "hab"
  hosturl = "https://10.42.0.1"
  username = "${var.routeros_hab_username}"
  password = "${var.routeros_hab_password}"
  insecure = true
}

provider "routeros" {
  alias = "scif"
  hosturl = "https://10.42.1.1"
  username = "${var.routeros_scif_username}"
  password = "${var.routeros_scif_password}"
  insecure = true
}
