job "dashboard" {
    region = "hab"
    datacenters = ["hab"]

    type = "service"

    group "web" {
        network {
            port "http" {
                to = 3000
            }
        }

        service {
            provider = "nomad"
            name = "dashboard"
            port = "http"
            tags = [
                "traefik.enable=true",
                "traefik.http.routers.dashboard.rule=Host(`dashboard.hab.mju.io`)",
                "traefik.http.routers.dashboard.tls=true",
                "traefik.http.routers.dashboard.tls.certresolver=letsencrypt",
            ]
            check {
                type     = "tcp"
                interval = "10s"
                timeout  = "2s"
            }
        }

        task "grafana" {
            driver = "docker"
            
            config {
                image = "grafana/grafana-oss:9.5.2"
                ports = ["http"]

                mount {
                    type = "volume"
                    target = "/var/lib/grafana"
                    source = "dashboard"
                }
            }    

            env {
                GF_LOG_LEVEL = "DEBUG"
                GF_LOG_MODE = "console"
                GF_AUTH_ANONYMOUS_ENABLED = "true"
                GF_AUTH_ANONYMOUS_ORG_NAME = "Hab"
                GF_AUTH_ANONYMOUS_ORG_ROLE = "Admin"
            }
        }
    }
}

