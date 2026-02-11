# System Acquisition and Configuration Policy

## Purpose

This policy defines the requirements for acquiring information systems and related technology components that conform to the Organization's information security policies, and for managing default configurations and system acceptance.

## Scope

This policy applies to all information technology hardware, software, and computer-related components that are acquired by the Organization and process or store confidential data, including cloud services, application software, and configurable hardware.

## Policy

### System Acquisition

**Security Requirements Identification** — Before a new information system is developed or acquired, management of the involved department, working with the Information Security Team, SHALL have clearly specified and documented the relevant security impacts and requirements. This process may be bypassed only if the system is effectively a replica of an existing system, in which case this SHALL be noted.

**Security Functionality** — Whenever feasible, third-party business systems SHALL rely on system services for security functionality rather than incorporating such functionality into applications. Examples include single sign-on services, operating systems, database management systems, access control packages, firewalls, and gateways.

**Purchase Requests** — All purchase requests for hardware, software, or computer-related components SHALL first be approved by a department head or designee before submission to the IT department.

**Risk Assessment** — If applicable, security threat and risk assessment during the requirements phase SHALL be conducted when developing, implementing major changes, or acquiring systems, to identify security requirements and threats.

### Capacity Management

**Capacity Requirements** — Capacity requirements SHALL be identified for each new and ongoing activity that requires the use of information technology resources.

**Tuning and Monitoring** — System tuning and monitoring SHALL be applied to all information technology resources.

**Future Projections** — Projections of future capacity requirements SHALL consider new business and system requirements and current and projected trends in information processing capabilities.

**Key Resources** — Management SHALL monitor the utilization of key system resources, especially those with long procurement lead times or high costs.

**Systems Expertise** — Critical computer and communications expertise SHALL be possessed by at least two immediately available persons.

**Auto-Scaling** — Cloud systems employing auto-scaling SHOULD have their limitations explicitly identified and addressed in capacity planning and documentation.

### System Acceptance

**Acceptance Criteria** — The acceptance criteria for new information systems, upgrades, and new versions SHALL include: performance requirements, capacity requirements, error recovery and restart procedures, contingency plans, routine operating procedures, system impact assessment, training requirements, and ease of use.

**Testing** — Appropriate tests SHALL be performed to confirm that all acceptance criteria have been fully satisfied.

**End-User Development** — All software that handles confidential information and has been developed by end users SHALL have its controls approved as part of the secure software development lifecycle process.

### Configuration Controls

**Baseline Standards** — All IT information systems SHALL conform to minimum security configuration standards defined by the Information Security Department.

**Default Passwords** — All vendor-supplied default passwords SHALL be changed before any computer or communications system is used for business.

**Least Privilege** — The principle of least privilege SHOULD be followed when accounts are given permissions to carry out tasks.

**System Hardening** — Operating systems deployed into production SHALL be hardened according to a policy defined for the use case and operating system. Where available, accepted security hardening standards such as CIS Benchmarks SHALL be applied.

**Unnecessary Software** — Software and features that are unnecessary for the intended tasks SHOULD be disabled or removed.

### Remote Management

**Access Encryption** — All non-local access to systems SHALL be encrypted using methods approved by the Information Security Department.

### Patches and Updates

**Server Software** — Only authorized Systems Administrators are permitted to install and update software on servers managed by the IT department.

**Security Updates** — All networked production systems SHALL have a process for detecting and installing operating system and application software security updates.

**Patch Verification** — Systems Administrators SHALL patch software only if downloaded from a trusted and recognized source. All patches with a digital signature SHOULD have the signature verified prior to installation.

**Patch Timing** — All security patches SHALL be installed based on schedules commensurate with the criticality of applications, systems, and contractual obligations.

**Change Control for System Software** — New or different versions of operating system and related systems software for production computers SHALL go through the established change control process prior to installation.

### System Testing

**Test Environment Separation** — Separation between testing and production environments SHALL be implemented to reduce the risk of negligent or deliberate systems misuse.

**Production Data** — Where appropriate, no live data SHALL be used to perform testing. Production data SHALL NOT be migrated to test or staging environments without appropriate controls.

### Vulnerability Management

**Security Forums** — Information security professionals SHALL maintain memberships with security forums and professional associations to receive early warnings of alerts, advisories, and patches.

**Vulnerability Detection** — Automated tooling used for system vulnerability analysis SHALL be configured to ensure it is always current with the latest detection updates.

**Vulnerability Scanning** — All systems directly connected to the Internet SHALL be subjected to automated risk analysis performed via vulnerability identification software.

**Scan Reviews** — The results of all vulnerability assessments of production systems SHALL be reviewed by technical personnel.

## Definitions

- **Capacity Planning**: The process of determining information processing capacity needed to meet changing demands
- **System Acceptance**: The demonstrable willingness and verification within a user group to employ information technology for its intended purpose
- **System Hardening**: The process of securing a system by reducing its attack surface through removal of unnecessary software, services, and privileges

## References

- ISO/IEC 27002: 5.19 Information security in supplier relationships, 8.6 Capacity management, 8.8 Configuration management, 8.9 Configuration management, 8.19 Installation of software on operational systems, 8.29 Security testing in development and acceptance
