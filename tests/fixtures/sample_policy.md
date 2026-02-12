---
title: "Sample Security Policy"
version: "1.0.0"
author: "Policy Team"
date: "2026-01-15"
---

# Access Control

All users must authenticate before accessing systems.

## Authentication Requirements

- Users must use multi-factor authentication
- Passwords must be at least 12 characters
- Sessions must timeout after 30 minutes of inactivity

## Authorization

- Access must follow principle of least privilege
- Role-based access control must be enforced

# Data Protection

## Encryption

- Data at rest must be encrypted using AES-256
- Data in transit must use TLS 1.2 or higher
- Encryption keys must be rotated annually
