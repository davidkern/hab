resource "routeros_dns" "scif_dns" {
  provider = routeros.scif
  allow_remote_requests = true
  servers               = "10.42.0.1"
}
