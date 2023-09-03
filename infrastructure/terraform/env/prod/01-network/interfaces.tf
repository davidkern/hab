resource "routeros_interface_bridge_port" "scif_david" {
    provider = routeros.scif
    bridge = "bridge"
    interface = "ether1"
    pvid = 100
    comment = "David's scif desk"
}

resource "routeros_interface_bridge_port" "scif_unifi_ap" {
    provider = routeros.scif
    bridge = "bridge"
    interface = "ether9"
    pvid = 100
    comment = "Unifi AP LR"
}

resource "routeros_interface_bridge_port" "scif_hab_uplink" {
    provider = routeros.scif
    bridge = "bridge"
    interface = "ether10"
    pvid = 100
    comment = "Uplink to hab via 10G switch"
}
