# RFP-001 Traceability

This document maps implemented behavior to RFP-001 and calls out unresolved areas without overclaiming.

## Implemented Requirements

| Requirement | Implementation | Test Coverage |
| --- | --- | --- |
| Admin authority is set at program initialization. | `initialize_admin`, `AdminAuthority::initialized` | `initialize_admin_sets_one_admin`, sample `end_to_end_admin_lifecycle_gates_config_updates` |
| Admin authority can transfer authority to a new signer. | `transfer_admin` | `admin_can_transfer_to_valid_new_admin`, sample lifecycle test |
| Admin authority can renounce admin control. | `renounce_admin`, irreversible `renounced` bit | `renounce_admin_is_irreversible`, sample lifecycle test |
| Only admin can call privileged library instructions. | `assert_admin`, `update_admin_config` | `assert_admin_requires_matching_authorized_admin`, `update_admin_config_is_gated` |
| Demonstrate gated config PDA update. | `AdminConfig`, `samples/gated-config-program`, SPEL reference source | core and sample tests |
| Only one admin authority at a time. | `AdminAuthority { admin: Option<AdminKey> }`; transfer replaces the key | initialization and transfer tests |
| README dependency and integration steps. | `README.md` | Manual documentation review |
| Sample program imports the library. | `samples/gated-config-program`, `samples/spel-gated-config-program` | host sample tests; SPEL source included as reference |
| CI green on default branch. | `.github/workflows/ci.yml` | Local format/test/clippy verification |

## Partially Supported Or Unresolved

| Area | Current Status |
| --- | --- |
| Single annotation/config flag | Not implemented. The code uses explicit `assert_admin(...)` because robust single-annotation gating likely requires SPEL macro support. |
| On-curve signer validation | Not implemented manually. SPEL runtime signer metadata is treated as the source of truth until an official on-curve API is available. |
| Deployed PDA admin semantics | Conservative validator hook only. The library does not invent PDA signing behavior. Maintainer clarification is required for full deployed-PDA admin support. |
| Full localnet e2e | Not part of default CI. `logos-scaffold` is Unix-oriented; maintainers can run a scaffold/localnet smoke test in a supported Linux environment. |
| Dependency pinning | SPEL sample pins a known commit. Replace with maintainer-approved tag/path when available. |

## Performance Documentation

The README documents approximate transaction size overhead:

- 32 bytes for a newly introduced admin account key.
- 1 byte for the instruction account index, ignoring compact-length boundary changes.
- 64 bytes when a newly introduced normal signer signature is required.

Worst-case overhead for a new normal admin signer is about 97 bytes.

