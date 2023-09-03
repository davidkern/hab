# Full configuration options can be found at https://www.nomadproject.io/docs/configuration

datacenter = "hab"
region = "hab"
data_dir   = "/opt/nomad/data"
bind_addr  = "0.0.0.0"

server {
  enabled          = true
  bootstrap_expect = 1
}

client {
  enabled = true
  servers = ["127.0.0.1"]

  host_volume "unifi" {
    path = "/volumes/unifi"
  }
}

ui {
  enabled =  true

  label {
    text             = "Hab"
    background_color = "white"
    text_color       = "#000000"
  }
}

plugin "docker" {
  config {
    gc {
      image       = true
      image_delay = "3m"
      container   = true

      dangling_containers {
        enabled        = true
        dry_run        = false
        period         = "5m"
        creation_grace = "5m"
      }
    }
    volumes {
      enabled      = true
      selinuxlabel = "z"
    }
  }
}
