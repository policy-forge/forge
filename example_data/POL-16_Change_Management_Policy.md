# Change Management Policy

## Purpose

This policy defines the requirements for managing changes to the Organization's computer and communications systems, ensuring that all modifications are properly authorized, documented, tested, and reviewed.

## Scope

This policy applies to all personnel responsible for the maintenance of computer and communications systems managed by the Organization's Corporate IT department. Production environment changes may follow supplemental change control processes.

## Policy

### Authorization and Review

**Change Committee** — Leaders from the information technology and security departments SHALL be appointed to a change committee responsible for the review of change submissions.

**Planned Changes** — Extensions, modifications, replacements, additions, or any other change to the established operating environment SHALL be submitted for approval through the change control system.

**Change Approval** — Change submissions SHALL be reviewed for approval on a regular cadence. Change requesters SHALL NOT approve their own change requests.

**Emergency Changes** — Where the security, integrity, or operational status of systems is impacted or under immediate threat, an emergency change control meeting may be called by the change owner outside the regular cadence. Where the change committee is unavailable in emergency situations, the CISO or Director of Information Technology may authorize such changes.

### Change Procedures

**Change Control Procedure** — Requesters SHALL determine the technical, human, and chronological requirements and impacts of their proposal and submit this information via a standardized format to the change committee for review.

**Change Personnel** — Only those with pre-authorized permissions SHALL be allowed to carry out approved changes. Requesters SHALL NOT be granted additional permissions above those required for their role.

**Back-Out Procedures** — Adequate back-out procedures, which permit information processing activities to revert to conditions prior to the most recent change, SHOULD be developed for all change requests.

**Communication** — Changes SHOULD be communicated to affected parties in advance to allow concerns or additional information to be presented.

### Recording Changes

**Change Logging** — The details of all changes SHALL be recorded and logged to a centralized system accessible by IT leadership.

**Change Log Contents** — The change management system SHALL log at minimum: date of submission, planned date of change, change sponsor, reason for change, rollout plan, impact if change is not made, impact during change, description including individual performing the change, systems being changed, versions of related software, testing status, backout plan, approval status, and approver name with date and time.

### Review and Testing

**Change Testing** — Prior to submission, changes SHOULD be tested for operational impact on test systems.

**Security Review** — Information security personnel SHOULD scan and review affected systems after changes to ensure that security has not been degraded.

**Software Controls** — Where applicable, software controls SHALL be used to enforce change control, preventing unauthorized changes and ensuring the defined process is followed.

## Definitions

- **Change**: Any modification to the information processing infrastructure resulting from implementation of new functionality, interruption of service, repair of existing functionality, or removal of existing functionality
- **Change Management**: The process of controlling modifications to hardware, software, firmware, and documentation to protect information resources against improper modification
- **Emergency Change**: An unauthorized immediate response to imminent critical system failure needed to prevent widespread service disruption

## References

- ISO/IEC 27002: 8.32 Change management
