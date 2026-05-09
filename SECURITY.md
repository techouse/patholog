# Security Policy

## Supported Versions

| Version | Supported |
| ------- | --------- |
| 0.x.x   | Yes       |

## Scope

`patholog` is a local CLI for PATH diagnostics and safe profile repair. Current v0.x commands are read-only except for `apply --yes`, which writes a patholog-managed shell profile block.

Please report issues that could affect confidentiality, integrity, or availability, including:

- unexpected file writes or shell profile mutation
- misleading `apply --dry-run` profile repair plans
- missing or misleading `apply --yes` backups
- misleading `--drop` or `--preset` cleanup policy output
- command resolution behavior that could mislead users into executing the wrong binary
- output escaping or injection issues in human or JSON output
- crashes or resource exhaustion from untrusted PATH/PATHEXT input
- platform modeling bugs with clear security impact

## Reporting a Vulnerability

We take the security of our software seriously. If you believe you have found a security vulnerability, please report it to us as described below.

**DO NOT CREATE A GITHUB ISSUE** reporting the vulnerability.

Instead, send an email to [techouse@gmail.com](mailto:techouse@gmail.com).

In the report, please include the following:

- Your name and affiliation (if any).
- A description of the technical details of the vulnerability and how to reproduce it.
- An explanation of who can exploit this vulnerability and what they gain by doing so. An attack scenario helps us evaluate the report quickly, especially for complex findings.
- Whether this vulnerability is public or known to third parties. If it is, please provide details.

If you do not get an acknowledgment from us or have heard nothing from us in a week, please contact us again.

We will send a response indicating the next steps in handling your report. We will keep you informed about the progress towards a fix and full announcement.

We will not disclose your identity to the public without your permission. We strive to credit researchers in our advisories when we release a fix, but only after getting your permission.

We appreciate your efforts to responsibly disclose your findings, and will make every effort to acknowledge your contributions.
