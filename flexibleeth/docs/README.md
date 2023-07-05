# Flexible Ethereum Confirmation Rule (Proof-of-Concept)

## Clarifications

- All work we do is with respect to the "canonical chain" of the consensus layer client. To ensure that the "canonical chain" in practice won't change inconsistently across our experiments (this is only a proof-of-concept afterall), we stay "far" away from the current tip of the chain.

## Database Schema

All data stored in the database is bincoded.
- `sync_progress: usize`: Number of the latest completely sync'ed slot.
- `block_<<root>>: data::Block`: Block for given root
- `block_<<slot>>: data::Root`: Block root for given slot on the canonical chain
- `chain_<<root>>: Vec<data::Root>`: Sequence of block roots on the chain identified by given root
- `state_<<slot>>_finality_checkpoints: (data::Checkpoint, data::Checkpoint, data::Checkpoint)`: Checkpoint information committed into the block for given slot on the canonical chain
