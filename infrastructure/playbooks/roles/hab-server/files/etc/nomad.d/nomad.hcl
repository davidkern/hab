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
}

ui {
  enabled =  true

  consul {
    ui_url = "https://habctl.hab.mju.io:8500/ui"
  }

  vault {
    ui_url = "http://habctl.hab.mju.io:8200/ui"
  }

  label {
    text             = "Hab"
    background_color = "white"
    text_color       = "#000000"
  }
}
