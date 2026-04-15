job "api" {
  group "worker" {
    task "bootstrap" {
      template {
        destination = "local/bootstrap.py"
        data = <<-EOF
import os

def render_message():
    print("hello ${NOMAD_ALLOC_ID}")
%{ if meta.env == "prod" }
    return os.environ.get("MODE", "prod")
%{ else }
    return "dev"
%{ endif }
EOF
      }
    }
  }
}
