python-check:
    #!/usr/bin/env python3
    import json
    from pathlib import Path

    payload = {"tool": "kat", "enabled": True}
    print(json.dumps(payload, indent=2))

node-report:
    #!/usr/bin/env node
    const report = { name: "kat", status: "preview" };
    console.log(JSON.stringify(report, null, 2));
