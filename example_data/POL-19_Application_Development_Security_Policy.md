# Application Development Security Policy

## Purpose

This policy defines the requirements for the secure development, testing, and deployment of applications developed in-house or by third parties.

## Scope

This policy applies to all software development performed by or for the Organization. The intended audience is developers, managers, architects, security specialists, site-reliability engineers, IT employees, and partners.

## Policy

### Roles and Responsibilities

**Employee Responsibilities** — All employees involved in specification, development, testing, or documentation of applications MUST be familiar with this policy.

**Security Responsibility** — Each application SHALL have a designated individual responsible for the overall security of the system.

**Separation of Responsibility** — Analysts, architects, and developers SHALL NOT review their own work. All production system changes SHALL be peer reviewed by at least one other person.

**Qualified Developers** — Developers SHALL have appropriate training and expertise for assigned tasks. Employees involved in coding SHOULD have appropriate knowledge of secure coding principles.

### Development Lifecycle

**Secure SDLC** — The Organization SHALL follow a secure software development lifecycle that includes: product approval, architectural approval, coding and testing, pre-release validation, and release phases.

**Architecture Review** — Development SHALL NOT begin without peer-reviewed architectural documentation. Security concerns SHALL be documented in a clearly identifiable section of the documentation. Threat modeling SHOULD be performed where applicable.

**Definition of Done** — Development features SHALL NOT be considered "done" unless all components are complete: requirement gathering, secure architecture analysis, design, coding, review, testing, and deployment.

### Secure Coding Practice

**Secure Coding Methods** — All source code MUST use secure coding methods approved by the Architecture and Information Security departments.

**OWASP Principles** — Coding SHALL follow, at minimum, the OWASP Secure Coding Principles.

**Data Validation** — Input data SHALL be validated at multiple levels. Business logic SHALL NOT trust or assume the existence of validation performed by the input layer. Where possible, data-access layers SHALL also perform validation.

**Encryption Standards** — Applications using encryption SHALL document the methods and types used. No new development SHALL use deprecated encryption mechanisms.

### Test Data

**Customer Data Protection** — Customer data SHALL NOT be used for testing unless documented customer approval has been obtained. Testing for systems handling private information MUST NOT employ unsanitized production information.

**Test Data Removal** — Test data and accounts MUST be removed before a production system becomes active.

**Dynamic Testing** — All software releases exposing a web interface or API MUST be analyzed by dynamic analysis tools prior to production deployment.

### Change Control

**Change Requirements** — A change SHALL only be approved if it has an associated tracking ticket describing the intent and associating it with a specific team.

**Change Approvals** — Changes SHALL be reviewed by peers and approved before incorporation into release branches. The person submitting a change SHALL NOT be the sole approver. At least two people SHALL consider any change before deployment.

**Emergency Changes** — Emergency changes may bypass standard restrictions but SHALL be fully documented in a tracking ticket. A follow-up review SHALL be performed by a responsible manager.

**Major Changes** — Significant changes involving large infrastructure modifications or security-impacting changes SHALL have secondary review by a specialist, confirming backout plans, documentation, and architectural review.

### Source Control

**Source Code Management** — All production source code SHALL be stored in a secure source code control and management system.

**Release Branch Security** — Release branches SHALL require pull requests with at least one approval from a person other than the committer. All releases SHALL be identified with tags.

**No Secrets in Source** — Secrets, including cryptographic keys, passwords, and access tokens, SHALL NOT be present in source code. Discovered secrets SHALL be immediately invalidated and removed.

**No Backdoors** — Developers SHALL NOT build or deploy secret user IDs or passwords with special privileges.

### Build Systems

**Automated Build and Test** — All software releases SHALL be built by an automated build system that runs automated tests and vulnerability scanning.

**Automated Deployment** — Production deployments SHALL be performed by an automated system that can only deploy builds that have passed all automated tests.

### Security Testing

**Automated Scanning** — Automated vulnerability scanning tools SHALL assess all production source code and dependencies. Security vulnerabilities MUST be addressed prior to production deployment.

**Repository Testing** — All production repositories SHALL undergo automated testing on build, secret scanning, compositional dependency analysis (SCA), static analysis (SAST), and container scanning.

**Product Testing** — All products SHALL undergo external penetration testing (at least annually), authenticated dynamic analysis (DAST at least weekly), and network scanning of exposed endpoints (at least weekly).

### Audit and Logging

**Security Event Logging** — Applications SHOULD produce a log of security-related events in a format supporting SIEM ingestion. Logs SHALL be structured as JSON data.

**Operational Logging** — Applications SHOULD generate logs memorializing significant events with timestamps, including data modifications, API calls, and configuration changes.

### Security of Dependencies

**Open-Source Standards** — Open-source software SHALL NOT be used without tracking and assigning accountability. A list of all third-party software packages SHALL be produced for all applications.

**Open-Source Support** — Production software SHOULD employ open-source that is supported by a reputable organization, has been available for at least six months, and is maintained with timely patches.

### Application Decommission

**Media Sanitization** — On decommissioning, any media storing application code or data SHALL be sanitized according to established guidelines.

**Documentation Confidentiality** — All computer-related documentation is confidential and SHALL NOT be removed when a worker leaves employment.

## Definitions

- **Secure Software Development Lifecycle (SDLC)**: A structured process integrating security at every phase of software development
- **Static Application Security Testing (SAST)**: Analysis of source code to detect security vulnerabilities without executing the application
- **Dynamic Application Security Testing (DAST)**: Testing of running applications to find security vulnerabilities
- **Software Composition Analysis (SCA)**: Analysis of third-party and open-source components for known vulnerabilities

## References

- ISO/IEC 27002: 8.25 Secure development life cycle, 8.26 Application security requirements, 8.27 Secure system architecture and engineering principles, 8.28 Secure coding, 8.29 Security testing in development and acceptance, 8.30 Outsourced development, 8.33 Test information
