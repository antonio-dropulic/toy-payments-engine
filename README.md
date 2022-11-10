A toy payments engine - tasked as a coding challenge.

## Task definition:

Implement a simple toy payments engine that reads a series of transactions
from a CSV, updates client accounts, handles disputes and chargebacks, and then outputs the
state of clients accounts as a CSV.

The following api must be exposed: `cargo run -- transactions.csv > accounts.csv`.

Input CSV example:

```
type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
withdrawal, 2, 5, 3.0
```

The output should be a list of client IDs (client), available amounts (available), held amounts(held), total amounts (total), and whether the account is locked (locked).

Output example:

```
client, available, held, total, locked
1, 1.5, 0.0, 1.5, false
2, 2.0, 0.0, 2.0, false
```

## Types

### amount

You can assume a precision of four places past the decimal and should output values with the
same level of precision.

### client

A valid u16

### tx

A valid u32

## Transaction types

### Deposit

A deposit is a credit to the client's asset account, meaning it should increase the available and total funds of the client account.

```
Deposit {
    client,
    tx,
    amount,
}
```

### Withdrawal

A withdraw is a debit to the client's asset account, meaning it should decrease the available and total funds of the client account.

```
Withdrawal {
    client,
    tx,
    amount,
}
```

### Dispute

A dispute represents a client's claim that a transaction was erroneous and should be reversed.
The transaction shouldn't be reversed yet but the associated funds should be held. This means
that the clients available funds should decrease by the amount disputed, their held funds should increase by the amount disputed, while their total funds should remain the same.
Dispute references the transaction that is disputed by ID. If the tx specified by the dispute doesn't exist you can ignore it and assume this is an error on our partners side.

```
Dispute {
    client,
    tx,
}
```

### Resolve

A resolve represents a resolution to a dispute, releasing the associated held funds. Funds that were previously disputed are no longer disputed. This means that the clients held funds should decrease by the amount no longer disputed, their available funds should increase by the
amount no longer disputed, and their total funds should remain the same. Like disputes, resolves do not specify an amount. Instead they refer to a transaction that was
under dispute by ID. If the tx specified doesn't exist, or the tx isn't under dispute, you can ignore the resolve and assume this is an error on our partner's side.

```
Resolve {
    client,
    tx,
}
```

### Chargeback

A chargeback is the final state of a dispute and represents the client reversing a transaction. Funds that were held have now been withdrawn. This means that the clients held funds and total funds should decrease by the amount previously disputed. If a chargeback occurs the client's account should be immediately frozen.  Like a dispute and a resolve a chargeback refers to the transaction by ID (tx) and does not
specify an amount. Like a resolve, if the tx specified doesn't exist, or the tx isn't under dispute, you can ignore chargeback and assume this is an error on our partner's side.

```
Chargeback {
    client,
    tx,
}
```

## TODO:

- CLI
- better error reporting (bubble up errors in the tx broker)
- add memory, cpu profiling (flamegraph)
- clean up account tests
- add docs
- stuck optimizing memory performance of the code
    - account memory scales linearly with input data. Need to hold deposits
    so disputes can reference it.
- with account operations being O(1) on average async broker seems like a bad idea,
    not sure if the overhead is ever worth it.
- run clippy
- remove amount checked addition to see if the code runs faster
- deserialization of tx can use &[u8] instead of &str to avoid utf8 checks
- test generation should be done in the bench code