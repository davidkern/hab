terraform {
  cloud {
    organization = "hab"

    workspaces {
        name = "00-auth"
    }
  }
}
