---
title: "Sample Security Policy"
version: "1.0.0"
author: "Policy Team"
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

This section covers data protection and encryption requirements.

- All sensitive data must be classified according to the organizational data classification scheme
- Data at rest must be encrypted using AES-256 or equivalent
- Data in transit must be protected using TLS 1.2 or higher
- Encryption keys must be rotated annually and stored in a hardware security module

## Data Retention

- Audit logs must be retained for a minimum of one year

# Incident Response

This section establishes incident response procedures and requirements.

- All security incidents must be reported within 24 hours of detection
- Systems must log all authentication attempts and must log all privilege escalation events
- An incident response plan must be tested at least annually through tabletop exercises
