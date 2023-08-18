job "hab-ve-direct-ext" {
  region      = "hab"
  datacenters = ["hab"]

  type = "service"

  group "app" {
    count = 1

    task "hab-ve-direct-ext" {
      driver = "docker"

      config {
        image = "registry.hab.mju.io/hab-ve-direct:0.2.2-build1"
        devices = [
          {
            host_path = "/dev/serial/by-id/usb-VictronEnergy_BV_VE_Direct_cable_VE683RVF-if00-port0"
            container_path = "/dev/ve-direct"
          }
        ]
      }

      template {
        destination = "${NOMAD_SECRETS_DIR}/env.txt"
        env         = true
        data        = <<EOT
          {{ with nomadVar "nomad/jobs/hab-ve-mk3"}}
          RUST_BACKTRACE=1
          RUST_LOG=trace
          DEVICE_NAME="mppt_ext"
          VE_DIRECT_PATH="/dev/ve-direct"
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
