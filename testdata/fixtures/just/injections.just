default:
    source .env
    echo "shell default"

python-check:
    #!/usr/bin/env python3
    import json

    payload = {"tool": "kat", "enabled": True}

    class Preview:
        def render(self) -> str:
            return json.dumps(payload)

    print(Preview().render())

bun-report:
    #!/usr/bin/env bun
    const report = { tool: "kat", ready: true };
    console.log(JSON.stringify(report));
