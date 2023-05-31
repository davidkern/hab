job "hab-ve-mk3" {
  region      = "hab"
  datacenters = ["hab"]

  type = "service"

  group "app" {
    count = 1

    task "hab-ve-mk3" {
      driver = "docker"

      config {
        image = "registry.hab.mju.io/hab-ve-mk3:0.1.0-build4"
        devices = [
          {
            host_path = "/dev/serial/by-id/usb-VictronEnergy_MK3-USB_Interface_HQ19125YEZ6-if00-port0"
            container_path = "/dev/ve-multiplus"
          }
        ]
      }

      template {
        destination = "${NOMAD_SECRETS_DIR}/env.txt"
        env         = true
        data        = <<EOT
          {{ with nomadVar "nomad/jobs/hab-ve-mk3"}}
          RUST_BACKTRACE=1
          RUST_LOG=debug
          MK3_PATH="/dev/ve-multiplus"
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
