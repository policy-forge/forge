# Cloud Computing Security Policy

## Purpose

This policy defines the specific information security requirements for the establishment and use of third-party cloud services to store or process the Organization's information assets without jeopardizing data and computing resources.

## Scope

This policy applies to all personnel responsible for handling information assets and to all external cloud services, including cloud-based email, document storage, Software-as-a-Service (SaaS), Infrastructure-as-a-Service (IaaS), and Platform-as-a-Service (PaaS).

## Policy

### Cloud Approval and Governance

**Approval Required** — Use of cloud computing services for business purposes SHOULD be formally authorized by the Information Security Team. Employees SHOULD NOT open cloud service accounts or enter into cloud service contracts for storage, manipulation, or exchange of company data without approval.

**Vendor Validation** — All cloud vendors SHOULD be approved by the Information Technology Department. The Information Security Team SHOULD certify that security, privacy, and all other IT management requirements will be adequately addressed by the vendor.

**Control Compliance** — Additional control requirements adopted as part of cloud arrangements SHALL be formally adopted into the Organization's internal control framework.

### Establishing Cloud Services

**Terms of Service** — For any cloud services requiring users to agree to terms of service, such agreements SHALL be reviewed and approved by the legal department.

**Shared Security Responsibility Model** — Terms for cloud services SHALL ensure that the division of security responsibilities between the Organization and the provider is clearly defined.

### Terminating Cloud Services

**Termination Considerations** — When terminating a cloud service, the following SHALL be addressed: de-provisioning of access rights, migration or archiving of cloud logs, confidentiality and contractual obligations, data protection regulations, and data retention and deletion requirements.

### Access Controls

**Access Credentials** — Employees and contractors establishing login credentials at cloud services SHALL comply with existing security requirements for secure passwords. Cloud access SHOULD be provisioned through single sign-on (SSO) to avoid creating additional credentials.

**Password Separation** — Employees and contractors SHOULD NOT use the same passwords for cloud services as those for corporate accounts.

**Team-Based Access** — The preferred unit of rights assignment is a team or group, not an individual. Rights assignments to individuals SHOULD be avoided where possible and, where present, reviewed periodically.

**Privileged Access** — Specially privileged cloud security identities SHALL NOT have username-and-password access enabled except temporarily. Access control of such credentials SHALL only be allowed for teams specially designated for cloud service management.

**Credential Security** — Teams SHALL take all reasonable measures to ensure the security of cloud system credentials. Credentials SHALL be stored in encrypted form where technically feasible. Credential sharing SHOULD be avoided where technically feasible.

**Access Keys** — Account access keys that do not require additional authentication factors and allow account administration activities SHALL only be created after approval by a security specialist and recording of that approval. Unapproved or suspected compromised access keys may be deactivated without notification.

### Account Owner Responsibilities

**Purpose and Content** — Teams assigned as account owners SHALL document the purpose and content of their assigned accounts, maintain required account tags, and promptly respond to requests for information.

**Infrastructure Compliance** — Teams SHALL ensure that infrastructure deployed within their assigned accounts is compliant with the Organization's policies and that systems are patched and updated.

**Unsupported Dependencies** — Systems with dependencies on third-party services or components no longer supported for patching SHALL replace those dependencies with maintainable alternatives. Where not immediately possible, the unsupported dependency SHALL be treated as a vulnerability.

**Software End of Life** — Teams SHALL ensure that software systems that have reached end of life are decommissioned promptly. End-of-life systems that continue to operate SHALL be considered a vulnerability.

**Security Tooling** — Teams SHALL deploy organizationally mandated security tools, instrumentation, and safeguards in their assigned accounts.

**Image Scanning** — Any machine image deployed to production SHALL be scanned for vulnerabilities prior to deployment and periodically thereafter. Vulnerable images SHALL NOT be deployed.

**Dynamic Application Security Testing** — Teams SHALL ensure that DAST scanning is performed for externally accessible endpoints they manage. System failures preventing effective DAST scanning SHALL be treated as a critical vulnerability.

### Segregation of Cloud Data

**Account Purpose** — Cloud accounts SHALL be dedicated to clearly defined functions based on data classification. Accounts designated to contain customer data or host customer-facing services SHALL be designated as production accounts.

**Segregation of Accounts** — Access permissions SHALL be segregated on the basis of production accounts, organization accounts, and non-production accounts. Accounts SHALL NOT be multi-purposed.

**Customer Data Segregation** — SaaS customers SHALL NOT be permitted access to data owned by other customers without express permission. The Organization SHALL employ reasonable technological measures to enforce this segregation.

**Data Replication** — Customer data SHALL NOT be replicated to non-production accounts without prior customer permission.

**Network Segregation** — Production accounts SHALL NOT be placed in the same network as non-production accounts.

### Privacy Controls

**Data Sovereignty** — Region-specific cloud services SHALL be used for customer services and data storage in adherence to contractual data sovereignty provisions.

**Regional Segregation** — Accounts providing production services SHALL only have services active for a single data sovereignty region.

**International Data Transfer** — Customer personally identifiable information SHALL NOT be relocated to other data sovereignty regions without approval from the data owner.

### Confidential Data Storage

**Data Storage Approval** — The Information Security Team SHALL approve the types of data that may be stored in cloud environments.

**Personal Cloud Services** — Personal cloud services accounts SHALL NOT be used for the storage, manipulation, or exchange of company-related communications or company-owned data.

## Definitions

- **Cloud Services**: Third-party services that process or store data outside the Organization's network, including SaaS, IaaS, and PaaS offerings
- **Data Sovereignty**: The principle that data is subject to the laws of the country in which it is stored or processed
- **Single Sign-On (SSO)**: An authentication scheme that allows a user to log in with a single set of credentials to multiple independent software systems

## References

- ISO/IEC 27002: 5.23 Information security for use of cloud services
