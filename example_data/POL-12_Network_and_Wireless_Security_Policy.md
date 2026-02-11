# Network and Wireless Security Policy

## Purpose

This policy defines the requirements for establishing network controls and wireless security measures related to the Organization's computer and communications systems infrastructure.

## Scope

This policy applies to all computer systems and networks of the Organization, including networks managed by third parties and all wireless networking technology. The target audience is Information Technology employees and partners.

## Policy

### Network Authorization

**Network Security Configuration** — Configurations on all hosts attached to the Organization's network SHOULD comply with security policies and standards, whether in production environments or not.

**New Network Connections** — Any expansion to existing network connections SHALL be approved by senior IT management and tested prior to production implementation.

**Intranet Connection Security** — All networking equipment SHALL meet security criteria established by IT management, including authentication standards and monitoring processes, before connection to the internal network.

**Network Management** — Management of network equipment SHALL NOT be performed by personnel who do not possess professional network technology skills.

### Network Design

**Single Point of Failure** — Communications networks SHOULD be designed so that no single point of failure could cause network services to be unavailable, employing diverse carriers where feasible.

**Network Domains** — All large networks crossing organizational boundaries SHALL have separately defined logical domains, each protected with suitable security perimeters and access control mechanisms.

**Critical Systems Segregation** — Systems and networks deemed critical or containing sensitive information SHALL run on dedicated equipment and not share resources with unsecured or test systems, unless approved by IT management.

### Network Documentation

**Network Diagram** — A network diagram illustrating all connections, including wireless networks, SHALL be developed and maintained.

**Network Information Restriction** — Internal system addresses, configurations, and related design information SHALL be restricted from systems and users outside the internal network.

**External Connection Inventory** — The IT Department SHALL maintain a current inventory of all connections to external networks.

### System Configuration

**Public Internet Servers** — Public Internet servers SHALL be placed on network segments separate from internal networks, with public traffic restricted by firewalls.

**External-Facing Systems** — All systems interfacing external networks SHOULD be running the latest secure version of vendor-supplied operating system software.

**Network Hardening** — Network equipment SHALL be configured so that only features required for intended operations are enabled and any deprecated configuration options are disabled.

**Logging** — Devices SHALL be configured to capture meaningful events in logs for detection and investigation of security-related actions.

### Access Control

**Network Access Control** — All computers reachable by third-party networks SHALL be protected by an access control system approved by IT management.

**Network Device Passwords** — All internal network devices, including routers, firewalls, and access control servers, SHALL have unique passwords that differ from default manufacturer-assigned passwords.

**Remote Network Access** — Remote connection to the internal network SHALL be restricted to approved VPN software.

**Web Filtering** — Web filtering rules SHALL be established and kept current to minimize exposure to known or suspected malicious online resources.

### Third-Party Network Access

**External Connections** — Direct connections between the Organization's systems and computers at external organizations SHALL be subject to approval by IT management.

**Third-Party Networks** — The Organization's computers or networks SHALL be connected to third-party networks only after IT management has determined compliance with security requirements.

### Firewalls and Traffic Control

**Internet Access** — All Internet access from offices SHALL be routed through a firewall or similar device providing firewall functionality.

**Firewall Management** — Network configuration standards SHALL include a description of groups, roles, and responsibilities for the logical management of firewalls and routers.

### Network Segregation

**Security Zones** — All internal data networks SHALL be divided into security zones.

**Traffic Restriction** — All inbound and outbound traffic SHALL be restricted to that which is necessary for the data environment.

**Protocol Restriction** — Inbound and outbound traffic SHALL be protected by a firewall and DMZ that permits only necessary ports and protocols.

**Inbound Traffic Limitation** — Inbound Internet traffic SHALL be limited to IP addresses within the DMZ.

**Data Segregation** — Confidential information SHALL be located in a separate network zone, segregated from the DMZ. Network segments SHALL be capable of isolation in emergency situations.

### Wireless Security

**Guest Access** — Guest access to corporate wireless networks is not supported. Guest access SHALL be provided on a separate dedicated wireless network that provides only public Internet access.

**Rogue Access Point Detection** — The Organization SHALL automatically detect the presence of rogue wireless access points on the LAN and alert the networking and information security teams.

**Wireless Procurement** — Users SHALL NOT purchase, rent, or otherwise procure wireless equipment independently. All wireless procurements SHALL be channeled through the Purchasing Department.

**Wireless Encryption** — Wireless networks SHALL NOT be used for applications processing confidential data unless the network provides encryption according to standards developed by the Information Security Department.

**Access Point Installation** — All wireless access points SHALL be installed and configured by authorized systems administration staff or authorized contractors, following IT Department standards.

**Vendor Defaults** — All vendor default passwords and SSIDs on wireless equipment SHALL be changed.

**Physical Security of Access Points** — All wireless access points SHALL be physically secured in areas accessible only by authorized personnel, and placed to minimize unauthorized signal interception.

**Logical Separation** — All wireless access points SHALL be logically separated from the internal network using configurations approved by the Information Security Department.

**Encryption Standards** — Wireless access points SHALL use the latest supported encryption standards and SHALL NOT support legacy connection types.

## Definitions

- **Demilitarized Zone (DMZ)**: A network segment that sits between an internal network and the Internet, providing an additional layer of security by isolating external-facing services
- **Firewall**: A system designed to prevent unauthorized access to or from a private network, examining traffic and blocking that which does not meet specified security criteria
- **Service Set Identifier (SSID)**: A sequence of characters that uniquely identifies a wireless local area network

## References

- ISO/IEC 27002: 8.20 Network security, 8.21 Security of network services, 8.22 Segregation of networks, 8.23 Web filtering
