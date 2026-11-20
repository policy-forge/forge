#!/usr/bin/env python3
"""Generate a representative SSP JSON from the component-definition output."""
import json
import uuid
from datetime import datetime, timezone

# Load the component definition to extract control IDs
with open("output/component-definition.json") as f:
    comp_def = json.load(f)

cd = comp_def["component-definition"]

# Collect control IDs from component definition, remembering which component
# supplied each one so `links[].href` can point at that component's real SSP
# UUID (fragments must resolve to a component uuid, not a slug).
# The CD has one documentary policy component owning all controls, so the
# owner signal is each IR's description, not the component title.
def ssp_seed_for_requirement(ir):
    if "database component" in ir.get("description", "").lower():
        return "database"
    return "web-application"


control_ids = []
owner_seed = {}
for comp in cd.get("components", []):
    for ci in comp.get("control-implementations", []):
        for ir in ci.get("implemented-requirements", []):
            cid = ir["control-id"]
            # One IR per control: duplicate uuids would be invalid OSCAL.
            # First declaration wins; the example CD has a single owner.
            if cid in owner_seed:
                continue
            control_ids.append(cid)
            owner_seed[cid] = ssp_seed_for_requirement(ir)

# Generate deterministic UUIDs for SSP components
def stable_uuid(seed):
    return str(uuid.uuid5(uuid.NAMESPACE_DNS, f"forge-ssp-{seed}"))

# Build implemented requirements list
implemented_reqs = []
for cid in control_ids:
    implemented_reqs.append({
        "uuid": stable_uuid(f"ir-{cid.lower()}"),
        "control-id": cid,
        # OSCAL implemented-requirement has no description field; narrative
        # belongs in by-components[].description tied to a component uuid.
        "by-components": [{
            "uuid": stable_uuid(f"bc-{cid.lower()}"),
            "component-uuid": stable_uuid(owner_seed[cid]),
            "description": f"Implementation statement for {cid} - see component definition for details."
        }]
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
            "oscal-version": "1.2.3",
            "parties": [
                {"uuid": stable_uuid("party-app-eng"), "type": "organization", "name": "Application Engineering Team"},
                {"uuid": stable_uuid("party-sec-eng"), "type": "organization", "name": "Security Engineering Team"},
                {"uuid": stable_uuid("party-db-ops"), "type": "organization", "name": "Database Operations Team"},
                {"uuid": stable_uuid("party-aws"), "type": "organization", "name": "Amazon Web Services"}
            ]
        },
        "import-profile": {"href": "profile.json"},
        "system-characteristics": {
            "system-ids": [{
                "identifier-type": "http://example.org/system-id",
                "id": "system-csp-001"
            }],
            "system-name": "Component-Based Security Policy System",
            "description": "Representative system built from the component definition.",
            "system-information": {
                "information-types": [{
                    "title": "Security Policy Data",
                    "description": "Policy requirements and implementation evidence."
                }]
            },
            "status": {"state": "operational"},
            "authorization-boundary": {
                "description": "The web application and database components."
            }
        },
        "system-implementation": {
            "components": [
                {
                    "uuid": stable_uuid("web-application"),
                    "type": "software",
                    "title": "Web Application Component",
                    "description": "The primary web application handling user authentication, authorization, and API requests. Implements TLS 1.3, MFA, RBAC, and audit logging.",
                    "responsible-roles": [
                        {"role-id": "system-admin", "party-uuids": [stable_uuid("party-app-eng")]},
                        {"role-id": "security-officer", "party-uuids": [stable_uuid("party-sec-eng")]}
                    ],
                    "status": {"state": "operational"}
                },
                {
                    "uuid": stable_uuid("database"),
                    "type": "software",
                    "title": "Database Component",
                    "description": "PostgreSQL database server with AES-256 encryption at rest, automated backups, and direct access audit logging. Communicates with the application component over mTLS.",
                    "responsible-roles": [
                        {"role-id": "database-admin", "party-uuids": [stable_uuid("party-db-ops")]},
                        {"role-id": "security-officer", "party-uuids": [stable_uuid("party-sec-eng")]}
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
                    "props": [
                        {
                            "name": "authorized-date",
                            "ns": "https://forge.policy-forge.github.io/ns/ssp-template",
                            "value": "2026-01-01T00:00:00Z"
                        },
                        {
                            "name": "account-status",
                            "ns": "https://forge.policy-forge.github.io/ns/ssp-template",
                            "value": "active"
                        }
                    ]
                },
                {
                    "uuid": stable_uuid("user-analyst"),
                    "title": "Security Analyst",
                    "short-name": "analyst-01",
                    "description": "Read-only access to security logs and audit reports",
                    "role-ids": ["analyst"],
                    "props": [
                        {
                            "name": "authorized-date",
                            "ns": "https://forge.policy-forge.github.io/ns/ssp-template",
                            "value": "2026-01-01T00:00:00Z"
                        },
                        {
                            "name": "account-status",
                            "ns": "https://forge.policy-forge.github.io/ns/ssp-template",
                            "value": "active"
                        }
                    ]
                },
                {
                    "uuid": stable_uuid("user-svc-ingest"),
                    "title": "Service Account",
                    "short-name": "svc-ingest",
                    "description": "Automated service account for data ingestion pipeline",
                    "role-ids": ["service-account"],
                    "props": [
                        {
                            "name": "authorized-date",
                            "ns": "https://forge.policy-forge.github.io/ns/ssp-template",
                            "value": "2026-01-01T00:00:00Z"
                        },
                        {
                            "name": "account-status",
                            "ns": "https://forge.policy-forge.github.io/ns/ssp-template",
                            "value": "active"
                        },
                        {
                            "name": "account-status-reason",
                            "ns": "https://forge.policy-forge.github.io/ns/ssp-template",
                            "value": "automated system process, no interactive login"
                        }
                    ]
                }
            ],
            "leveraged-authorizations": [
                {
                    "uuid": stable_uuid("leveraged-aws"),
                    "title": "AWS GovCloud Infrastructure",
                    "party-uuid": stable_uuid("party-aws"),
                    "date-authorized": "2026-01-01",
                    "links": [{"href": "https://aws.amazon.com/compliance/fedramp/", "rel": "reference"}],
                    "remarks": "Physical security, hypervisor, and infrastructure controls inherited from AWS GovCloud (FedRAMP Moderate)"
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

# Every by-component reference must resolve to a declared SSP component.
declared = {c["uuid"] for c in ssp["system-security-plan"]["system-implementation"]["components"]}
for ir in implemented_reqs:
    for bc in ir["by-components"]:
        assert bc["component-uuid"] in declared, f"dangling component uuid for {ir['control-id']}"

with open("output/ssp.json", "w") as f:
    json.dump(ssp, f, indent=2)

print(f"SSP generated with {len(control_ids)} implemented requirements")
print(f"Components: web-application, database")
print(f"Leveraged authorization: AWS GovCloud")
