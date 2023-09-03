job "terraform-agent" {
  region      = "hab"
  datacenters = ["hab"]

  type = "service"

  group "app" {
    count = 1

    task "agent" {
      driver = "docker"

      config {
        image = "hashicorp/tfc-agent:latest"
      }

      template {
        destination = "${NOMAD_SECRETS_DIR}/env.txt"
        env         = true
        data        = <<EOT
          {{ with nomadVar "nomad/jobs/terraform-agent"}}
          TFC_AGENT_NAME="hab"
          TFC_AGENT_TOKEN="{{ .TFC_AGENT_TOKEN }}"
          {{ end }}
        EOT
      }

      env {
      }
    }
  }
}
