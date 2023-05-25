job "container_registry" {
  region = "hab"
  datacenters = [ "hab" ]
  type = "service"

  group "app" {
    count = 1

    network {
      port "registry" {
        static = 5000
        to = 5000
      }
    }

    service {
      name = "container-registry"
      port = "registry"

      check {
        name     = "alive"
        type     = "http"
        path     = "/"
        interval = "10s"
        timeout  = "2s"
      }
    }

    restart {
      attempts = 2
      interval = "30m"
      delay    = "15s"
      mode     = "fail"
    }

    task "server" {
      driver = "docker"

      config {
        image = "registry"
        ports = ["registry"]

        mount {
          type = "volume"
          target = "/var/lib/registry"
          source = "container-registry"
        }
      }
    }
  }
}
