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

|              | No inflation (`N`)                        | Inflatable (`I`)                |
|--------------|-------------------------------------------|---------------------------------|
| Non-burnable | [FNA: Fungible Non-inflatable Asset](FNA) | FIA: Fungible Inflatable Asset  |
| Burnable     | FBA: Fungible Burnable Asset              | FRA: Fungible Replaceable Asset |

### NFT (RGB21) Issuers

|                              | Indivisible (`F`)                         | Divisible (`D`)                                     |
|------------------------------|-------------------------------------------|-----------------------------------------------------|
| Unique (single, `U`)         | [NFU: Non-fungible Unique asset](NFU)     | NDU: Non-fungible Divisible Unique asset            |
| Collection (fixed size, `C`) | [NFC: Non-fungible asset Collection](NFC) | [NDC: Non-fungible asset Collection](NDC)           |
| Extendable collection (`E`)  | NFE: Non-fungible Extendable collection   | NDE: Non-fungible Divisible & Extendable collection |

### Unique fungible asset (RGB25) Issuers

UDA: Unique fungible asset

## Issuer readiness

| Standard | Name | ETA      | Tests   | Audit |
|----------|------|----------|---------|-------|
| RGB20    | FNA  | Ready    | Full    | No    |
| RGB20    | FIA  | 2025 Jul | Partial | No    |
| RGB20    | FBA  | 2025 H2  | -       | -     |
| RGB20    | FRA  | 2025     | -       | -     |
| RGB25    | UFA  | Ready    | Full    | No    |
| RGB21    | NFU  | Ready    | Full    | No    |
| RGB21    | NFC  | Ready    | Full    | No    |
| RGB21    | NFE  | 2025 H2  | -       | -     |
| RGB21    | NDU  | 2025 H2  | -       |       |
| RGB21    | NDC  | Ready    | Full    | No    |
| RGB21    | NDE  | 2025 H2  | -       | -     |

[FNA]: compiled/RGB20-Simplest.issuer

[NFU]: compiled/RGB21-UniqueNFT.issuer

[NFC]: compiled/RGB21-NFTCollection.issuer

[NDC]: compiled/RGB21-DivisibleCollection.issuer

[UDA]: compiled/RGB25-UniquelyFungible.issuer
