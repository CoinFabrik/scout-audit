# Known vulnerabilities

## Description

- Category: `Known Bugs`
- Severity: `Medium`
- Detectors: [`known-vulnerabilities`](https://github.com/CoinFabrik/scout-audit/tree/main/detectors/rust/known-vulnerabilities)

Using libraries marked as risky is dangerous, as they contain vulnerabilities that pose significant risks to the security and stability of the codebase.

## Remediation

Updating to secure and supported versions of critical libraries or frameworks can help mitigate risks associated with outdated dependencies. Conducting routine security scans using tools such as cargo audit or similar ensures that potential vulnerabilities are identified and addressed promptly. Additionally, staying informed about security advisories and monitoring updates for patches or upgrades related to your dependencies will help maintain the long-term security of your application.

## References

- [RustSec](https://rustsec.org/)
- [RustSec Github Repository](https://github.com/RustSec/advisory-db)
