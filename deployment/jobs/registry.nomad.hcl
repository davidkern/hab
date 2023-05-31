job "registry" {
  region      = "hab"
  datacenters = ["hab"]

  type = "service"

  group "web" {
    count = 1

    network {
      port "http" {
        to = 5000
      }
    }

    service {
      provider = "nomad"
      name     = "registry"
      port     = "http"
      tags = [
        "traefik.enable=true",
        "traefik.http.routers.registry.rule=Host(`registry.hab.mju.io`)",
        "traefik.http.routers.registry.tls=true",
        "traefik.http.routers.registry.tls.certresolver=letsencrypt",
      ]

      check {
        name     = "alive"
        type     = "http"
        path     = "/"
        interval = "10s"
        timeout  = "2s"
      }
    }

    task "server" {
      driver = "docker"

      config {
        image = "registry"
        ports = ["http"]

        mount {
          type   = "volume"
          target = "/var/lib/registry"
          source = "registry"
        }
      }
    }
  }
}
