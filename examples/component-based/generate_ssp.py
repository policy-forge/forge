#!/usr/bin/env python3
"""Generate a representative SSP JSON from the component-definition output."""
import json
import uuid
from datetime import datetime, timezone

# Load the component definition to extract control IDs
with open("output/component-definition.json") as f:
    comp_def = json.load(f)

cd = comp_def["component-definition"]

# Collect control IDs from component definition
control_ids = []
for comp in cd.get("components", []):
    for ci in comp.get("control-implementations", []):
        for ir in ci.get("implemented-requirements", []):
            control_ids.append(ir["control-id"])

# Generate deterministic UUIDs for SSP components
def stable_uuid(seed):
    return str(uuid.uuid5(uuid.NAMESPACE_DNS, f"forge-ssp-{seed}"))

# Build implemented requirements list
implemented_reqs = []
for cid in control_ids:
    implemented_reqs.append({
        "uuid": stable_uuid(f"ir-{cid.lower()}"),
        "control-id": cid,
        "description": f"Implementation statement for {cid} - see component definition for details.",
        "links": [
            {"href": "#component-web-application", "rel": "implements"}
        ]
    })

# Build the SSP
now = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%f")[:-3] + "+00:00"

ssp = {
    "system-security-plan": {
        "uuid": stable_uuid("system-security-plan"),
        "metadata": {
            "title": "System Security Plan for Component-Based Security Policy",
            "last-modified": now,
            "version": "1.0.0",
            "oscal-version": "1.1.3"
        },
        "system-id": {
            "identifier-type": "http://example.org/system-id",
            "id": "system-csp-001"
        },
        "system-implementation": {
            "components": [
                {
                    "uuid": stable_uuid("web-application"),
                    "type": "software",
                    "title": "Web Application Component",
                    "description": "The primary web application handling user authentication, authorization, and API requests. Implements TLS 1.3, MFA, RBAC, and audit logging.",
                    "responsible-roles": [
                        {"role-id": "system-admin", "responsible-party": "Application Engineering Team"},
                        {"role-id": "security-officer", "responsible-party": "Security Engineering Team"}
                    ],
                    "status": {"state": "operational"}
                },
                {
                    "uuid": stable_uuid("database"),
                    "type": "software",
                    "title": "Database Component",
                    "description": "PostgreSQL database server with AES-256 encryption at rest, automated backups, and direct access audit logging. Communicates with the application component over mTLS.",
                    "responsible-roles": [
                        {"role-id": "database-admin", "responsible-party": "Database Operations Team"},
                        {"role-id": "security-officer", "responsible-party": "Security Engineering Team"}
                    ],
                    "status": {"state": "operational"}
                }
            ],
            "users": [
                {
                    "uuid": stable_uuid("user-system-admin"),
                    "title": "System Administrator",
                    "short-name": "sysadmin",
                    "description": "Full system administration access with privileged role assignment capabilities",
                    "role-ids": ["system-admin", "security-officer"],
                    "authorized-date": "2026-01-01T00:00:00Z",
                    "status": {"state": "active"}
                },
                {
                    "uuid": stable_uuid("user-analyst"),
                    "title": "Security Analyst",
                    "short-name": "analyst-01",
                    "description": "Read-only access to security logs and audit reports",
                    "role-ids": ["analyst"],
                    "authorized-date": "2026-01-01T00:00:00Z",
                    "status": {"state": "active"}
                },
                {
                    "uuid": stable_uuid("user-svc-ingest"),
                    "title": "Service Account",
                    "short-name": "svc-ingest",
                    "description": "Automated service account for data ingestion pipeline",
                    "role-ids": ["service-account"],
                    "authorized-date": "2026-01-01T00:00:00Z",
                    "status": {"state": "active", "reason": "automated system process, no interactive login"}
                }
            ],
            "leveraged-authorizations": [
                {
                    "uuid": stable_uuid("leveraged-aws"),
                    "title": "AWS GovCloud Infrastructure",
                    "href": "https://aws.amazon.com/compliance/fedramp/",
                    "description": "Physical security, hypervisor, and infrastructure controls inherited from AWS GovCloud (FedRAMP Moderate)"
                }
            ]
        },
        "control-implementation": {
            "description": "Control implementations for the Component-Based Security Policy, mapped from policy requirements to system components.",
            "implemented-requirements": implemented_reqs
        },
        "back-matter": {
            "resources": [
                {
                    "uuid": stable_uuid("resource-policy"),
                    "title": "Component-Based Security Policy",
                    "description": "Source policy document defining security controls for the two-tier architecture",
                    "rlinks": [{"href": "../policy.md"}]
                },
                {
                    "uuid": stable_uuid("resource-component-def"),
                    "title": "Component Definition",
                    "description": "OSCAL Component Definition mapping policy controls to system components",
                    "rlinks": [{"href": "./component-definition.json"}]
                }
            ]
        }
    }
}

with open("output/ssp.json", "w") as f:
    json.dump(ssp, f, indent=2)

print(f"SSP generated with {len(control_ids)} implemented requirements")
print(f"Components: web-application, database")
print(f"Leveraged authorization: AWS GovCloud")
