job "unifi" {
    region = "hab"
    datacenters = ["hab"]

    type = "service"

    group "unifi" {
        network {
            mode = "host"
            port "web_admin" {
                static = 8443
            }
            port "stun" {
                static = 3478
            }
            port "discovery" {
                static = 10001
            }
            port "device_comms" {
                static = 8080
            }
        }

        volume "unifi_volume" {
            type = "host"
            source = "unifi"
        }

        task "controller" {
            driver = "docker"

            config {
                image = "linuxserver/unifi-controller:7.4.162"
                ports = [
                    "web_admin",
                    "stun",
                    "discovery",
                    "device_comms"
                ]
            }

            volume_mount {
                volume = "unifi_volume"
                destination = "/config"
            }

            resources {
                cpu = 1024
                memory = 1024
            }

            env {
                PUID = 1000
                PGID = 1000
                TZ = "America/Los_Angeles"
            }
        }
    }
}

