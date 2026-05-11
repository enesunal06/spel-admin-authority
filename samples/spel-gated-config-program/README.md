# SPEL Gated Config Sample

This is reference SPEL source for RFP-001. It follows the current SPEL pattern:

- `#[lez_program]`
- `#[instruction]`
- `#[account(init/mut/signer/pda = ...)]`
- explicit `assert_admin(...)` helper calls inside privileged handlers

It is intentionally not a default workspace member because it depends on pinned
SPEL/LEE Git dependencies and a RISC Zero guest build toolchain. Maintainers can
copy this sample into a `logos-scaffold` project or run it in a supported
Unix/Linux environment after replacing the pinned dependency with the project
standard.

The dependency pin used here is:

```toml
spel-framework = { git = "https://github.com/logos-co/spel.git", rev = "ea2f998ed13c5ad66ef1cafe51de859bef27f872", package = "spel-framework" }
nssa_core = { git = "https://github.com/logos-blockchain/logos-execution-zone.git", tag = "v0.2.0-rc3" }
risc0-zkvm = { version = "=3.0.5", features = ["std"] }
```
