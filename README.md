# RGB Contract Issuers

The repository contains a set of RGB contract issuers made by company Pandora Prime Inc.

NB: Use at your own risk; no external audit of the contract security has been made yet.

## Issuer classification

Each issuer has an abbreviation of three letters.

The first letter signifies the type of assets:

- fungible RGB20 (`F`),
- non-fungible RGB21 (`N`), and
- uniquely fungible RGB25 (`U`).

The second letter signifies subtype, shown in columns in the tables below;
and the third letter is a subtype, shown in rows.

### Fungible asset (RGB20) Issuers

|              | No inflation, `N`                         | Inflatable, `I`                 |
|--------------|-------------------------------------------|---------------------------------|
| Non-burnable | [FNA: Fungible Non-inflatable Asset][FNA] | FIA: Fungible Inflatable Asset  |
| Burnable     | FBA: Fungible Burnable Asset              | FRA: Fungible Replaceable Asset |

### NFT (RGB21) Issuers

|                              | Indivisible, `F`                        | Divisible, `D`                                      |
|------------------------------|-----------------------------------------|-----------------------------------------------------|
| Unique (single), `U`         | [NFU: Non-fungible Unique asset][NFU]   | NDU: Non-fungible Divisible Unique asset            |
| Collection (fixed size), `C` | NFC: Non-fungible asset Collection)     | NDC: Non-fungible asset Collection                  |
| Extendable collection, `E`   | NFE: Non-fungible Extendable collection | NDE: Non-fungible Divisible & Extendable collection |

### Unique fungible asset (RGB25) Issuers

[UFA: Unique fungible asset][UFA]

## Issuer readiness

| Standard | Short Name | ETA      | Tests   | Audit | Codex Id                                           |
|----------|------------|----------|---------|-------|----------------------------------------------------|
| RGB20    | [FNA]      | Ready    | Full    | No    | `7C15w3W1-L0T~zXw-Aeh5~kV-Zquz729-HXQFKQW-_5lX9O8` |
| RGB20    | FIA        | 2025 Jul | Partial | No    |                                                    |
| RGB20    | FBA        | 2025 H2  | Partial | No    |                                                    |
| RGB20    | FRA        | 2025     | Partial | No    |                                                    |
| RGB25    | [UFA]      | 2025 Jun | Full    | No    | `mqmGvodN-WxpxyRb-RsGHiTQ-jDoRvMy-ClhrQGx-FMa5Cpw` |
| RGB21    | [NFU]      | 2025 Jun | Full    | No    | `1T9uRsVB-_A8owmV-mv9SMiF-vgFFjN2-kxd9AZz-lpU8K1A` |
| RGB21    | NFC        | 2025 Jul | Partial | No    |                                                    |
| RGB21    | NFE        | 2025 H2  | Partial | No    |                                                    |
| RGB21    | NDU        | 2025 Jul | Partial | No    |                                                    |
| RGB21    | NDC        | 2025 Jul | Partial | No    |                                                    |
| RGB21    | NDE        | 2025 H2  | Partial | No    |                                                    |

[FNA]: compiled/RGB20-Simplest-v0-AYkSrg.issuer

[NFU]: compiled/RGB21-UniqueNFT-v0-FtU80Q.issuer

[UFA]: compiled/RGB25-UniquelyFungible-v0-aiRB3w.issuer
