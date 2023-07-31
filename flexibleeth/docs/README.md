# Flexible Ethereum Confirmation Rule (Proof-of-Concept)

## Clarifications

- All work we do is with respect to the "canonical chain" of the consensus layer client. To ensure that the "canonical chain" in practice won't change inconsistently across our experiments (this is only a proof-of-concept afterall), we stay "far" away from the current tip of the chain.

## Database Schema

All data stored in the database is bincoded.
- `block_<<slot>>: data::Root`: Block-root for given slot on the canonical chain
- `block_<<root>>: data::Block`: Block for given block-root
- `epoch_<<epoch>>_root: data::Root`: Block-root for the epoch boundary block of given epoch
- `state_<<root>>_finality_checkpoints: (data::Checkpoint, data::Checkpoint, data::Checkpoint)`: Checkpoint information committed by the given state-root
- `state_<<root>>_committees: Vec<data::CommitteeAssignment>`: Committee information committed by the given state-root
- `slot_<<slot>>_synched: bool`: Set if given slot is synched (the value is always true) 
- `epoch_<<epoch>>_state_synched: bool`: Set if state of given epoch is synched (the value is always true) 
