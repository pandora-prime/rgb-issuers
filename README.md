# RGB Contract Issuers

The repository contains a set of RGB contract issuers made by company Pandora Prime Inc.

NB: Use at your own risk; no external audit of the contract security has been made yet.

## Issuer classification

### Fungible asset (RGB20) Issuers

|              | No inflation                       | Inflatible                     |
|--------------|------------------------------------|--------------------------------|
| Non-burnable | NFA: Non-inflatable Fungible Asset | IFA: Inflatable Fungible Asset |
| Burnable     | BFA: Burnable Fungible Asset       | RFA: Replacable Fungible Asset |

### Unique digital asset (RGB21 and RGB25) Issuers

|                       | Indivisible                            | Fractionable                                |
|-----------------------|----------------------------------------|---------------------------------------------|
| Single                | IDA: Indivisible Digital Asset         | FUA: Fractional Unique Asset (RGB25)        |
| Fized-size collection | IAC: Indivisible Asset Collection      | FAC: Fractional Asset Collection            |
| Extendible collection | IEC: Indivisible Extendible Collection | FEC: Fractional Asset Extendible Collection |

## Issuer readiness

| Interface | Name | ETA     | Tests | Audit |
|-----------|------|---------|-------|-------|
| RGB20     | NFA  | Ready   | Full  | No    |
| RGB20     | IFA  | WIP     | -     | -     |
| RGB20     | BFA  | 2024 H2 | -     | -     |
| RGB20     | RGA  | 2025    | -     | -     |
| RGB25     | FUA  | Ready   | No    | No    |
| RGB21     | IDA  | Ready   | No    | No    |
| RGB21     | IAC  | Ready   | No    | No    |
| RGB21     | IEC  | 2024 H2 | -     | -     |
| RGB21     | FAC  | Ready   | Yes   | No    |
| RGB21     | FEC  | 2024 H2 | -     | -     |

