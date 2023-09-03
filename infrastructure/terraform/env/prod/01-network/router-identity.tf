resource "routeros_system_identity" "hab" {
  provider = routeros.hab
  name = "hab"
}

resource "routeros_system_identity" "scif" {
  provider = routeros.scif
  name = "scif"
}
