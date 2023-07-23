job "api" {
    region = "hab"
    datacenters = ["hab"]

    type = "service"

    group "web" {
        network {
            port "http" {
                to = 9999
            }
        }

        service {
            provider = "nomad"
            name = "api"
            port = "http"
            tags = [
                "traefik.enable=true",
                "traefik.http.routers.api.rule=Host(`api.hab.mju.io`)",
                "traefik.http.routers.api.tls=true",
                "traefik.http.routers.api.tls.certresolver=letsencrypt",
            ]
            check {
                type     = "tcp"
                interval = "10s"
                timeout  = "2s"
            }
        }

        task "api" {
            driver = "docker"

            config {
                image = "registry.hab.mju.io/hab-api:0.1.0-build1"
                ports = ["http"]
            }

            template {
                destination = "${NOMAD_SECRETS_DIR}/env.txt"
                env         = true
                data        = <<EOT
                {{ with nomadVar "nomad/jobs/hab-ve-mk3"}}
                RUST_BACKTRACE=1
                RUST_LOG=debug
                BIND_ADDRESS=0.0.0.0:9999
                INFLUXDB_URL="{{ .INFLUXDB_URL }}"
                INFLUXDB_ORG="{{ .INFLUXDB_ORG }}"
                INFLUXDB_TOKEN="{{ .INFLUXDB_TOKEN }}"
                {{ end }}
                EOT
            }

            env {
            }
        }
    }
}

