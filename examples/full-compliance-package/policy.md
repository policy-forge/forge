# Cloud Security Baseline Policy

## Purpose

This policy establishes the minimum security controls required for all cloud-based systems, services, and workloads operated by or on behalf of the Organization. It maps to NIST SP 800-53 Rev. 5 control families and defines requirements for access control, audit logging, configuration management, identification and authentication, incident response, and system integrity.

## Scope

This policy applies to all cloud infrastructure, Platform-as-a-Service (PaaS), Software-as-a-Service (SaaS), and containerized workloads deployed in any cloud environment (AWS, Azure, GCP, or hybrid). It covers production, staging, and development environments unless explicitly exempted by the Chief Information Security Officer.

## Policy

### Access Control Policy and Procedures

The Organization SHALL develop, document, and disseminate an access control policy that addresses purpose, scope, roles, responsibilities, management commitment, coordination among organizational entities, and compliance. Access control procedures SHALL be reviewed and updated at least annually or when significant changes occur to the cloud environment.

### Account Management

The Organization SHALL manage information system accounts, including establishing, activating, modifying, reviewing, disabling, and removing accounts. Account provisioning and deprovisioning SHALL be completed within 24 hours of a personnel action (onboarding, role change, or termination). Privileged accounts SHALL be limited to designated personnel with a documented business need. Privileged account usage SHALL be logged and reviewed monthly.

### Access Enforcement

1. The information system SHALL enforce approved authorizations for controlling access to the system. Role-Based Access Control (RBAC) SHALL be implemented for all cloud resources, and least privilege SHALL be enforced by default.
2. Production workloads SHALL be isolated from development and testing environments using network segmentation, security groups, or virtual private cloud (VPC) boundaries.
3. Cross-environment access SHALL require explicit approval from the environment owner.

### Audit Events

The Organization SHALL determine that the information system is capable of auditing the following events:

- Successful and unsuccessful logon attempts
- Account creation and modification
- Privilege escalation
- Resource provisioning and deprovisioning
- Security group changes
- Data export operations

Each auditable event SHALL include the timestamp, source IP address, user or service account identity, action performed, and outcome (success or failure).

### Content of Audit Records

Audit records SHALL contain sufficient information to establish what type of event occurred, when it occurred, where it occurred (source and destination), and the outcome. Audit records SHALL be immutable and protected against unauthorized modification or deletion.

### Audit Review, Analysis, and Reporting

The Organization SHALL review and analyze information system audit records at least weekly for indications of inappropriate or unusual activity. Automated alerting SHALL be configured for security-relevant events including privilege escalation, unauthorized access attempts, and configuration changes to security controls. Security audit reports SHALL be generated monthly and distributed to the Information Security team and relevant system owners.

### Baseline Configuration

The Organization SHALL develop, document, and maintain under configuration control, a current baseline configuration of the information system. All cloud resources SHALL be deployed from approved Infrastructure-as-Code (IaC) templates or approved machine images. Deviations from the baseline configuration SHALL require documented approval from the Change Advisory Board and SHALL be time-limited with a remediation plan.

### Least Functionality

1. The Organization SHALL configure the information system to provide only essential capabilities.
2. Unnecessary services, daemons, protocols, and ports SHALL be disabled.
3. Default credentials SHALL be changed before any system goes into production.
4. Only software approved by the Organization's software catalog SHALL be installed on production systems.

### Identification and Authentication

The information system SHALL uniquely identify and authenticate organizational users (or processes acting on behalf of organizational users). Shared accounts SHALL NOT be used for individual user access; each person SHALL have a unique account. Multi-factor authentication (MFA) SHALL be required for all access to cloud management consoles, production systems, and privileged accounts.

### Authenticator Management

The Organization SHALL manage information system authenticators by verifying the identity of the individual, group, role, or device before distributing credentials. Passwords SHALL be at least 14 characters and SHALL NOT be reused for a minimum of 12 iterations. Service account credentials, API keys, and SSH keys SHALL be rotated at least every 90 days. Compromised or potentially compromised credentials SHALL be rotated immediately upon discovery.

### Incident Handling

The Organization SHALL implement an incident handling capability for security incidents that includes preparation, detection and analysis, containment, eradication, and recovery. Incident response procedures SHALL be tested at least semi-annually through tabletop exercises. Security incidents SHALL be reported to the Security Operations Center (SOC) within 1 hour of detection. Critical incidents SHALL be escalated to the CISO within 4 hours.

### Incident Reporting

The Organization SHALL require personnel to report suspected security incidents to the organizational incident response capability. Automated incident reporting SHALL be integrated with the Security Information and Event Management (SIEM) system. Incidents involving regulated data (PII, PHI, financial data) SHALL be reported to the appropriate regulatory bodies within the timeframes required by applicable laws and contracts.

### Boundary Protection

1. The information system SHALL monitor and control communications at the external boundary of the system and at key internal boundaries.
2. A default-deny firewall policy SHALL be enforced, with only explicitly approved traffic permitted.
3. All communications crossing trust boundaries SHALL be encrypted using TLS 1.2 or higher.
4. Internal service-to-service communications within the production environment SHALL use mutual TLS (mTLS) where technically feasible.

### Transmission Confidentiality and Integrity

The information system SHALL protect the confidentiality and integrity of transmitted information. Sensitive data SHALL be encrypted using FIPS 140-2 validated cryptographic modules. Public-facing services SHALL use certificates from a trusted Certificate Authority (CA).

### Flaw Remediation

1. The Organization SHALL identify, report, and correct information system flaws.
2. Critical and high severity vulnerabilities SHALL be remediated within 15 business days of discovery.
3. Medium severity vulnerabilities SHALL be remediated within 30 business days.
4. Security patches SHALL be applied to all production systems within 14 days of release.
5. Emergency patches for actively exploited vulnerabilities SHALL be applied within 48 hours.

### System Monitoring

The information system SHALL monitor events on the system to detect attacks and indicators of potential attacks. Intrusion detection systems (IDS) or intrusion prevention systems (IPS) SHALL be deployed at network boundaries and on critical hosts. All security-relevant logs SHALL be aggregated into a centralized SIEM platform. Log sources SHALL include cloud audit logs, application logs, network flow data, and endpoint telemetry. Log retention SHALL be a minimum of 365 days for security logs.

## References

- NIST SP 800-53 Rev. 5: Security and Privacy Controls for Information Systems and Organizations
- NIST SP 800-137: Continuous Monitoring
- CIS Controls v8: Implementation Groups 1 and 2
- ISO/IEC 27001:2022: Information Security Management Systems
