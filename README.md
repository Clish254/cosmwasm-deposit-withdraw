This is a simple cosmwasm smart contract that enables the owner to deposit and withdraw custom coins.

Here is a brief overview of the entrypoints

# Instantiate

To instantiate the contract you pass in:

1. owner - if this is _null_ it will be defaulted to the transaction sender's address.
2. accepted_denoms - this is an array of the addresses of the two custom coins which can be deposited and withdrawn from the contract.

# Execute

There are two variants of execute messages that can be sent:

1. Deposit - this allows a user to deposit the two accepted custom coins to the contract.
2. Withdraw - this allows a user to withdraw two custom coins from the contract.
   _N/B_ only the registered contract owner can deposit and withdraw

# Tests

After cloning this repo you can run tests using the following command:

```shell
cargo test
```
