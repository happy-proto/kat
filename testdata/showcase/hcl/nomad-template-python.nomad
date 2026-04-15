# Nomad template showcasing target-runtime dispatch plus HCL template syntax.
job "api" {
  datacenters = ["dc1"]
  type        = "service"

  group "worker" {
    task "bootstrap" {
      driver = "docker"

      config {
        image = "python:3.13-alpine"
      }

      template {
        destination = "local/bootstrap.py"
        data = <<-EOF
import os
from pathlib import Path

def render_message(user: str) -> str:
    home = Path("${NOMAD_TASK_DIR}")
    print(f"hello {user} from ${node.unique.name}")
%{ if meta.env == "prod" }
    return os.environ.get("MODE", "prod")
%{ else }
    return f"dev::{user}"
%{ endif }
EOF
      }
    }
  }
}
