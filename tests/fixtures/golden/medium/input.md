---
title: "Enterprise Security Policy"
version: "2.0.0"
author: "Information Security Office"
date: "2026-01-15"
---

# Access Control

This section defines access control requirements for all organizational systems.

- All users must authenticate using approved credentials before accessing any system
- Multi-factor authentication must be enabled for all privileged accounts
- Access permissions must follow the principle of least privilege
- User accounts must be reviewed quarterly and inactive accounts disabled after 90 days

## Authentication Standards

- Passwords must be at least 12 characters and include uppercase, lowercase, numbers, and symbols

# Data Protection

This section covers data protection and encryption requirements per NIST SP 800-53.

- All sensitive data must be classified according to the organizational data classification scheme
- Data at rest must be encrypted using AES-256 or equivalent
- Data in transit must be protected using TLS 1.2 or higher
- Encryption keys must be rotated annually and stored in a hardware security module

## Data Retention

- Audit logs must be retained for a minimum of one year
- Backup copies must be encrypted and stored in a geographically separate location

# Incident Response

This section establishes incident response procedures and requirements per NIST SP 800-61.

- All security incidents must be reported within 24 hours of detection
- An incident response plan must be tested at least annually through tabletop exercises
