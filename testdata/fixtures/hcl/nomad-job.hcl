# Nomad-style HCL fixture used to lock top-level detection and highlighting.
job "api" {
  datacenters = ["dc1"]
  type        = "service"

  group "web" {
    count = 2

    network {
      port "http" {
        static = 8080
        to     = 8080
      }
    }

    task "server" {
      driver = "docker"

      config {
        image   = "ghcr.io/acme/api:1.2.3"
        command = "/bin/api"
        args    = ["-config", "${NOMAD_TASK_DIR}/config.hcl"]
      }

      env {
        ENABLE_METRICS = true
        LOG_LEVEL      = "debug"
      }

      service {
        name = "api"
        port = "http"
        tags = ["edge", "v1"]
      }

      template {
        destination = "local/config.hcl"
        data = <<-EOF
port = 8080
enabled = true
node_name = "${node.unique.name}"
%{ if meta.env == "prod" }
mode = "strict"
%{ else }
mode = "dev"
%{ endif }
EOF
      }
    }
  }
}
