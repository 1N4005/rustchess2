# Uci Chess Engine Written in Rust
## Dependencies
Cargo `rand` crate
## Features
### Board Representation
Mailbox + Bitboards
### Search
* Quiescence Search
* Alpha-Beta Pruning
* Iterative Deepening
* Move Ordering
    * TT Move
    * MVV-LVA
* Extensions
    * Check Extensions
### Evaluation
* Piece Square Tables
### Time Management
Allocates 1/40th of the remaining time on the clock for the search.
