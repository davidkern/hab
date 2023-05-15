# Full configuration options can be found at https://www.consul.io/docs/agent/config

datacenter = "hab"
data_dir = "/opt/consul"
client_addr = "[::]"
client_addr = "0.0.0.0"
ui_config{
  enabled = true
}
server = true
bind_addr = "[::]"
bind_addr = "0.0.0.0"
bootstrap_expect=1
