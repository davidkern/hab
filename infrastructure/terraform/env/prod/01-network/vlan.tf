resource "routeros_interface_vlan" "scif_corp" {
  provider = routeros.scif
  interface = "bridge"
  name = "corp"
  vlan_id = 100
}

resource "routeros_interface_vlan" "scif_iot" {
  provider = routeros.scif
  interface = "bridge"
  name = "iot"
  vlan_id = 200
}

resource "routeros_interface_vlan" "scif_guest" {
  provider = routeros.scif
  interface = "bridge"
  name = "guest"
  vlan_id = 300
}

resource "routeros_interface_vlan" "hab_corp" {
  provider = routeros.hab
  interface = "bridge"
  name = "corp"
  vlan_id = 100
}

resource "routeros_interface_vlan" "hab_iot" {
  provider = routeros.hab
  interface = "bridge"
  name = "iot"
  vlan_id = 200
}

resource "routeros_interface_vlan" "hab_guest" {
  provider = routeros.hab
  interface = "bridge"
  name = "guest"
  vlan_id = 300
}
