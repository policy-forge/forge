# Backup and Business Continuity Policy

## Purpose

This policy defines the requirements for maintaining and recovering backup copies of critical information, and for developing, testing, and maintaining the Organization's business continuity plan to ensure resilient operations during disruptions.

## Scope

This policy applies to all computer systems, facilities, and information assets of the Organization. The target audience is management and Information Technology employees.

## Policy

### Backup Schedule

**Backup Scheduling** — The Organization SHALL maintain documentation describing how backup scheduling is planned, giving guidance to data custodians on determining backup specifications including retention times, recovery point objectives (RPO), and recovery time objectives (RTO).

**Scheduled Backups** — Critical data SHALL be periodically backed up on at least a weekly basis, supplemented by daily backups where deemed necessary.

**Retention** — Data custodians SHOULD establish the retention period suitable for their systems.

### Application Backup

**Decommission Backup** — Before any applications are taken out of production, a final backup of critical and confidential production data SHALL be made.

**Backup Classification** — Where target data has mixed classifications, backup files and media SHOULD have a classification label matching the highest (most sensitive) classification of the stored data.

### Backup Locations

**On-Site Backup** — At least one generation of backup files SHALL be maintained on data storage media wherever production computers are located.

**Physical Backup Storage** — For backups to physical media, critical data SHALL be stored at a site that is a sufficient distance from the originating facility.

**Cloud Backup Availability** — For cloud backups, critical data SHOULD be stored in at least two availability zones or regions, or with equivalent redundancy measures.

### Replication and Immutability

**Immutable Backups** — Files SHALL be archived in a manner that prevents their future modification. Removal of redundant replicas SHOULD NOT result in fewer than two valid copies remaining.

### Backup Media

**Media Storage** — Critical data backups SHALL be stored in an environmentally protected and access-controlled site with appropriate fire safeguards.

**Media Encryption** — All confidential, valuable, or critical information in backup files stored outside offices, including cloud backups, SHALL be encrypted.

### Backup Testing and Review

**Backup Testing** — Backup archives of critical data SHALL be tested at least annually to provide assurance that they can be fully recovered.

**Recovery Documentation** — Backup process owners SHOULD document restoration activities.

**Backup Review** — Department managers SHALL ensure that appropriate backups of critical data are being made and verify that restoration procedures are correctly described.

### Business Impact Analysis

**BIA Review** — The Information Security Department or designee SHOULD review the business impact analysis at least annually or following significant change.

**BIA Requirements** — The business impact analysis SHALL identify, at minimum: critical systems, regulatory reporting requirements, ICT continuity requirements, outage tolerance and operational impacts (RTO and RPO), and financial impact.

### System and Business Process Criticality

**Criticality Rating** — As part of the BIA, Information Owners SHALL define the criticality of all applications and business processes using a consistent classification system.

### Continuity Plan Development

**Plan Preparation** — Management SHALL prepare, periodically update, and regularly test a business recovery plan defining how workers can continue operations during a business interruption.

**Critical System Recovery** — A contingency plan SHOULD be developed to enable restoration of service within a defined timeframe for all critical applications.

**Recovery Procedures** — Procedures for restoring service SHALL be documented in formal contingency plans that are reviewed, tested, and updated periodically. System recovery procedures SHALL specifically assign responsibility for managing and facilitating the restoration of service.

**Plan Evolution** — The plan SHALL be evaluated for effectiveness in light of significant business changes or emerging threats.

### Plan Communication

**Plan Availability** — Business and information systems contingency plans SHALL be located across multiple systems such that an outage does not impact availability.

**Plan Classification** — The Organization's contingency plans are classified as Internal and SHALL NOT be disclosed to third parties without prior approval from the CISO or Information Security Manager.

### Plan Testing

**Contingency Plan Testing** — Computer and communication system contingency plans SHALL be tested periodically to ensure relevance and effectiveness. Tests SHALL include critical business resources and be followed by a documented report with findings and remedial actions.

**Plan Findings** — All findings SHALL be recorded and remediations tracked to completion. Critical findings SHOULD be remediated within 45 days.

**Plan Review** — The business continuity plan SHALL be reviewed at least annually.

### Recovery Personnel

**Training** — All designated recovery workers SHOULD receive sufficient training and practice to perform recovery tasks.

**Minimum Staffing** — At least two people SHOULD have the technical knowledge needed to perform essential recovery tasks.

**Notifications** — Every worker with responsibility for business recovery SHALL be notified of responsibilities and corresponding work requirements.

## Definitions

- **Business Continuity Plan (BCP)**: Documentation of a predetermined set of instructions describing how business functions will be sustained during and after a significant disruption
- **Business Impact Analysis (BIA)**: A management-level analysis identifying the impacts of losing resources, measuring the effect of loss over time
- **Recovery Time Objective (RTO)**: The maximum acceptable time that a system or process can be unavailable after a disruption
- **Recovery Point Objective (RPO)**: The maximum acceptable amount of data loss measured in time

## References

- ISO/IEC 27002: 5.29 Information security during disruption, 5.30 ICT readiness for business continuity, 8.13 Information backup, 8.14 Redundancy of information processing facilities
