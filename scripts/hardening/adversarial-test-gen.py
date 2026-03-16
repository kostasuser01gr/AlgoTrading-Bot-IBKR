#!/usr/bin/env python3
import json

payloads = {
    "injection": ["'; DROP TABLE users; --", "${SHELL}", "%00", "../../etc/passwd"],
    "type_confusion": [None, 0, "", [], {}, True],
    "resource_exhaustion": ["A" * 100000],
}

print(json.dumps(payloads, indent=2))
