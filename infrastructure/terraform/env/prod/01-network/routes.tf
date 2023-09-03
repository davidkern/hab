resource "routeros_ip_route" "scif_default" {
  dst_address = "0.0.0.0/0"
  gateway     = "10.42.0.1"
}
