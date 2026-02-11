# Information Exchange and Transfer Policy

## Purpose

This policy defines controls for the proper exchange, storage, retention, and disposal of all information belonging to the Organization, in both paper and electronic format.

## Scope

This policy applies to all computer systems and facilities of the Organization, including those managed by third parties. It applies to all employees, partners, and third parties with access to information assets in digital or hardcopy form.

## Policy

### Disclosure Restrictions

**External Information Requests** — All requests from a third party for internal information not classified as PUBLIC SHALL require approval by the Information Owner.

**Disclosure of Sensitive Information** — When sensitive information must be shared, information SHALL be concealed using methods including, but not limited to, data masking, pseudonymization, and anonymization.

### Physical Transit Controls

**Delivering Confidential Output** — Confidential computer system hardcopy output SHOULD be personally delivered to designated recipients and never delivered to an unattended desk or left out in the open.

### Electronic Transmission

**Transferring Confidential Information** — Before any Confidential or Internal information may be transferred from one system to another, the worker making the transfer SHALL ensure that access controls on the destination system are commensurate with access controls on the originating system.

**Wireless Transmissions** — Unencrypted wireless technology SHALL never be used to transmit unencrypted confidential information.

**Third-Party Delivery** — Unencrypted Confidential Information SHALL NOT be sent through any third parties, including couriers, postal services, telephone companies, and Internet service providers.

**Public Network Data Transmission** — Strong cryptography and security protocols SHALL be implemented to safeguard confidential information during transmission over open, public networks.

**Data Leakage Prevention** — Data leakage prevention (DLP) measures SHALL be implemented, managed, and evolved to align with business requirements.

### Verbal Transmission

**Verbal Transmission Guidelines** — Care SHOULD be taken to protect confidential information from verbal transfer by ensuring conversations are conducted in areas where privacy can be maintained. Restricted or Confidential information MUST NOT be transferred by telephone unless the identity and authorization of the receiver has been appropriately confirmed.

### Electronic Mail

**Email Encryption** — All sensitive information, including personally identifiable information (PII), SHALL be encrypted when transmitted through electronic mail.

**Email Addresses** — Workers SHALL NOT employ any email addresses other than official corporate addresses for all company business matters.

**Email Authenticity** — Email systems SHALL be configured to use industry-standard methods of proving email authenticity, including DKIM signing and SPF where applicable.

### Data Storage and Retention

**Information Asset Inventory** — Annually, the Organization SHALL compile a data-mapping exercise that includes a high-level description of major information assets. The data dictionary SHOULD include data owners.

**System of Record** — Where appropriate, each data owner SHALL designate a system of record that serves as the most authoritative copy of the information under their care.

**Storage Restrictions** — Confidential data SHOULD be encrypted during storage on electronic media. All encryption SHALL follow standards established by the Information Technology Department.

**External Storage Devices** — Only devices provided by IT, configured with encryption and access protection, SHALL be allowed for use with corporate devices.

**Retention Periods** — A retention period SHALL be assigned to all confidential information, regardless of form. Information not specifically listed on the Records Retention Schedule SHALL be retained only for as long as necessary, as designated by the information owner.

**Customer Data Retention** — Customer data SHALL be retained and protected for the duration of the business relationship. After that point, the Organization may destroy customer data based on contract terms.

**Source Code** — In Organization-owned source control management systems, active repositories SHALL NOT be deleted but may be archived. Archived repositories may only be deleted after a minimum archival period and review by an appropriate manager.

**Litigation Hold** — If there is credible reason to believe that internal documents may be needed as evidence in upcoming litigation, these documents SHALL NOT be destroyed. They SHALL be brought to the attention of internal legal counsel and properly secured.

### Disposal of Information and Media

**Hardcopy Disposal** — When disposed of, all confidential information in hardcopy form SHALL be either shredded or incinerated using approved equipment. Confidential information no longer needed SHALL be placed in designated locked destruction containers and never in publicly accessible trash or recycle bins.

**Electronic Media Destruction** — All data storage devices SHALL be destroyed or the data sanitized using certified third-party vendors, and certificates of destruction SHALL be obtained.

**Equipment Disposal** — Data storage devices SHALL be destroyed or sanitized before disposal. Endpoints MUST be encrypted before being sent to a third party for destruction.

**Transfer of Electronic Media** — Before electronic media is transferred from the custody of the current owner, appropriate care SHALL be taken to ensure that no unauthorized person can access data by ordinary means.

### Travel Considerations

**Travel with Confidential Information** — Employees traveling with Confidential Information SHALL employ both full-disk encryption and multi-factor authentication.

### Internet Transmission

**Cryptographic Standards** — Internet communications of non-public information SHALL be transmitted using TLS 1.2 or stronger cryptography. The use of deprecated cryptographic protocols with non-public information is prohibited.

**Public Website Restrictions** — Internal and Confidential information SHALL NOT be published on public websites.

## Definitions

- **Data Leakage Prevention (DLP)**: Security measures designed to detect and prevent unauthorized transmission of sensitive data outside the Organization
- **Litigation Hold**: A directive to preserve documents and data that may be relevant to pending or anticipated legal proceedings
- **Data Masking**: The process of obscuring specific data within a database to protect it from unauthorized access

## References

- ISO/IEC 27002: 5.14 Information transfer, 5.33 Protection of records, 7.10 Storage media, 7.14 Secure disposal or re-use of equipment, 8.10 Information deletion, 8.11 Data masking, 8.12 Data leakage prevention
- SOC 2: CC6.5, CC6.7, C1.1, C1.2
