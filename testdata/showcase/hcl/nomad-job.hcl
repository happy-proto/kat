# Rich Nomad-style showcase for HCL rendering.
job "payments" {
  datacenters = ["dc1", "dc2"]
  namespace   = "platform"
  type        = "service"

  constraint {
    attribute = "${node.class}"
    operator  = "="
    value     = "edge"
  }

  group "api" {
    count = 3

    network {
      mode = "bridge"

      port "http" {
        static = 9000
        to     = 8080
      }
    }

    service {
      name = "payments-api"
      port = "http"
      tags = ["public", "v2"]

      check {
        type     = "http"
        path     = "/health"
        interval = "10s"
        timeout  = "2s"
      }
    }

    task "server" {
      driver = "docker"

      config {
        image      = "ghcr.io/acme/payments:2.4.0"
        force_pull = false
        command    = "/bin/payments"
        args = [
          "-config",
          "${NOMAD_TASK_DIR}/config.hcl",
          "-http",
          "${NOMAD_ADDR_http}",
        ]
      }

      env {
        LOG_FORMAT     = "json"
        ENABLE_METRICS = true
        SAMPLE_RATE    = 0.5
      }

      resources {
        cpu    = 750
        memory = 512
      }

      template {
        destination = "local/runtime.hcl"
        data = <<-EOF
bind = "${NOMAD_ADDR_http}"
service_name = "payments"
meta = {
  node = "${node.unique.name}"
}
%{ if meta.env == "prod" }
log_level = "warn"
%{ else }
log_level = "debug"
%{ endif }
EOF
      }
    }
  }
}
