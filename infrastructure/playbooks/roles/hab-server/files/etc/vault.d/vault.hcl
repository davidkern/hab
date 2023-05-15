# Full configuration options can be found at https://www.vaultproject.io/docs/configuration

ui = true

cluster_addr  = "http://10.42.0.2:8201"
api_addr      = "http://10.42.0.2:8200"
disable_mlock = true

listener "tcp" {
  address            = "0.0.0.0:8200"
  tls_disable        = true
#  tls_cert_file      = "/opt/vault/tls/tls.crt"
#  tls_key_file       = "/opt/vault/tls/tls.key"
}

storage "raft" {
  path    = "/opt/vault/data"
  node_id = "hab-server"
}