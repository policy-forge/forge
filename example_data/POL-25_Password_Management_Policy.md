# Password Management Policy

## Purpose

This policy defines the acceptable use of password management tools provided by the Organization and establishes requirements for the secure storage of credentials and other secret information.

## Scope

This policy applies to all staff working for the Organization, whether employed directly or indirectly. It covers the use of Organization-provided password management tools and the handling of credentials and secrets.

## Policy

### Password Manager Provision

**Tooling** — The Organization SHALL provide a password manager tool to all staff for the purposes of storing secret information, primarily for authentication to systems, and for other confidential information used as part of designated job duties.

### Acceptable Use

The following are acceptable uses of the Organization-provided password manager tool:

1. Storing company-related credentials personal to the employee, such as usernames and passwords used on test systems or domains
2. Storing company-related credentials for shared accounts, such as service accounts used by automated systems
3. Sharing non-personal credentials (e.g., service accounts) with authorized internal staff
4. Importing company-related passwords from other password repositories, such as browsers or other password managers
5. Storing company-related personal API keys, SSH keys, or encryption keys
6. Using the browser extension to auto-fill and log in to web applications and sites
7. Using the browser extension to create and save new credentials for web applications and sites

### Prohibited Activities

The following activities SHALL be blocked or prohibited:

**Sharing Restriction** — Sharing credentials or vault contents with external parties is prohibited.

**Export Restriction** — Exporting data from the password manager is prohibited.

**Financial Information** — Storing financial information in the password manager is prohibited unless specifically authorized for the employee's role.

**Personal Password Managers** — Employees SHALL NOT store company-related secrets, credentials, or other sensitive information in products or services not provided by the Organization. The use of personal password managers for company-related information is prohibited.

### Activities to Avoid

Employees SHOULD avoid using the password manager tool for the following purposes:

1. Storing personal financial or personally identifiable information
2. Storing personal, non-company-related credentials that may be needed after leaving the Organization

### Vault Access and Privacy

**Vault Privacy** — The Organization SHALL NOT access an employee's password vault during active employment.

**Vault Transfer** — After employment termination, the vault or specific records may be transferred to an authorized recipient for business or process continuity purposes. Vault transfer permissions SHALL be restricted to authorized senior members of the IT or Security team.

### Authentication

**Single Sign-On** — Access to password vaults SHALL be authenticated through the Organization's single sign-on (SSO) system and protected by multi-factor authentication (MFA).

### Employment Termination

**Access Revocation** — Access to the password manager tool SHALL be revoked upon termination of employment.

**Data Retention** — All data stored in the vault becomes inaccessible to the former employee after employment ends. Requests to make vault information available after termination SHALL be refused.

**Employee Guidance** — Employees SHALL be advised to store only company-related information in the Organization-provided password manager.

## Definitions

- **Password Manager**: A software tool that securely stores and manages credentials, secrets, and other sensitive authentication information
- **Vault**: An encrypted container within the password manager that stores an individual's credentials and secrets
- **Single Sign-On (SSO)**: An authentication method allowing users to access multiple systems with one set of credentials

## References

- ISO/IEC 27002: 5.17 Authentication information, 8.5 Secure authentication

