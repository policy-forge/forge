//! Deterministic synthetic policy document generator for benchmarking (WI-24).
//!
//! Produces a ~150KB Markdown policy document exercising all pipeline stages:
//! sections, clauses, compound statements, citations, and tables.
//!
//! # Determinism
//! No randomness, no system time, no RNG. Two calls produce byte-identical output.
#![allow(dead_code)]

use std::fmt::Write;

/// Domain definition for a top-level policy section (H2).
struct Domain {
    title: &'static str,
    intro: &'static str,
    subsections: &'static [Subsection],
}

/// Subsection definition (H3) within a domain.
struct Subsection {
    title: &'static str,
    body: &'static str,
    requirements: &'static [&'static str],
    /// If Some, a table is included after the requirements.
    table: Option<&'static str>,
    /// Optional H4 sub-subsection for deeper hierarchy testing.
    h4_block: Option<&'static str>,
}

// ─── Domain Definitions ─────────────────────────────────────────────────

const DOMAINS: &[Domain] = &[
    // ── 1. Access Control ──────────────────────────────────────────────
    Domain {
        title: "Access Control",
        intro: "\
The organization shall establish, document, and maintain a comprehensive access control program \
to protect information systems and data from unauthorized access, use, disclosure, disruption, \
modification, or destruction. Access control policies and procedures shall be reviewed at least \
annually and updated as necessary to reflect changes in the threat landscape, organizational \
structure, and regulatory requirements. The Chief Information Security Officer shall be responsible \
for oversight of the access control program and shall ensure that appropriate resources are \
allocated for its implementation and maintenance. All personnel, including employees, contractors, \
and third-party service providers, shall comply with access control requirements as a condition \
of their access to organizational information systems. Violations of access control policy may \
result in disciplinary action up to and including termination of employment or contract. The \
organization shall implement the principle of least privilege across all information systems, \
ensuring that users are granted only the minimum access necessary to perform their assigned \
duties. Access control mechanisms shall be implemented at all layers of the technology stack, \
including network, operating system, application, and data layers.",
        subsections: &[
            Subsection {
                title: "User Account Management",
                body: "\
User account management procedures shall ensure that all accounts are created, modified, \
suspended, and terminated in accordance with organizational policies and applicable regulatory \
requirements. The identity management team shall maintain a current inventory of all user \
accounts across organizational information systems. Account provisioning shall require formal \
authorization from the account holder's supervisor and the system owner prior to account creation. \
Service accounts and shared accounts shall be individually tracked and assigned to a responsible \
administrator who shall ensure compliance with password policies and access restrictions. \
Dormant accounts that have not been used within ninety calendar days shall be automatically \
disabled and reviewed for potential removal. The account management process shall integrate \
with the human resources lifecycle to ensure timely account creation during onboarding and \
prompt account termination during offboarding. Quarterly access reviews shall be conducted \
by system owners to verify that all active accounts remain necessary and appropriately scoped.",
                requirements: &[
                    "The organization shall create user accounts only upon receipt of a documented and approved access request from an authorized manager or system owner.",
                    "All user accounts shall be uniquely identified using the employee identifier assigned by the human resources department and must not be shared between individuals.",
                    "The identity management team shall disable user accounts within twenty-four hours of notification of employment termination, contract completion, or extended leave of absence.",
                    "System administrators shall conduct quarterly reviews of all active accounts and shall remove or disable accounts that are no longer required for business operations.",
                    "The organization shall implement automated mechanisms to disable accounts after ninety consecutive days of inactivity and shall log all automated account status changes.",
                ],
                table: Some(
                    "\
| Role | Account Type | Review Frequency | Approver |\n\
| --- | --- | --- | --- |\n\
| Employee | Individual | Quarterly | Direct Supervisor |\n\
| Contractor | Individual | Monthly | Contract Manager |\n\
| System Administrator | Privileged | Monthly | CISO |\n\
| Service Account | Non-Interactive | Quarterly | System Owner |\n\
| Vendor | External | Monthly | Vendor Manager |\n\
| Executive | Individual | Annually | Board Designee |\n\
| Intern | Temporary | Weekly | Supervising Manager |\n\
| Auditor | Read-Only | Per Engagement | Audit Director |",
                ),
                h4_block: None,
            },
            Subsection {
                title: "Authentication Requirements",
                body: "\
The organization shall implement multi-factor authentication for all access to information \
systems containing sensitive or classified data. Authentication mechanisms shall be selected \
based on a risk assessment that considers the sensitivity of the data, the threat environment, \
and the operational requirements of the user population. Password-based authentication shall \
comply with NIST SP 800-63B guidelines for memorized secrets. The organization shall maintain \
a list of commonly used, expected, or compromised passwords and shall prevent users from \
selecting passwords that appear on this list. Authentication events shall be logged with \
sufficient detail to support incident investigation and forensic analysis. Failed authentication \
attempts shall be limited to five consecutive attempts before temporary account lockout. The \
lockout duration shall be at least fifteen minutes or until an administrator manually unlocks \
the account. The organization shall implement session management controls including automatic \
session termination after thirty minutes of inactivity.",
                requirements: &[
                    "All users shall authenticate using multi-factor authentication that combines something the user knows with something the user possesses as specified in NIST SP 800-63B.",
                    "Passwords shall be a minimum of fourteen characters in length and must not appear on the organizational list of prohibited passwords maintained by the security operations team.",
                    "The authentication system shall enforce account lockout after five consecutive failed authentication attempts and shall maintain the lockout for a minimum of fifteen minutes.",
                    "Authentication credentials shall be transmitted only over encrypted channels using Transport Layer Security version 1.2 or higher in compliance with RFC 8446.",
                    "The organization shall implement certificate-based authentication for all system-to-system communications and must rotate certificates at least annually.",
                ],
                table: None,
                h4_block: Some(
                    "\
#### Biometric Authentication Standards\n\n\
Where biometric authentication is deployed, the organization shall ensure that biometric \
templates are stored in encrypted form using algorithms approved by NIST SP 800-175B. \
Biometric sensors shall achieve a false acceptance rate of no more than one in one hundred \
thousand and a false rejection rate of no more than one in one hundred. The organization \
shall provide alternative authentication mechanisms for individuals who cannot use biometric \
systems due to physical limitations or religious objections. Biometric data shall be classified \
as personally identifiable information and handled in accordance with the data protection \
requirements specified in Section 2 of this policy.",
                ),
            },
            Subsection {
                title: "Authorization and Privileges",
                body: "\
The organization shall implement role-based access control as the primary authorization \
mechanism for all information systems. Access privileges shall be assigned based on job \
function, operational need, and the principle of least privilege. Privileged access including \
system administrator, database administrator, and security administrator accounts shall require \
enhanced controls including separate privileged and non-privileged accounts for each user, \
enhanced logging, and periodic review by the Chief Information Security Officer. Privilege \
escalation shall require formal approval through the change management process and shall be \
time-limited whenever operationally feasible. The organization shall maintain documentation \
of all role definitions, associated permissions, and the personnel assigned to each role. \
Separation of duties shall be enforced for critical business processes to prevent any single \
individual from controlling all aspects of a transaction or operation.",
                requirements: &[
                    "Access privileges shall be assigned based on role-based access control principles and must be documented in the organizational access control matrix maintained by the security team.",
                    "Privileged accounts shall be assigned to named individuals only and must not be shared between personnel as required by NIST SP 800-53 AC-6.",
                    "The organization shall enforce separation of duties for all critical business processes and shall implement compensating controls where technical enforcement is not feasible.",
                    "Privilege escalation requests shall be approved through the change management process and shall be automatically revoked after the approved time period expires.",
                    "The security team shall review all privileged access assignments monthly and shall revoke privileges that are no longer required for the assigned role. See Section 1.1 for account review procedures.",
                ],
                table: None,
                h4_block: None,
            },
            Subsection {
                title: "Remote Access",
                body: "\
Remote access to organizational information systems shall be controlled through secure virtual \
private network connections or equivalent encrypted communication channels approved by the \
information security team. All remote access sessions shall be authenticated using multi-factor \
authentication regardless of the sensitivity level of the systems being accessed. The \
organization shall monitor remote access sessions in real time and shall terminate sessions \
that exhibit anomalous behavior including connections from unauthorized geographic locations, \
connections outside approved business hours, or unusual data transfer patterns. Remote access \
shall be restricted to organization-managed devices that meet the minimum security baseline \
defined in the endpoint security policy. Split tunneling shall be prohibited on devices used \
for remote access to ensure that all network traffic is routed through the organizational \
security controls. The organization shall maintain a current inventory of all remote access \
tools and protocols authorized for use.",
                requirements: &[
                    "Remote access to organizational systems shall require virtual private network connectivity using Internet Protocol Security or Transport Layer Security as approved by the security team.",
                    "All remote access sessions shall be authenticated using multi-factor authentication and shall be encrypted end-to-end using cryptographic algorithms approved by NIST SP 800-52.",
                    "The organization shall monitor all remote access sessions in real time and must terminate sessions originating from geographic locations not on the approved access list.",
                    "Remote access shall be permitted only from organization-managed devices that meet the security baseline and must have current endpoint protection software installed and active.",
                    "Split tunneling shall be prohibited on all devices used for remote access and must be enforced through technical controls in the VPN client configuration.",
                ],
                table: None,
                h4_block: None,
            },
        ],
    },
    // ── 2. Data Protection ─────────────────────────────────────────────
    Domain {
        title: "Data Protection",
        intro: "\
The organization shall establish and maintain a comprehensive data protection program to safeguard \
the confidentiality, integrity, and availability of all organizational data throughout its lifecycle. \
Data protection controls shall be applied based on the classification level of the data and the \
risk assessment associated with each data processing activity. The data protection program shall \
comply with all applicable federal, state, and international data protection regulations including \
but not limited to the General Data Protection Regulation, the Health Insurance Portability and \
Accountability Act, and the Federal Information Security Modernization Act. The Chief Privacy Officer \
shall be responsible for maintaining the data protection policy and shall coordinate with the Chief \
Information Security Officer to ensure alignment between privacy and security requirements. All \
personnel shall receive annual training on data protection responsibilities and shall acknowledge \
their understanding of data handling procedures. The organization shall conduct annual reviews of \
data protection controls and shall update procedures to address emerging threats and regulatory changes.",
        subsections: &[
            Subsection {
                title: "Data Classification",
                body: "\
All organizational data shall be classified according to the four-tier classification scheme \
defined in this policy: Public, Internal, Confidential, and Restricted. Data owners shall be \
responsible for assigning the appropriate classification level to all data under their stewardship. \
Classification decisions shall be based on the potential impact to the organization if the data \
were to be disclosed, modified, or destroyed without authorization. Classification labels shall be \
applied to all data storage locations, transmission channels, and processing systems. Data that \
combines elements from multiple classification levels shall be classified at the highest level \
of any component element. The classification of data shall be reviewed whenever there is a \
material change in the data content, regulatory requirements, or business context. Automated \
data classification tools shall be deployed to assist data owners in maintaining accurate and \
consistent classification across large data repositories.",
                requirements: &[
                    "All organizational data shall be classified into one of four tiers: Public, Internal, Confidential, or Restricted, based on the potential impact of unauthorized disclosure as defined in FIPS 199.",
                    "Data owners shall assign classification labels to all data assets within thirty calendar days of creation and shall review classifications annually.",
                    "Mixed-classification datasets shall be classified at the highest classification level of any component data element and must be protected accordingly.",
                    "Automated data classification tools shall be deployed for all repositories containing more than ten thousand records and must achieve classification accuracy of at least ninety-five percent.",
                    "The data governance committee shall maintain the authoritative classification guide and shall publish updates within fourteen business days of any regulatory change. See Table 2 for the classification matrix.",
                ],
                table: Some(
                    "\
| Classification | Description | Examples | Handling Requirements |\n\
| --- | --- | --- | --- |\n\
| Public | No impact if disclosed | Marketing materials, press releases | No restrictions |\n\
| Internal | Low impact if disclosed | Internal memos, meeting notes | Encrypt in transit |\n\
| Confidential | Moderate impact if disclosed | Employee records, financial data | Encrypt at rest and in transit |\n\
| Restricted | Severe impact if disclosed | Trade secrets, PII, PHI | Encrypt, access log, DLP |",
                ),
                h4_block: None,
            },
            Subsection {
                title: "Encryption Standards",
                body: "\
The organization shall implement encryption controls to protect data confidentiality and integrity \
during storage, transmission, and processing. Encryption algorithms and key lengths shall comply \
with NIST SP 800-175B recommendations and shall be reviewed annually against current cryptanalytic \
capabilities. The organization shall maintain a cryptographic key management infrastructure that \
supports the full key lifecycle including generation, distribution, storage, rotation, revocation, \
and destruction. Hardware security modules certified to FIPS 140-3 Level 2 or higher shall be \
used for the storage and management of root certificate authority keys and other high-value \
cryptographic keys. Key rotation schedules shall be defined based on the classification level \
of the data being protected and the cryptographic algorithm in use. All deprecated or weakened \
algorithms shall be removed from organizational systems within ninety days of deprecation notification.",
                requirements: &[
                    "All data classified as Confidential or Restricted shall be encrypted at rest using Advanced Encryption Standard with a minimum key length of two hundred and fifty-six bits as specified in NIST SP 800-175B.",
                    "Data in transit shall be protected using Transport Layer Security version 1.2 or higher and must use cipher suites approved by the organization in compliance with RFC 8446.",
                    "Cryptographic keys shall be managed using hardware security modules certified to FIPS 140-3 Level 2 or higher for all Restricted data encryption operations.",
                    "Symmetric encryption keys shall be rotated at least annually and must be rotated immediately upon suspected or confirmed compromise as required by the key management procedures.",
                    "The organization shall maintain an inventory of all cryptographic implementations and shall remove deprecated algorithms within ninety calendar days of deprecation notification from NIST.",
                ],
                table: None,
                h4_block: None,
            },
            Subsection {
                title: "Data Retention and Archival",
                body: "\
The organization shall define and enforce data retention periods based on legal, regulatory, and \
business requirements. Data shall be retained for the minimum period necessary to satisfy all \
applicable retention obligations and shall be securely disposed of upon expiration of the retention \
period. The data retention schedule shall be maintained by the records management office in \
coordination with legal counsel and shall be reviewed annually to ensure compliance with current \
regulatory requirements. Archived data shall maintain the same classification and protection \
controls as active data throughout the retention period. The organization shall implement automated \
retention management mechanisms to ensure consistent enforcement of retention periods and to \
prevent both premature deletion and indefinite retention. Litigation hold procedures shall be \
established to preserve relevant data when litigation is reasonably anticipated.",
                requirements: &[
                    "The organization shall maintain a data retention schedule that specifies retention periods for all categories of organizational data in compliance with applicable federal and state regulations.",
                    "Data shall be securely disposed of within ninety calendar days of the expiration of its retention period and must be disposed of using methods appropriate to its classification level.",
                    "Archived data shall maintain the same classification level and protection controls as active data and shall be accessible for retrieval within seventy-two hours of a valid request.",
                    "The records management office shall conduct annual reviews of the retention schedule and shall update retention periods within thirty days of identifying new regulatory requirements.",
                    "Litigation hold procedures shall be initiated within twenty-four hours of notification from legal counsel and must preserve all relevant data in its current state.",
                ],
                table: None,
                h4_block: None,
            },
            Subsection {
                title: "Data Disposal",
                body: "\
The organization shall implement secure data disposal procedures to ensure that data is rendered \
unrecoverable when it is no longer needed. Disposal methods shall be selected based on the data \
classification level and the storage medium type. Physical media containing Confidential or \
Restricted data shall be destroyed using approved methods including degaussing, shredding, or \
incineration performed by certified destruction vendors. Electronic data on reusable storage \
media shall be sanitized using NIST SP 800-88 compliant methods before the media is repurposed \
or disposed of. The organization shall maintain disposal records including the date of disposal, \
the method used, the classification level of the data disposed, and the identity of the individual \
who performed or verified the disposal. Cloud service providers shall provide documented evidence \
of data disposal upon termination of service agreements.",
                requirements: &[
                    "Physical storage media containing Confidential or Restricted data shall be destroyed using methods compliant with NIST SP 800-88 Guidelines for Media Sanitization.",
                    "Electronic data sanitization shall use a minimum of one overwrite pass for Internal data and must use cryptographic erasure or three-pass overwrite for Confidential and Restricted data.",
                    "The organization shall maintain disposal records for a minimum of seven years and shall include the date, method, data classification, and responsible individual for each disposal event.",
                    "Cloud service providers shall provide written certification of data disposal within thirty calendar days of contract termination or service migration.",
                    "The data disposal process shall be audited annually by an independent assessor and must achieve a compliance rate of at least ninety-eight percent. See Appendix A for approved disposal methods.",
                ],
                table: None,
                h4_block: None,
            },
        ],
    },
    // ── 3. Incident Response ───────────────────────────────────────────
    Domain {
        title: "Incident Response",
        intro: "\
The organization shall establish, implement, and maintain a comprehensive incident response \
capability to detect, analyze, contain, eradicate, and recover from information security incidents. \
The incident response program shall be aligned with NIST SP 800-61 guidelines and shall address \
incidents across all organizational information systems, networks, and data repositories. An \
incident response team shall be established with clearly defined roles, responsibilities, and \
authority to act during security incidents. The team shall include representatives from information \
security, information technology operations, legal counsel, human resources, public relations, and \
senior management. The incident response plan shall be tested at least annually through tabletop \
exercises or full-scale simulations. Lessons learned from actual incidents and exercises shall be \
incorporated into plan updates within thirty days. The organization shall establish communication \
channels and escalation procedures to ensure timely notification of relevant stakeholders during \
incidents of varying severity levels.",
        subsections: &[
            Subsection {
                title: "Incident Detection and Reporting",
                body: "\
The organization shall deploy detection capabilities across all network segments, endpoints, and \
cloud environments to identify potential security incidents in near real time. Security information \
and event management systems shall correlate events from multiple sources to identify patterns \
indicative of malicious activity. Detection rules shall be updated at least weekly to reflect \
current threat intelligence. All personnel shall be trained to recognize and report potential \
security incidents through the designated reporting channels. The security operations center \
shall acknowledge all incident reports within one hour and shall perform initial triage within \
four hours. Anonymous reporting mechanisms shall be available for personnel who wish to report \
suspicious activities without identifying themselves. The organization shall track all reported \
incidents from initial detection through final resolution using the incident management system.",
                requirements: &[
                    "The organization shall deploy security information and event management systems across all network segments and must correlate events from a minimum of ten distinct log sources.",
                    "Detection rules and signatures shall be updated at least weekly based on current threat intelligence feeds and must be tested against known attack patterns before deployment.",
                    "All personnel shall report suspected security incidents within one hour of detection through the designated incident reporting system or the security operations center hotline.",
                    "The security operations center shall acknowledge incident reports within one hour and shall complete initial triage and severity classification within four hours of receipt.",
                    "The organization shall maintain an incident tracking system that records all incidents from detection through resolution and must retain incident records for a minimum of five years.",
                ],
                table: Some(
                    "\
| Severity Level | Description | Response Time | Escalation |\n\
| --- | --- | --- | --- |\n\
| Critical | Active data breach or system compromise | Immediate | CISO and CEO |\n\
| High | Attempted breach or malware detection | Within 1 hour | CISO and IT Director |\n\
| Medium | Policy violation or suspicious activity | Within 4 hours | Security Manager |\n\
| Low | Minor policy deviation or informational | Within 24 hours | SOC Analyst |\n\
| Informational | False positive or resolved alert | Within 48 hours | Logged only |",
                ),
                h4_block: None,
            },
            Subsection {
                title: "Incident Handling and Containment",
                body: "\
Upon confirmation of a security incident, the incident response team shall execute the appropriate \
containment strategy based on the incident type and severity. Containment strategies shall be \
predefined for common incident categories including malware infections, unauthorized access, \
data exfiltration, denial of service attacks, and insider threats. Short-term containment actions \
shall be designed to limit the immediate impact of the incident while preserving forensic evidence. \
Long-term containment actions shall address the root cause and prevent recurrence while allowing \
business operations to continue. All containment actions shall be documented in the incident \
record with timestamps, rationale, and the identity of the personnel who authorized and executed \
each action. The incident response team shall coordinate with external parties including law \
enforcement, regulatory bodies, and affected third parties as required by the incident severity \
and applicable notification requirements.",
                requirements: &[
                    "The incident response team shall execute predefined containment procedures within two hours of incident confirmation for Critical and High severity incidents.",
                    "Containment actions shall preserve forensic evidence including system logs, memory dumps, and network traffic captures and must maintain chain of custody documentation.",
                    "All containment decisions and actions shall be recorded in the incident management system with timestamps and must be approved by the incident commander or designee.",
                    "The organization shall maintain pre-authorized containment actions including network isolation and account suspension that can be executed without additional management approval during Critical incidents.",
                    "External communications during incident response shall be coordinated through the public relations team and must be approved by legal counsel prior to release.",
                ],
                table: None,
                h4_block: None,
            },
            Subsection {
                title: "Incident Recovery",
                body: "\
The organization shall establish recovery procedures to restore affected systems and data to \
normal operations following a security incident. Recovery procedures shall be prioritized based \
on the business impact assessment and shall ensure that critical business functions are restored \
first. System restoration shall be performed from known-good backups that have been verified \
to be free of compromise. The incident response team shall verify the integrity of restored \
systems before returning them to production use. Recovery activities shall include validation \
that the vulnerability exploited in the incident has been remediated and that monitoring \
controls have been enhanced to detect similar attacks. The organization shall track recovery \
metrics including time to restore normal operations and shall use these metrics to improve \
recovery procedures.",
                requirements: &[
                    "Critical business systems shall be restored to normal operations within the recovery time objectives defined in the business continuity plan following containment and eradication of the incident.",
                    "System restoration shall be performed from verified backups and must include integrity validation before the system is returned to production. See Section 8.3 for backup procedures.",
                    "The incident response team shall verify that all exploited vulnerabilities have been remediated and shall enhance monitoring controls before declaring the incident resolved.",
                    "Recovery procedures shall be documented in the incident record and must include a timeline of restoration activities and verification steps performed.",
                    "The organization shall conduct a post-recovery validation within seven calendar days to confirm that restored systems are operating normally and that no residual indicators of compromise exist.",
                ],
                table: None,
                h4_block: None,
            },
            Subsection {
                title: "Post-Incident Analysis",
                body: "\
The organization shall conduct a formal post-incident analysis for all incidents classified as \
Medium severity or higher. The analysis shall be completed within thirty calendar days of incident \
closure and shall include a comprehensive timeline of events, root cause analysis, effectiveness \
assessment of the incident response, and actionable recommendations for improvement. Post-incident \
analysis meetings shall include all members of the incident response team and representatives from \
affected business units. Findings and recommendations shall be presented to senior management and \
shall be tracked through the corrective action process until fully implemented. The organization \
shall maintain a repository of post-incident analysis reports that is accessible to the security \
team for reference during future incidents.",
                requirements: &[
                    "A formal post-incident analysis shall be conducted for all incidents classified as Medium severity or higher and shall be completed within thirty calendar days of incident closure.",
                    "Post-incident analysis reports shall include a detailed timeline, root cause analysis, response effectiveness assessment, and shall identify specific corrective actions with assigned owners and deadlines.",
                    "Corrective actions identified during post-incident analysis shall be tracked through the organizational corrective action system and must be implemented within ninety calendar days of identification.",
                    "Lessons learned from incident analysis shall be incorporated into security awareness training materials within sixty calendar days and shall be distributed to all relevant personnel.",
                    "The organization shall review incident trends quarterly and shall update the incident response plan to address recurring incident patterns and emerging threat vectors.",
                ],
                table: None,
                h4_block: None,
            },
        ],
    },
    // ── 4. Risk Management ─────────────────────────────────────────────
    Domain {
        title: "Risk Management",
        intro: "\
The organization shall implement a risk management framework aligned with NIST SP 800-37 to \
identify, assess, mitigate, and monitor information security risks across all organizational \
operations and assets. The risk management program shall integrate with the organizational \
governance structure and shall inform decision-making at all levels. Risk assessments shall consider \
threats from both internal and external sources including nation-state actors, cybercriminals, \
insider threats, and natural disasters. The risk management framework shall establish risk tolerance \
thresholds for each organizational unit and shall escalate risks exceeding tolerance to the \
executive risk committee. Risk management activities shall be documented and reported to the \
board of directors at least quarterly. The Chief Risk Officer shall maintain oversight of the \
risk management program and shall ensure coordination with information security, compliance, and \
business continuity functions.",
        subsections: &[
            Subsection {
                title: "Risk Assessment Methodology",
                body: "\
The organization shall conduct comprehensive risk assessments at least annually and whenever \
significant changes occur to the information system environment, business processes, or threat \
landscape. Risk assessments shall follow the methodology defined in NIST SP 800-30 and shall \
include asset identification and valuation, threat identification, vulnerability identification, \
likelihood determination, and impact analysis. Assessment results shall be documented in the \
organizational risk register and shall include risk ratings for each identified risk. The risk \
assessment methodology shall incorporate both quantitative and qualitative analysis techniques \
appropriate to the risk being assessed. Vulnerability scanning shall be conducted at least \
monthly on all internet-facing systems and quarterly on all internal systems to identify \
technical vulnerabilities that may be exploited by threat actors.",
                requirements: &[
                    "The organization shall conduct comprehensive risk assessments at least annually and within thirty days of any significant change to the information system environment as defined in NIST SP 800-30.",
                    "Risk assessments shall identify and evaluate threats from all relevant sources including nation-state actors, cybercriminals, insider threats, and natural disasters.",
                    "All identified risks shall be recorded in the organizational risk register with a risk rating and must be assigned to a risk owner who is accountable for mitigation.",
                    "Vulnerability scanning shall be conducted at least monthly on all internet-facing systems and must be conducted quarterly on all internal network systems.",
                    "Risk assessment results shall be reported to the executive risk committee within fourteen business days of assessment completion and shall include recommended mitigation actions.",
                ],
                table: Some(
                    "\
| Risk Level | Likelihood | Impact | Response Strategy | Review Frequency |\n\
| --- | --- | --- | --- | --- |\n\
| Critical | Very High | Severe | Immediate mitigation | Weekly |\n\
| High | High | Major | Mitigate within 30 days | Bi-weekly |\n\
| Medium | Moderate | Moderate | Mitigate within 90 days | Monthly |\n\
| Low | Low | Minor | Accept or mitigate within 180 days | Quarterly |\n\
| Informational | Very Low | Negligible | Accept | Annually |",
                ),
                h4_block: None,
            },
            Subsection {
                title: "Risk Mitigation and Treatment",
                body: "\
The organization shall develop and implement risk treatment plans for all risks that exceed the \
defined risk tolerance thresholds. Risk treatment options shall include mitigation through \
implementation of security controls, transfer through insurance or contractual arrangements, \
avoidance through elimination of the risk source, or acceptance with documented justification \
approved by the appropriate authority. Risk mitigation controls shall be selected from the \
NIST SP 800-53 control catalog and shall be tailored to the specific risk context. The \
effectiveness of implemented controls shall be monitored through ongoing assessment activities. \
The organization shall maintain a plan of action and milestones for all risks undergoing active \
mitigation that tracks implementation progress and expected completion dates.",
                requirements: &[
                    "Risk treatment plans shall be developed for all risks rated as Medium or higher and must identify specific controls, implementation timelines, and responsible parties.",
                    "Risk acceptance decisions shall be documented with justification and must be approved by the executive risk committee for risks rated High or Critical.",
                    "Security controls selected for risk mitigation shall be drawn from the NIST SP 800-53 control catalog and shall be tailored to address the specific risk scenario.",
                    "The organization shall maintain a plan of action and milestones for all active risk mitigations and must update the plan at least monthly to reflect implementation progress.",
                    "Risk mitigation effectiveness shall be evaluated within ninety days of control implementation and shall be reassessed annually thereafter.",
                ],
                table: None,
                h4_block: None,
            },
            Subsection {
                title: "Continuous Risk Monitoring",
                body: "\
The organization shall implement continuous monitoring processes to maintain awareness of \
information security risks, assess the effectiveness of security controls, and detect changes \
that may affect the security posture. Continuous monitoring activities shall include automated \
security control assessments, vulnerability management, threat intelligence analysis, and \
compliance monitoring. Security metrics shall be collected and reported to management on a \
monthly basis. Automated tools shall be deployed to continuously assess the compliance of \
information system configurations against approved security baselines. Deviations from the \
approved baseline shall be flagged for investigation and remediation within defined timeframes \
based on the severity of the deviation.",
                requirements: &[
                    "The organization shall implement automated continuous monitoring tools to assess security control effectiveness across all information systems in accordance with NIST SP 800-137.",
                    "Security configuration baselines shall be monitored continuously and must alert the security operations team within one hour of detecting deviations from approved configurations.",
                    "Threat intelligence feeds shall be integrated into monitoring systems and shall be reviewed daily by security analysts for indicators relevant to the organizational environment.",
                    "Security metrics shall be collected from all monitoring systems and must be reported to management monthly with trend analysis covering a minimum of twelve months.",
                    "The continuous monitoring strategy shall be reviewed and updated annually and shall incorporate lessons learned from incidents and assessments conducted during the previous period.",
                ],
                table: None,
                h4_block: None,
            },
            Subsection {
                title: "Third-Party Risk Management",
                body: "\
The organization shall assess and manage information security risks associated with third-party \
service providers, vendors, and business partners who access, process, store, or transmit \
organizational data. Third-party risk assessments shall be conducted prior to engagement and \
shall be repeated at least annually throughout the business relationship. Assessment scope shall \
include the third party's security controls, compliance certifications, incident response \
capabilities, and business continuity preparedness. Contracts with third parties shall include \
security requirements, audit rights, incident notification obligations, and data protection \
provisions. The organization shall maintain a third-party risk register and shall monitor \
identified risks through the normal risk management process.",
                requirements: &[
                    "Third-party risk assessments shall be completed prior to contract execution and shall evaluate the vendor's security posture against organizational security requirements.",
                    "All contracts with third-party service providers shall include provisions for security requirements, audit rights, breach notification within seventy-two hours, and data protection obligations.",
                    "Third-party security posture shall be reassessed at least annually and must be reassessed within thirty days of notification of a security incident affecting the third party.",
                    "The organization shall maintain a third-party risk register that documents all assessed risks and must track remediation activities to completion.",
                    "Critical third-party service providers shall provide annual SOC 2 Type II reports or equivalent independent security assessments. See Appendix B for the vendor assessment questionnaire.",
                ],
                table: None,
                h4_block: None,
            },
        ],
    },
    // ── 5. Physical Security ───────────────────────────────────────────
    Domain {
        title: "Physical Security",
        intro: "\
The organization shall implement physical and environmental security controls to protect \
information systems, personnel, and facilities from physical threats including unauthorized \
access, theft, damage, and natural disasters. Physical security controls shall be designed \
based on the results of a physical security risk assessment that considers the criticality of \
the assets housed within each facility, the geographic location, and the threat environment. \
The facilities management team shall coordinate with the information security team to ensure \
that physical security controls are integrated with logical security controls to provide \
defense in depth. Physical security incidents shall be reported and investigated using the \
same incident response procedures defined for information security incidents. Physical \
security controls shall be inspected at least quarterly and tested at least annually.",
        subsections: &[
            Subsection {
                title: "Facility Access Controls",
                body: "\
Access to organizational facilities shall be controlled through a combination of physical \
barriers, electronic access control systems, and personnel verification procedures. All \
facility entry points shall be equipped with electronic access control systems that log all \
access attempts including date, time, and identity of the individual. Access badges shall be \
uniquely assigned to individuals and shall include a photograph for visual verification by \
security personnel. Tailgating prevention measures shall be implemented at all controlled entry \
points. Emergency exits shall be equipped with alarms and shall be monitored by security cameras. \
The organization shall maintain a current access control list for each secure area and shall \
review access authorizations at least quarterly.",
                requirements: &[
                    "All facility entry points shall be equipped with electronic access control systems that log access attempts with date, time, and individual identity information.",
                    "Physical access badges shall be uniquely assigned to individuals and must include a photograph for visual verification by security personnel at controlled entry points.",
                    "Server rooms and data centers shall require multi-factor physical access including badge and biometric verification and must be monitored by security cameras at all times.",
                    "Visitor access shall require prior authorization from the hosting employee and must include escort by an authorized employee at all times within restricted areas.",
                    "Physical access logs shall be retained for a minimum of twelve months and must be reviewed monthly by the facilities security team for anomalous access patterns.",
                ],
                table: Some(
                    "\
| Area Classification | Access Control Type | Monitoring | Review Frequency |\n\
| --- | --- | --- | --- |\n\
| Public Areas | Open access | Camera | Monthly |\n\
| Office Areas | Badge access | Camera and logging | Quarterly |\n\
| Server Rooms | Badge plus biometric | Camera, logging, alarm | Monthly |\n\
| Data Centers | Badge, biometric, mantrap | 24x7 camera, logging, alarm | Weekly |\n\
| Executive Areas | Badge plus PIN | Camera and logging | Quarterly |",
                ),
                h4_block: None,
            },
            Subsection {
                title: "Environmental Controls",
                body: "\
The organization shall implement environmental controls to protect information systems from \
damage due to fire, flooding, temperature extremes, humidity, power fluctuations, and other \
environmental hazards. Environmental monitoring systems shall be deployed in all server rooms \
and data centers to continuously monitor temperature, humidity, water presence, and smoke. \
Automated alerts shall be generated when environmental conditions deviate from defined \
acceptable ranges. Fire suppression systems shall be installed in all areas housing information \
system equipment and shall use clean agent suppression to prevent damage to electronic equipment. \
Uninterruptible power supply systems shall be deployed for all critical information systems with \
sufficient capacity to support orderly shutdown procedures.",
                requirements: &[
                    "Environmental monitoring systems shall be deployed in all server rooms and data centers and must continuously monitor temperature, humidity, water presence, and smoke detection.",
                    "Temperature in server rooms shall be maintained between eighteen and twenty-seven degrees Celsius and must generate automated alerts when readings deviate by more than two degrees from the target range.",
                    "Fire suppression systems in data centers shall use clean agent suppression and must be tested annually by a certified fire protection engineer.",
                    "Uninterruptible power supply systems shall provide a minimum of thirty minutes of runtime for critical systems and must be load-tested quarterly.",
                    "Emergency power generators shall be tested monthly under load conditions and shall be capable of sustaining critical operations for a minimum of seventy-two hours.",
                ],
                table: None,
                h4_block: None,
            },
            Subsection {
                title: "Equipment Security",
                body: "\
The organization shall implement controls to protect information system equipment from theft, \
tampering, and unauthorized physical access throughout its lifecycle. Equipment containing \
sensitive data shall be physically secured using cable locks, locked cabinets, or locked rooms \
when not in active use. Portable devices including laptops, tablets, and mobile phones shall \
be encrypted and shall be equipped with remote wipe capabilities. Equipment disposal shall \
follow the data disposal procedures defined in Section 2.4 of this policy. Maintenance and \
repair of equipment shall be performed by authorized personnel only and shall be supervised \
when performed by external service providers.",
                requirements: &[
                    "All portable computing devices containing organizational data shall be encrypted using full-disk encryption and must be equipped with remote wipe capabilities.",
                    "Equipment containing Confidential or Restricted data shall be physically secured when unattended and shall be stored in locked areas outside of business hours.",
                    "Equipment maintenance shall be performed by authorized personnel only and must be supervised when performed by external service providers. See Section 7.4 for personnel clearance requirements.",
                    "The organization shall maintain an asset inventory of all information system equipment and must reconcile the inventory at least quarterly against physical inspections.",
                    "Equipment disposal shall follow the media sanitization procedures defined in Section 2.4 and must be documented in the asset management system.",
                ],
                table: None,
                h4_block: Some(
                    "\
#### Mobile Device Security\n\n\
Mobile devices used to access organizational information systems shall be enrolled in the \
organization's mobile device management system. The mobile device management system shall enforce \
security policies including device encryption, screen lock requirements, application restrictions, \
and remote wipe capabilities. Personal devices used under the bring-your-own-device program shall \
meet the same minimum security requirements as organization-owned devices and shall be subject \
to the same monitoring and management controls.",
                ),
            },
            Subsection {
                title: "Visitor Management",
                body: "\
The organization shall establish visitor management procedures to control and monitor the \
access of non-employees to organizational facilities. All visitors shall be required to register \
at the reception area and shall be issued a temporary visitor badge that clearly identifies them \
as non-employees. Visitor badges shall be returned upon departure and shall be deactivated at \
the end of each business day. Visitors shall be escorted by an authorized employee at all times \
while in restricted areas. Visitor logs shall be maintained and shall include the visitor name, \
organization, purpose of visit, hosting employee, and arrival and departure times. Delivery \
personnel shall be restricted to designated receiving areas and shall not enter secure areas \
without escort.",
                requirements: &[
                    "All visitors shall register at the reception area and shall be issued a temporary visitor badge that clearly identifies them as non-employees before entering the facility.",
                    "Visitors shall be escorted by an authorized employee at all times while in areas classified as Internal or above and must return visitor badges upon departure.",
                    "Visitor logs shall be maintained for a minimum of twelve months and must include visitor name, organization, purpose, host employee, and arrival and departure times.",
                    "Delivery and maintenance personnel shall be restricted to designated areas and must be escorted by authorized employees when access to restricted areas is required.",
                    "Temporary visitor badges shall be automatically deactivated at the end of each business day and must not provide access to server rooms, data centers, or other high-security areas.",
                ],
                table: None,
                h4_block: None,
            },
        ],
    },
    // ── 6. Network Security ────────────────────────────────────────────
    Domain {
        title: "Network Security",
        intro: "\
The organization shall implement network security controls to protect the confidentiality, \
integrity, and availability of information transmitted across organizational networks. Network \
security architecture shall implement defense-in-depth principles with multiple layers of \
controls including perimeter defenses, network segmentation, intrusion detection and prevention, \
and encrypted communications. The network security team shall maintain current documentation \
of the network architecture including all connections to external networks, network segmentation \
boundaries, and security control placement. Network security controls shall be tested at least \
annually through penetration testing and shall be continuously monitored for effectiveness. \
Changes to the network architecture shall be approved through the change management process \
and shall include a security impact assessment prior to implementation. The organization \
shall maintain network diagrams that accurately reflect the current state of the network.",
        subsections: &[
            Subsection {
                title: "Network Architecture and Segmentation",
                body: "\
The organization shall implement network segmentation to isolate systems and data based on \
their classification level and functional requirements. Network segments shall be defined \
based on the sensitivity of the data processed, the criticality of the systems, and the \
risk profile of the user population. Firewalls shall be deployed at all segment boundaries to \
control traffic flow between segments. A demilitarized zone shall be implemented for all \
internet-facing services. Management networks shall be isolated from production networks and \
shall be accessible only from authorized management workstations. Microsegmentation shall be \
implemented in data center environments to limit lateral movement in the event of a compromise.",
                requirements: &[
                    "The organization shall implement network segmentation to isolate systems based on data classification level, functional requirements, and risk profile as defined in the network architecture document.",
                    "Firewalls shall be deployed at all network segment boundaries and must enforce deny-by-default access control policies that permit only explicitly authorized traffic.",
                    "Internet-facing services shall be deployed in a demilitarized zone that is isolated from internal networks and must be protected by application-layer firewalls.",
                    "Management networks shall be isolated from production networks and must be accessible only from authorized management workstations using encrypted connections.",
                    "Network architecture diagrams shall be updated within five business days of any change and must accurately reflect all connections, segments, and security control placement.",
                ],
                table: Some(
                    "\
| Network Zone | Purpose | Access Level | Monitoring |\n\
| --- | --- | --- | --- |\n\
| External DMZ | Internet-facing services | Restricted | Full packet capture |\n\
| Internal DMZ | Internal shared services | Controlled | Flow logging |\n\
| Production | Business applications | Role-based | Full logging |\n\
| Management | Infrastructure management | Privileged | Enhanced logging |\n\
| Development | Non-production systems | Developer | Standard logging |\n\
| Guest | Visitor internet access | Internet only | Basic logging |",
                ),
                h4_block: None,
            },
            Subsection {
                title: "Firewall and Perimeter Security",
                body: "\
Firewall policies shall be configured based on the principle of least privilege, permitting \
only the minimum network traffic necessary for authorized business operations. All firewall \
rules shall be documented with a business justification, an owner, and an expiration date for \
temporary rules. Firewall rule sets shall be reviewed at least quarterly to identify and remove \
rules that are no longer necessary. Inbound traffic from the internet shall be restricted to \
explicitly authorized services on explicitly authorized ports. Outbound traffic filtering shall \
be implemented to prevent data exfiltration and unauthorized communications. The organization \
shall implement intrusion prevention capabilities at the network perimeter to detect and block \
known attack signatures and anomalous traffic patterns.",
                requirements: &[
                    "Firewall policies shall implement deny-by-default and permit-by-exception and must be configured to allow only the minimum traffic required for authorized business operations.",
                    "All firewall rules shall be documented with a business justification, rule owner, creation date, and must include an expiration date for all temporary access rules.",
                    "Firewall rule sets shall be reviewed at least quarterly and must remove or disable rules that are no longer required for business operations within five business days of identification.",
                    "Outbound traffic filtering shall be implemented to prevent unauthorized data exfiltration and must block communications to known malicious destinations updated from threat intelligence feeds.",
                    "Intrusion prevention systems shall be deployed at the network perimeter and must be configured to detect and block attack signatures with signatures updated at least daily.",
                ],
                table: None,
                h4_block: None,
            },
            Subsection {
                title: "Intrusion Detection and Prevention",
                body: "\
The organization shall deploy intrusion detection and prevention systems at strategic points \
within the network architecture to detect and respond to malicious activity. Network-based \
intrusion detection systems shall be deployed at all network segment boundaries and shall \
monitor traffic for signatures of known attacks, protocol anomalies, and behavioral indicators \
of compromise. Host-based intrusion detection systems shall be deployed on all critical servers \
and endpoints. Detection signatures shall be updated at least daily from vendor-provided and \
threat intelligence sources. Alerts generated by intrusion detection systems shall be triaged \
by the security operations center within one hour during business hours and within four hours \
outside business hours. False positive rates shall be monitored and tuned to maintain an \
actionable alert volume.",
                requirements: &[
                    "Network-based intrusion detection and prevention systems shall be deployed at all network segment boundaries and must monitor all traffic traversing those boundaries.",
                    "Host-based intrusion detection shall be deployed on all servers classified as critical and must detect unauthorized file modifications, privilege escalation attempts, and anomalous process execution.",
                    "Intrusion detection signatures and rules shall be updated at least daily from vendor and threat intelligence sources and must be validated before deployment to production sensors.",
                    "Intrusion detection alerts shall be triaged by the security operations center within one hour during business hours and must be escalated according to the incident severity matrix in Section 3.1.",
                    "The organization shall conduct quarterly tuning reviews of intrusion detection systems to optimize detection accuracy and must maintain a false positive rate below five percent.",
                ],
                table: None,
                h4_block: None,
            },
            Subsection {
                title: "Wireless Network Security",
                body: "\
Wireless networks shall be secured using WPA3 Enterprise or equivalent encryption and \
authentication mechanisms. Guest wireless networks shall be isolated from internal networks \
and shall provide internet access only. Wireless access points shall be centrally managed \
and shall enforce security policies including encryption requirements, authentication requirements, \
and session timeout values. Rogue access point detection shall be enabled on all wireless \
controllers and shall alert the security team when unauthorized wireless devices are detected. \
Wireless network security assessments shall be conducted at least annually and shall include \
testing for unauthorized access points, encryption weaknesses, and authentication bypass \
vulnerabilities. The organization shall maintain an inventory of all authorized wireless \
access points and shall disable wireless capabilities on devices where wireless connectivity \
is not required for business operations.",
                requirements: &[
                    "All organizational wireless networks shall use WPA3 Enterprise encryption and must authenticate users through the centralized authentication system using certificate-based or RADIUS authentication.",
                    "Guest wireless networks shall be logically isolated from all internal networks and must provide internet-only access with bandwidth limitations enforced at the wireless controller.",
                    "Rogue access point detection shall be enabled on all wireless controllers and must alert the security operations center within fifteen minutes of detecting an unauthorized wireless device.",
                    "Wireless network security assessments shall be conducted at least annually by qualified assessors and shall test for unauthorized access points, encryption weaknesses, and authentication vulnerabilities.",
                    "The organization shall maintain a current inventory of all authorized wireless access points and shall disable wireless capabilities on devices where wireless access is not required.",
                ],
                table: None,
                h4_block: None,
            },
        ],
    },
    // ── 7. Personnel Security ──────────────────────────────────────────
    Domain {
        title: "Personnel Security",
        intro: "\
The organization shall implement personnel security controls to ensure that all individuals \
with access to organizational information systems and data are trustworthy, qualified, and \
aware of their security responsibilities. Personnel security controls shall apply to all \
employees, contractors, temporary workers, and third-party personnel from the time of initial \
engagement through separation from the organization. The human resources department shall \
coordinate with the information security team to integrate security requirements into all \
personnel management processes including recruitment, onboarding, performance management, and \
separation. Background investigation requirements shall be proportional to the sensitivity of \
the position and the level of access to organizational data. All personnel security incidents \
shall be reported and investigated through the incident response process.",
        subsections: &[
            Subsection {
                title: "Background Screening and Verification",
                body: "\
The organization shall conduct background screening for all personnel prior to granting \
access to organizational information systems. Background screening requirements shall be \
determined based on the sensitivity level of the position as defined in the position \
classification guide. Screening shall include at a minimum identity verification, criminal \
history check, and verification of employment history and educational credentials. Positions \
with access to Confidential or Restricted data shall require additional screening including \
credit history checks and professional reference verification. Background screening shall be \
repeated at least every five years for positions with access to Restricted data. The organization \
shall maintain records of all background screening activities in accordance with applicable \
privacy regulations.",
                requirements: &[
                    "Background screening shall be completed for all personnel before granting access to organizational information systems and must include identity verification and criminal history checks.",
                    "Positions with access to Confidential or Restricted data shall require enhanced background screening including credit history checks and must include professional reference verification.",
                    "Background screening shall be repeated at least every five years for positions with access to Restricted data and must be repeated within thirty days of a role change to a higher sensitivity position.",
                    "The organization shall maintain background screening records in compliance with applicable privacy regulations and must retain records for the duration of employment plus seven years.",
                    "Adverse screening results shall be reviewed by the human resources director in consultation with legal counsel and must be adjudicated before access to information systems is granted.",
                ],
                table: Some(
                    "\
| Position Sensitivity | Screening Level | Components | Rescreening |\n\
| --- | --- | --- | --- |\n\
| Low | Basic | Identity, criminal check | None |\n\
| Moderate | Standard | Basic plus employment history | Every 5 years |\n\
| High | Enhanced | Standard plus credit, references | Every 3 years |\n\
| Critical | Comprehensive | Enhanced plus polygraph option | Every 2 years |",
                ),
                h4_block: None,
            },
            Subsection {
                title: "Security Awareness Training",
                body: "\
The organization shall provide security awareness training to all personnel upon initial \
access to information systems and at least annually thereafter. Training content shall address \
current threats, organizational security policies, acceptable use requirements, incident \
reporting procedures, and data handling requirements. Specialized training shall be provided \
to personnel with elevated responsibilities including system administrators, security analysts, \
and incident response team members. Training effectiveness shall be measured through assessments \
and phishing simulation exercises. Personnel who fail to complete required training within the \
designated timeframe shall have their access to information systems suspended until training \
is completed. The security awareness program shall be updated at least annually to address \
emerging threats and shall incorporate lessons learned from recent incidents.",
                requirements: &[
                    "All personnel shall complete security awareness training within thirty days of initial access to information systems and must complete annual refresher training within the designated enrollment period.",
                    "Security awareness training shall cover current threats, organizational security policies, incident reporting procedures, and data handling requirements specific to the individual's role.",
                    "Phishing simulation exercises shall be conducted at least quarterly and must achieve a click rate below five percent to demonstrate training effectiveness.",
                    "Personnel who fail to complete required training within the designated timeframe shall have their system access suspended and shall not regain access until training is completed.",
                    "Specialized security training shall be provided to system administrators, security analysts, and incident response team members and must include hands-on exercises relevant to their responsibilities.",
                ],
                table: None,
                h4_block: None,
            },
            Subsection {
                title: "Acceptable Use Policy",
                body: "\
All personnel shall comply with the acceptable use policy as a condition of their access to \
organizational information systems and networks. The acceptable use policy defines permitted \
and prohibited uses of organizational technology resources and establishes expectations for \
professional conduct in the digital environment. Personnel shall acknowledge the acceptable \
use policy upon initial access and annually thereafter. Monitoring of acceptable use compliance \
shall be conducted through automated tools and periodic reviews. Violations of the acceptable \
use policy shall be addressed through the disciplinary process and may result in suspension or \
revocation of access privileges depending on the severity and frequency of the violation.",
                requirements: &[
                    "All personnel shall acknowledge the acceptable use policy before receiving access to organizational systems and must re-acknowledge the policy annually.",
                    "Organizational information systems shall be used only for authorized business purposes and must not be used for activities that are illegal, offensive, or contrary to organizational interests.",
                    "The organization shall monitor information system usage for compliance with the acceptable use policy and shall implement automated tools to detect policy violations.",
                    "Personnel shall not install unauthorized software on organizational devices and must obtain approval from the information technology department before installing any software not on the approved list.",
                    "Violations of the acceptable use policy shall be documented and addressed through the disciplinary process with penalties proportional to the severity and frequency of the violation.",
                ],
                table: None,
                h4_block: None,
            },
            Subsection {
                title: "Personnel Separation Procedures",
                body: "\
The organization shall implement personnel separation procedures to ensure that access to \
organizational information systems and data is promptly revoked when personnel depart the \
organization or change roles. Separation procedures shall be initiated immediately upon \
notification of resignation, termination, or role change. The separation checklist shall \
include revocation of all system access, return of organizational equipment and credentials, \
transfer of data ownership, and exit interview with security briefing. For involuntary \
separations involving personnel with access to Confidential or Restricted data, the security \
team shall be notified prior to the separation meeting to ensure that access is revoked \
simultaneously with notification. Knowledge transfer activities shall be completed before \
the effective separation date whenever possible.",
                requirements: &[
                    "All system access shall be revoked within four hours of notification of employment termination and must be revoked simultaneously with notification for involuntary separations of high-sensitivity personnel.",
                    "The separation checklist shall be completed for all departing personnel and must include return of all organizational equipment, badges, credentials, and cryptographic material.",
                    "Exit interviews shall include a security briefing reminding departing personnel of their ongoing obligations regarding confidential information and intellectual property.",
                    "Data and account ownership shall be transferred to designated successors before the separation date and must be completed within five business days of separation for unplanned departures.",
                    "The human resources department shall notify the identity management team of all personnel separations within one business day and shall provide the effective separation date for advance processing.",
                ],
                table: None,
                h4_block: None,
            },
        ],
    },
    // ── 8. Business Continuity ─────────────────────────────────────────
    Domain {
        title: "Business Continuity and Disaster Recovery",
        intro: "\
The organization shall establish and maintain a business continuity and disaster recovery program \
to ensure the continued delivery of critical business functions during and after disruptive events. \
The program shall be aligned with ISO 22301 and shall address a range of threat scenarios including \
natural disasters, infrastructure failures, cyber attacks, and pandemic events. Business continuity \
plans shall be developed for each critical business function based on the results of a business \
impact analysis. Recovery strategies shall ensure that critical systems and processes can be \
restored within the recovery time objectives and recovery point objectives defined for each \
function. The business continuity program shall be tested at least annually through tabletop \
exercises, functional exercises, or full-scale tests. The business continuity coordinator shall \
maintain oversight of the program and shall report program status to senior management quarterly.",
        subsections: &[
            Subsection {
                title: "Business Impact Analysis",
                body: "\
The organization shall conduct a business impact analysis at least annually to identify critical \
business functions, determine the impact of disruption over time, and establish recovery \
priorities. The business impact analysis shall assess both quantitative impacts including revenue \
loss, regulatory penalties, and contractual damages, and qualitative impacts including \
reputational harm, customer satisfaction, and employee morale. Recovery time objectives and \
recovery point objectives shall be defined for each critical business function based on the \
maximum tolerable downtime and maximum tolerable data loss. Dependencies between business \
functions, information systems, and third-party services shall be mapped to identify single \
points of failure and cascading failure scenarios.",
                requirements: &[
                    "A business impact analysis shall be conducted at least annually and must be updated within thirty days of any significant change to critical business functions or supporting infrastructure.",
                    "Recovery time objectives and recovery point objectives shall be defined for all critical business functions and must be approved by the business function owner and senior management.",
                    "The business impact analysis shall identify and document all dependencies between critical business functions, information systems, and third-party service providers.",
                    "Single points of failure identified during the business impact analysis shall be documented in the risk register and must have mitigation strategies developed within sixty calendar days.",
                    "The business impact analysis shall assess both quantitative impacts including financial losses and qualitative impacts including reputational damage for each critical business function.",
                ],
                table: Some(
                    "\
| Business Function | RTO | RPO | Priority | Dependencies |\n\
| --- | --- | --- | --- | --- |\n\
| Payment Processing | 1 hour | 0 minutes | Critical | Database, Network, Power |\n\
| Customer Portal | 4 hours | 1 hour | High | Web servers, Database |\n\
| Email Services | 8 hours | 4 hours | High | Mail servers, DNS |\n\
| Internal Applications | 24 hours | 8 hours | Medium | App servers, Database |\n\
| Development Systems | 72 hours | 24 hours | Low | Dev servers, Source control |",
                ),
                h4_block: None,
            },
            Subsection {
                title: "Recovery Planning and Testing",
                body: "\
The organization shall develop and maintain recovery plans for all critical business functions \
that specify the procedures, resources, and responsibilities required to restore operations \
within defined recovery time objectives. Recovery plans shall address infrastructure recovery, \
application recovery, data recovery, and personnel recovery for each critical function. Plans \
shall include step-by-step procedures that can be executed by personnel who may not be familiar \
with the systems being recovered. Recovery plans shall be tested at least annually using \
tabletop exercises, functional tests, or full-scale tests. Test results shall be documented \
and shall include an assessment of whether recovery time objectives were met. Identified gaps \
shall be addressed through plan updates within thirty days of the test.",
                requirements: &[
                    "Recovery plans shall be developed for all critical business functions and must specify step-by-step procedures for restoring operations within the defined recovery time objectives.",
                    "Recovery plans shall be tested at least annually and must include at least one functional test that validates actual recovery procedures against a simulated disruption scenario.",
                    "Recovery plan test results shall be documented and must include an assessment of whether recovery time and recovery point objectives were achieved during the test.",
                    "Gaps identified during recovery plan testing shall be addressed through plan updates within thirty days and must be retested within ninety days to verify the effectiveness of corrections.",
                    "Recovery plans shall be accessible from an offsite location and must be available in both electronic and printed formats to ensure access during infrastructure failures.",
                ],
                table: None,
                h4_block: None,
            },
            Subsection {
                title: "Backup and Restoration Procedures",
                body: "\
The organization shall implement backup procedures to protect against data loss and to support \
recovery operations following a disruptive event. Backup schedules shall be defined based on \
the recovery point objectives established for each business function and data classification. \
Full backups shall be performed at least weekly for all critical systems, with incremental \
backups performed daily. Backup media shall be stored at a secure offsite location that is \
geographically separated from the primary facility by at least one hundred kilometers. \
Backup integrity shall be verified through regular restoration tests. The organization shall \
implement immutable backup copies for data classified as Restricted to protect against \
ransomware and malicious data destruction.",
                requirements: &[
                    "Full backups shall be performed at least weekly for all critical systems and must be supplemented by daily incremental backups to meet recovery point objectives.",
                    "Backup media shall be stored at a secure offsite facility geographically separated from the primary site by at least one hundred kilometers and must be encrypted using approved algorithms.",
                    "Backup restoration tests shall be conducted at least quarterly for critical systems and must verify that restored data is complete, accurate, and usable within the recovery point objective window.",
                    "Immutable backup copies shall be maintained for all data classified as Restricted and must be retained for a minimum of ninety days to protect against ransomware and malicious destruction.",
                    "Backup failures shall be reported to the infrastructure team within one hour and must be resolved within twenty-four hours with a successful backup verified before the issue is closed.",
                ],
                table: None,
                h4_block: None,
            },
            Subsection {
                title: "Emergency Communications",
                body: "\
The organization shall establish emergency communication procedures to ensure timely and \
accurate information exchange during disruptive events. The emergency communication plan shall \
identify primary and alternate communication channels, notification procedures for key \
stakeholders, and escalation paths based on event severity. The communication plan shall include \
provisions for communication when primary communication systems are unavailable. A mass \
notification system shall be deployed to enable rapid notification of all personnel during \
emergencies. Emergency communication procedures shall be tested at least semi-annually and \
shall include testing of alternate communication channels. Contact information for all key \
personnel shall be maintained in the emergency communication plan and shall be updated at \
least quarterly.",
                requirements: &[
                    "The organization shall maintain an emergency communication plan that identifies primary and alternate communication channels for use during disruptive events.",
                    "A mass notification system shall be deployed and must be capable of reaching all personnel within thirty minutes through multiple communication channels including phone, text, and email.",
                    "Emergency communication procedures shall be tested at least semi-annually and must include testing of alternate communication channels to verify availability during primary system failures.",
                    "Contact information for all key personnel and external stakeholders shall be maintained in the emergency communication plan and must be verified and updated at least quarterly.",
                    "Emergency communication templates shall be pre-approved by legal counsel and public relations and must be readily accessible to authorized communicators during an event.",
                ],
                table: None,
                h4_block: None,
            },
        ],
    },
    // ── 9. Audit and Accountability ────────────────────────────────────
    Domain {
        title: "Audit and Accountability",
        intro: "\
The organization shall implement audit and accountability controls to create, protect, and \
retain information system audit records to the extent needed to enable monitoring, analysis, \
investigation, and reporting of unauthorized or inappropriate information system activity. Audit \
records shall be generated for events including user authentication, privilege changes, data \
access, system configuration changes, and security-relevant administrative actions. Audit \
records shall contain sufficient detail to establish what events occurred, when they occurred, \
where they occurred, the source of the events, and the outcome of the events. The organization \
shall protect audit records from unauthorized access, modification, and deletion to ensure the \
integrity and availability of audit data for forensic and compliance purposes. Audit logging \
shall be implemented as specified in NIST SP 800-92.",
        subsections: &[
            Subsection {
                title: "Audit Logging Requirements",
                body: "\
All information systems shall generate audit records for security-relevant events as defined \
in the audit logging standard. Audit records shall include at a minimum the event type, event \
timestamp, event source, event outcome, and the identity of the user or process associated \
with the event. System clocks shall be synchronized to an authoritative time source using \
Network Time Protocol to ensure accurate and consistent timestamps across all systems. Audit \
logging shall be configured to capture events at both the operating system level and the \
application level. Logging configurations shall be protected from unauthorized modification \
through access controls and integrity monitoring. Systems that fail to generate audit records \
shall generate an alert to the security operations center.",
                requirements: &[
                    "All information systems shall generate audit records for security-relevant events including authentication, authorization, data access, configuration changes, and administrative actions.",
                    "Audit records shall include the event type, timestamp, source, outcome, and user or process identity and must use a consistent format across all systems as defined in the audit logging standard.",
                    "System clocks shall be synchronized to an authoritative Network Time Protocol source and must maintain accuracy within one second to ensure consistent timestamps across all systems.",
                    "Logging configurations shall be protected from unauthorized modification and must generate alerts to the security operations center if logging is disabled or modified on any system.",
                    "Application-level audit logging shall capture business transaction events and must include sufficient detail to reconstruct the complete transaction for forensic analysis.",
                ],
                table: Some(
                    "\
| Event Category | Examples | Retention | Alert Threshold |\n\
| --- | --- | --- | --- |\n\
| Authentication | Login, logout, failed attempts | 12 months | 5 failures in 5 minutes |\n\
| Authorization | Privilege changes, access denials | 12 months | Any privilege escalation |\n\
| Data Access | Read, write, delete operations | 24 months | Bulk data export |\n\
| Configuration | System setting changes | 36 months | Any critical system change |\n\
| Administrative | User management, policy changes | 36 months | Any action on admin accounts |",
                ),
                h4_block: None,
            },
            Subsection {
                title: "Log Management and Protection",
                body: "\
Audit logs shall be collected, aggregated, and stored in a centralized log management system that \
provides tamper-evident storage, indexed search, and automated analysis capabilities. Log data \
shall be transmitted from source systems to the centralized log management system in near real \
time using encrypted transport. Access to the log management system shall be restricted to \
authorized security personnel and shall require multi-factor authentication. Log data shall be \
retained for the periods specified in the retention schedule and shall be archived to long-term \
storage in a format that preserves the integrity and searchability of the original records. The \
organization shall implement integrity verification mechanisms to detect any unauthorized \
modification of stored log data.",
                requirements: &[
                    "Audit logs shall be transmitted to the centralized log management system within five minutes of generation and must use encrypted transport to protect log data in transit.",
                    "Access to the log management system shall be restricted to authorized security personnel and must require multi-factor authentication for all access. See Section 1.2 for authentication requirements.",
                    "Log data integrity shall be protected through cryptographic hash verification and must generate alerts if any modification to stored log records is detected.",
                    "Log retention periods shall comply with the data retention schedule and must retain security event logs for a minimum of twelve months online and thirty-six months in archive storage.",
                    "The log management system shall provide indexed search capabilities and must support queries that return results within sixty seconds for searches spanning up to twelve months of data.",
                ],
                table: None,
                h4_block: None,
            },
            Subsection {
                title: "Audit Review and Analysis",
                body: "\
The organization shall review and analyze audit records on a regular basis to identify anomalous \
or suspicious activity, potential security incidents, and policy violations. Automated analysis \
tools shall be deployed to correlate events across multiple systems and to identify patterns \
that may indicate a security threat. Security analysts shall review automated analysis results \
daily and shall investigate alerts that exceed defined thresholds. Periodic manual reviews \
shall be conducted to identify trends and patterns that automated tools may not detect. \
Audit review findings shall be documented and shall be escalated through the incident response \
process when potential security incidents are identified.",
                requirements: &[
                    "Automated log analysis and correlation shall be performed continuously and must generate alerts for events matching predefined threat indicators and behavioral anomaly patterns.",
                    "Security analysts shall review automated analysis results daily and must investigate all alerts classified as High or Critical within four hours of generation.",
                    "Manual audit reviews shall be conducted monthly for critical systems and must include analysis of access patterns, privilege usage, and data transfer activities.",
                    "Audit review findings indicating potential security incidents shall be escalated through the incident response process within one hour and must be documented in the incident tracking system.",
                    "Quarterly trend analysis shall be performed on audit data and must be presented to management with recommendations for security control improvements based on observed patterns.",
                ],
                table: None,
                h4_block: None,
            },
            Subsection {
                title: "Compliance Reporting",
                body: "\
The organization shall generate compliance reports from audit data to demonstrate adherence \
to regulatory requirements, contractual obligations, and internal policies. Compliance reports \
shall be generated at the frequency required by each applicable regulation or standard and \
shall include evidence of control effectiveness. The compliance team shall coordinate with the \
information security team to ensure that audit data collection supports all compliance reporting \
requirements. Gaps identified through compliance reporting shall be tracked through the \
corrective action process and shall be remediated within the timeframes specified by the \
applicable regulation. Compliance reports shall be retained for the period specified by the \
applicable regulation or for a minimum of seven years, whichever is longer.",
                requirements: &[
                    "Compliance reports shall be generated from audit data at the frequency required by each applicable regulation and must include evidence of control effectiveness.",
                    "The compliance team shall maintain a regulatory calendar that tracks all reporting deadlines and must submit reports at least five business days before the regulatory deadline.",
                    "Gaps identified through compliance reporting shall be documented in the corrective action system and must include a remediation plan with specific actions, responsible parties, and target dates.",
                    "Compliance report templates shall be reviewed annually and must be updated to reflect changes in regulatory requirements within thirty days of notification.",
                    "Compliance reports shall be retained for the period specified by the applicable regulation or seven years, whichever is longer, and must be stored in a format that ensures long-term accessibility.",
                ],
                table: None,
                h4_block: None,
            },
        ],
    },
    // ── 10. Compliance and Regulatory ──────────────────────────────────
    Domain {
        title: "Compliance and Regulatory Management",
        intro: "\
The organization shall establish and maintain a compliance management program to ensure \
adherence to all applicable laws, regulations, standards, and contractual obligations related \
to information security and data protection. The compliance program shall be led by the Chief \
Compliance Officer in coordination with the Chief Information Security Officer, legal counsel, \
and business unit leaders. The organization shall maintain a current inventory of all applicable \
regulatory requirements and shall map these requirements to organizational policies, controls, \
and procedures. Compliance assessments shall be conducted at least annually for each applicable \
regulation. The organization shall establish a corrective action process to address identified \
compliance gaps and shall track remediation activities to completion. Regulatory changes shall \
be monitored continuously and shall be incorporated into the compliance program within ninety \
days of effective date.",
        subsections: &[
            Subsection {
                title: "Regulatory Requirements Tracking",
                body: "\
The organization shall maintain a comprehensive registry of all applicable regulatory \
requirements related to information security and data protection. The registry shall include \
the regulation name, jurisdiction, applicability determination, key requirements, compliance \
status, and responsible owner. New regulatory requirements shall be assessed within thirty \
days of identification and shall be incorporated into the compliance program if applicable. \
The compliance team shall monitor regulatory developments through subscriptions to regulatory \
agencies, legal advisories, and industry groups. Impact assessments shall be conducted for \
all significant regulatory changes to determine the effect on organizational policies and \
controls.",
                requirements: &[
                    "The organization shall maintain a regulatory requirements registry that documents all applicable information security and data protection regulations with their key requirements and compliance status.",
                    "New regulatory requirements shall be assessed for applicability within thirty calendar days of identification and must be incorporated into the compliance program within ninety days if applicable.",
                    "The compliance team shall monitor regulatory developments continuously through subscriptions to regulatory agencies and must distribute regulatory change notifications to affected business units within five business days.",
                    "Impact assessments shall be conducted for all significant regulatory changes and must identify required modifications to organizational policies, controls, and procedures.",
                    "The regulatory requirements registry shall be reviewed at least semi-annually and must be updated to reflect changes in the regulatory landscape and organizational operations.",
                ],
                table: Some(
                    "\
| Regulation | Jurisdiction | Applicability | Key Requirements | Review Date |\n\
| --- | --- | --- | --- | --- |\n\
| FISMA | United States Federal | All federal data | Risk management, continuous monitoring | Annually |\n\
| HIPAA | United States | Health data | Privacy, security, breach notification | Annually |\n\
| GDPR | European Union | EU resident data | Consent, data rights, DPO | Annually |\n\
| PCI DSS | Global | Payment card data | Network security, encryption, access | Quarterly |\n\
| SOX | United States | Financial data | Internal controls, audit trails | Annually |",
                ),
                h4_block: None,
            },
            Subsection {
                title: "Internal Policy Compliance",
                body: "\
The organization shall assess compliance with internal information security policies on a \
regular basis through automated tools, manual reviews, and self-assessments. Policy compliance \
assessments shall cover all policy domains including access control, data protection, incident \
response, risk management, and personnel security. Assessment findings shall be documented \
and shall identify specific instances of non-compliance, the associated risk, and recommended \
remediation actions. Business unit leaders shall be responsible for ensuring that remediation \
actions are implemented within the agreed timelines. The compliance team shall track remediation \
progress and shall escalate overdue items to senior management.",
                requirements: &[
                    "Internal policy compliance assessments shall be conducted at least annually for each policy domain and must include a representative sample of systems, processes, and personnel.",
                    "Automated compliance scanning tools shall be deployed to continuously monitor compliance with technical security policies and must report violations to the compliance team daily.",
                    "Non-compliance findings shall be documented with the specific policy requirement violated, the associated risk level, and must include a remediation plan with assigned owner and target date.",
                    "Remediation of compliance findings rated High or Critical shall be completed within thirty calendar days and must be verified by the compliance team before closure.",
                    "Business unit leaders shall certify compliance with all applicable security policies annually and must acknowledge responsibility for any identified gaps within their area of authority.",
                ],
                table: None,
                h4_block: None,
            },
            Subsection {
                title: "Independent Assessments and Audits",
                body: "\
The organization shall engage independent assessors to conduct security assessments at least \
annually to evaluate the effectiveness of the information security program. Independent \
assessments shall be performed by qualified firms with relevant certifications including CISA, \
CISSP, or equivalent professional credentials. Assessment scope shall include technical \
vulnerability assessment, security control testing, policy review, and process evaluation. \
Assessment findings shall be reported to senior management and the board of directors. The \
organization shall develop corrective action plans for all findings and shall track remediation \
to completion. The organization shall also facilitate assessments required by regulators and \
business partners in a timely manner.",
                requirements: &[
                    "Independent security assessments shall be conducted at least annually by qualified firms with relevant certifications and must include technical vulnerability assessment and control testing.",
                    "Assessment scope shall include all critical information systems and must cover technical controls, administrative controls, and physical security controls.",
                    "Assessment findings shall be reported to senior management within thirty calendar days of assessment completion and must include risk ratings and recommended remediation actions.",
                    "Corrective action plans shall be developed for all assessment findings within fourteen calendar days of report delivery and must specify responsible parties, target dates, and verification methods.",
                    "The organization shall facilitate regulatory and business partner assessments within fifteen business days of request and must provide access to relevant documentation, systems, and personnel.",
                ],
                table: None,
                h4_block: None,
            },
            Subsection {
                title: "Remediation and Continuous Improvement",
                body: "\
The organization shall implement a structured remediation process to address compliance findings, \
audit observations, and assessment recommendations in a timely and effective manner. Remediation \
activities shall be tracked through the corrective action management system from identification \
through verification and closure. Remediation priorities shall be based on the risk rating of \
the finding and the regulatory or contractual timeline for compliance. Root cause analysis shall \
be performed for recurring findings to identify systemic issues that require structural \
improvements. The information security program shall be continuously improved through the \
integration of lessons learned from incidents, assessments, audits, and regulatory changes \
into updated policies, procedures, and controls.",
                requirements: &[
                    "All compliance findings shall be tracked through the corrective action management system and must include finding description, risk rating, remediation plan, responsible party, and target completion date.",
                    "Remediation of Critical findings shall be completed within fourteen calendar days, High findings within thirty days, and Medium findings within ninety days of identification.",
                    "Root cause analysis shall be performed for all recurring findings and must result in structural improvements that address the underlying cause rather than individual symptoms.",
                    "The corrective action management system shall generate automated notifications for approaching deadlines and must escalate overdue items to the next management level after five business days.",
                    "The information security program shall conduct an annual maturity assessment and must demonstrate measurable improvement against the previous year's assessment results.",
                ],
                table: None,
                h4_block: Some(
                    "\
#### Metrics and Key Performance Indicators\n\n\
The organization shall define and track key performance indicators for the information security \
program including mean time to detect security incidents, mean time to respond to incidents, \
percentage of systems compliant with security baselines, percentage of personnel current on \
security awareness training, and number of open audit findings by age and severity. Metrics \
shall be reported to senior management monthly and to the board of directors quarterly. Trend \
analysis shall be performed to identify areas requiring additional investment or attention. \
Target values for each metric shall be established annually and shall be aligned with the \
organization's risk appetite and strategic objectives.",
                ),
            },
        ],
    },
];

// ─── NIST Control References (deterministic, cycled for citations) ──────

const NIST_REFERENCES: &[&str] = &[
    "NIST SP 800-53 AC-2",
    "NIST SP 800-53 AC-3",
    "NIST SP 800-53 AC-6",
    "NIST SP 800-53 AC-17",
    "NIST SP 800-53 AT-2",
    "NIST SP 800-53 AU-2",
    "NIST SP 800-53 AU-6",
    "NIST SP 800-53 CA-7",
    "NIST SP 800-53 CM-6",
    "NIST SP 800-53 CP-9",
    "NIST SP 800-53 IA-2",
    "NIST SP 800-53 IA-5",
    "NIST SP 800-53 IR-4",
    "NIST SP 800-53 IR-6",
    "NIST SP 800-53 MA-4",
    "NIST SP 800-53 MP-6",
    "NIST SP 800-53 PE-3",
    "NIST SP 800-53 PL-2",
    "NIST SP 800-53 PM-9",
    "NIST SP 800-53 RA-5",
    "NIST SP 800-53 SA-9",
    "NIST SP 800-53 SC-7",
    "NIST SP 800-53 SC-13",
    "NIST SP 800-53 SC-28",
    "NIST SP 800-53 SI-4",
    "ISO 27001 A.9.2",
    "ISO 27001 A.12.4",
    "ISO 27001 A.14.1",
    "RFC 6238",
    "FIPS 140-3",
];

// ─── Generator Function ─────────────────────────────────────────────────

/// Generate a deterministic 50-page synthetic Markdown policy document.
///
/// Produces ~25,000 words / ~150,000 characters of Markdown with:
/// - YAML frontmatter (title, version, author, date)
/// - 10 H2 sections (Access Control, Data Protection, etc.)
/// - ~40 H3 subsections (3-5 per H2)
/// - ~200 numbered policy requirements (normative language)
/// - ~20 compound statements ("must X and must Y")
/// - ~30 citations/references ("[NIST SP 800-53 AC-2]")
/// - ~10 tables (role-responsibility matrices)
///
/// # Determinism
/// This function uses NO randomness, NO system time, NO RNG.
/// Two calls produce byte-identical output.
#[must_use]
pub fn generate_synthetic_policy() -> String {
    let mut doc = String::with_capacity(200_000);

    // ── YAML Frontmatter ──
    doc.push_str("---\n");
    doc.push_str("title: \"Comprehensive Information Security Policy\"\n");
    doc.push_str("version: \"1.0.0\"\n");
    doc.push_str("author: \"Policy Division\"\n");
    doc.push_str("date: \"2026-01-01\"\n");
    doc.push_str("---\n\n");

    // ── H1 Title and Executive Summary ──
    doc.push_str("# Comprehensive Information Security Policy\n\n");
    doc.push_str(EXECUTIVE_SUMMARY);
    doc.push_str("\n\n");

    // ── Generate Each Domain (H2) ──
    let mut nist_idx: usize = 0;
    for (domain_idx, domain) in DOMAINS.iter().enumerate() {
        let domain_num = domain_idx + 1;

        // H2 heading
        let _ = write!(doc, "## {}. {}\n\n", domain_num, domain.title);

        // Domain intro
        doc.push_str(domain.intro);
        doc.push_str("\n\n");

        // Scope and applicability paragraph (template-generated per domain)
        write_scope_paragraph(&mut doc, domain.title, domain_num);

        // Generate subsections
        for (sub_idx, sub) in domain.subsections.iter().enumerate() {
            let sub_num = sub_idx + 1;

            // H3 heading
            let _ = write!(doc, "### {}.{}. {}\n\n", domain_num, sub_num, sub.title);

            // Subsection body
            doc.push_str(sub.body);
            doc.push_str("\n\n");

            // Numbered requirements
            for (req_idx, req) in sub.requirements.iter().enumerate() {
                let _ = writeln!(doc, "{}. {}", req_idx + 1, req);
            }
            doc.push('\n');

            // Table (if present)
            if let Some(table) = sub.table {
                doc.push_str(table);
                doc.push_str("\n\n");
            }

            // H4 block (if present)
            if let Some(h4) = sub.h4_block {
                doc.push_str(h4);
                doc.push_str("\n\n");
            }

            // Supplementary guidance (template-generated per subsection)
            write_supplementary_guidance(
                &mut doc,
                domain.title,
                sub.title,
                domain_num,
                sub_num,
                NIST_REFERENCES[nist_idx % NIST_REFERENCES.len()],
            );
            nist_idx += 1;
        }
    }

    doc
}

const EXECUTIVE_SUMMARY: &str = "\
This document establishes the comprehensive information security policy for the \
organization. It defines the security requirements, controls, and procedures that all \
personnel must follow to protect organizational information assets. This policy applies \
to all employees, contractors, temporary workers, and third-party personnel who access, \
process, store, or transmit organizational information. Compliance with this policy is \
mandatory and subject to regular audit and review.\n\n\
The policy is organized into ten domains covering access control, data protection, incident \
response, risk management, physical security, network security, personnel security, business \
continuity, audit and accountability, and compliance management. Each domain contains specific \
requirements that have been aligned with NIST SP 800-53 controls, ISO 27001 requirements, and \
industry best practices. The organization is committed to maintaining a security posture \
that protects the confidentiality, integrity, and availability of all information assets \
while enabling the business to achieve its strategic objectives.\n\n\
This policy shall be reviewed at least annually by the Chief Information Security Officer and \
updated as necessary to reflect changes in the threat landscape, regulatory environment, \
organizational structure, and technology infrastructure. All revisions shall be approved by \
the executive security committee before publication. The effective date of this policy version \
is January 1, 2026. Previous versions are superseded upon the effective date of this revision. \
Questions regarding the interpretation or application of this policy should be directed to the \
information security governance team at security-policy@organization.example.com.\n\n\
The organization recognizes that information security is a shared responsibility that requires \
the commitment and cooperation of all stakeholders. Senior management is committed to providing \
the resources necessary to implement and maintain an effective information security program. \
Business unit leaders are responsible for ensuring that their teams understand and comply with \
the requirements in this policy. Individual personnel are responsible for following the security \
procedures applicable to their roles and reporting any suspected security incidents or policy \
violations through the designated reporting channels.";

fn write_scope_paragraph(doc: &mut String, domain_title: &str, domain_num: usize) {
    let domain_lower = domain_title.to_lowercase();
    let _ = write!(
        doc,
        "The requirements in this section apply to all organizational units that manage, \
operate, or interact with information systems and data subject to {domain_lower} controls. \
Compliance with Section {domain_num} requirements shall be assessed during the annual security \
assessment and during any ad hoc assessments triggered by significant changes to the \
{domain_lower} environment. Business unit leaders shall designate a {domain_lower} coordinator \
responsible for ensuring implementation of these requirements within their respective areas. \
The designated coordinator shall report compliance status to the information security governance \
team on a quarterly basis and shall escalate any significant gaps or resource constraints \
that may impede timely compliance. Exceptions to the requirements in this section shall be \
documented using the standard exception request process and shall require approval from the \
Chief Information Security Officer. Approved exceptions shall be reviewed at least annually \
and shall be revoked when the conditions justifying the exception no longer apply.\n\n",
    );
}

fn write_supplementary_guidance(
    doc: &mut String,
    domain_title: &str,
    sub_title: &str,
    domain_num: usize,
    sub_num: usize,
    nist_ref: &str,
) {
    let domain_lower = domain_title.to_lowercase();
    let sub_lower = sub_title.to_lowercase();
    let _ = write!(
        doc,
        "**Supplementary Guidance for Section {domain_num}.{sub_num}**: \
Organizations implementing {sub_lower} controls should consider the specific operational \
context and risk profile of their environment when determining the appropriate level of \
rigor for each requirement. The requirements in this section are aligned with {nist_ref} \
and reflect industry best practices for {domain_lower} in enterprise environments. \
Organizations operating in highly regulated industries may need to implement additional \
controls beyond those specified in this section to meet sector-specific requirements. \
The implementation of {sub_lower} controls should be coordinated with related controls \
in other sections of this policy to ensure a consistent and comprehensive security \
posture. Regular testing and validation of implemented controls is essential to ensure \
continued effectiveness as the threat landscape evolves. Documentation of control \
implementation decisions, including any tailoring rationale, should be maintained as \
part of the system security plan and made available for audit review upon request. \
Personnel responsible for implementing and maintaining {sub_lower} controls should \
receive specialized training appropriate to their role and should maintain current \
knowledge of emerging threats and countermeasures relevant to {domain_lower}.\n\n",
    );
}
