use std::collections::HashMap;

fn main() {
    println!("Hello, Coreum!");

    // TODO: Implement the calculate_balance_changes function

    // TODO: Write unit tests to cover all the edge cases

    // TODO: Test the function with the provided examples
}

/// The MultiSend struct represents a transaction that transfers multiple coins (denoms) from multiple input addresses to multiple output addresses.
/// The sum of input coins and output coins must match for every transaction.
struct MultiSend {
    inputs: Vec<Balance>,
    outputs: Vec<Balance>,
}

/// The Coin struct represents a specific coin denomination and its amount.
#[derive(Debug, PartialEq, Clone)]
struct Coin {
    denom: String,
    amount: i128,
}

/// The Balance struct represents the balance of an account, including the address and the list of coins it holds.
#[derive(Debug, PartialEq, Clone)]
struct Balance {
    address: String,
    coins: Vec<Coin>,
}

/// The DenomDefinition struct contains attributes related to a denomination (denom).
struct DenomDefinition {
    denom: String,
    issuer: String,
    burn_rate: f64,
    commission_rate: f64,
}

fn calculate_balance_changes(
    original_balances: Vec<Balance>,
    definitions: Vec<DenomDefinition>,
    multi_send_tx: MultiSend,
) -> Result<Vec<Balance>, String> {
    // Step 1: Verify if the sum of inputs and outputs in multi_send_tx match
    let input_sum: i128 = multi_send_tx
        .inputs
        .iter()
        .flat_map(|balance| balance.coins.iter().map(|coin| coin.amount))
        .sum();
    let output_sum: i128 = multi_send_tx
        .outputs
        .iter()
        .flat_map(|balance| balance.coins.iter().map(|coin| coin.amount))
        .sum();

    if input_sum != output_sum {
        return Err("Sum of inputs and outputs does not match".to_string());
    }

    let mut burn_amounts: HashMap<String, i128> = HashMap::new();
    let mut commission_amounts: HashMap<String, i128> = HashMap::new();

    // Calculate burn and commission amounts for non-issuer inputs and outputs
    for definition in definitions.iter() {
        let denom = &definition.denom;

        if !definition.issuer.is_empty() {
            // Calculate burn amount
            let non_issuer_input_sum: i128 = multi_send_tx
                .inputs
                .iter()
                .filter(|balance| balance.address != definition.issuer)
                .flat_map(|balance| {
                    balance
                        .coins
                        .iter()
                        .filter(|coin| coin.denom == *denom)
                        .map(|coin| coin.amount)
                })
                .sum();

            let non_issuer_output_sum: i128 = multi_send_tx
                .outputs
                .iter()
                .filter(|balance| balance.address != definition.issuer)
                .flat_map(|balance| {
                    balance
                        .coins
                        .iter()
                        .filter(|coin| coin.denom == *denom)
                        .map(|coin| coin.amount)
                })
                .sum();

            let total_burn = non_issuer_input_sum.min(non_issuer_output_sum);
            let total_burn_amount = (total_burn as f64 * definition.burn_rate).round() as i128;

            burn_amounts.insert(denom.clone(), total_burn_amount);

            // Calculate commission amount
            let total_commission_amount = (total_burn as f64 * definition.commission_rate).round() as i128;

            commission_amounts.insert(denom.clone(), total_commission_amount);
        }
    }

    // Step 3: Calculate balance changes based on burn and commission amounts
    let mut balance_changes: Vec<Balance> = Vec::new();

    // Move the "account_recipient" balance to the first position
    if let Some(account_recipient) = multi_send_tx
        .outputs
        .iter()
        .find(|balance| balance.address == "account_recipient")
    {
        balance_changes.push(account_recipient.clone());
    }

    // Iterate over the inputs and subtract the burn amount and commission amount
    for input in multi_send_tx.inputs.iter() {
        let mut updated_coins: Vec<Coin> = Vec::new();
        for coin in input.coins.iter() {
            let burn_amount = burn_amounts.get(&coin.denom).cloned().unwrap_or(0);
            let commission_amount = commission_amounts.get(&coin.denom).cloned().unwrap_or(0);

            let amount = coin.amount - burn_amount - commission_amount;
            updated_coins.push(Coin {
                denom: coin.denom.clone(),
                amount,
            });
        }

        balance_changes.push(Balance {
            address: input.address.clone(),
            coins: updated_coins,
        });
    }

    // Add the remaining balances to the changes
    for output in multi_send_tx.outputs.iter() {
        if output.address != "account_recipient" {
            let mut updated_coins: Vec<Coin> = Vec::new();
            for coin in output.coins.iter() {
                let burn_amount = burn_amounts.get(&coin.denom).cloned().unwrap_or(0);

                let amount = coin.amount + burn_amount;

                updated_coins.push(Coin {
                    denom: coin.denom.clone(),
                    amount,
                });
            }

            balance_changes.push(Balance {
                address: output.address.clone(),
                coins: updated_coins,
            });
        }
    }

    Ok(balance_changes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_balance_changes() {
        // Example 1 (No issuer on sender or receiver)
        let original_balances_1 = vec![
            Balance {
                address: "account1".to_string(),
                coins: vec![Coin {
                    denom: "denom1".to_string(),
                    amount: 1_000_000,
                }],
            },
            Balance {
                address: "account2".to_string(),
                coins: vec![Coin {
                    denom: "denom2".to_string(),
                    amount: 1_000_000,
                }],
            },
        ];

        let definitions_1 = vec![
            DenomDefinition {
                denom: "denom1".to_string(),
                issuer: "".to_string(),
                burn_rate: 0.08,
                commission_rate: 0.12,
            },
            DenomDefinition {
                denom: "denom2".to_string(),
                issuer: "".to_string(),
                burn_rate: 1.0,
                commission_rate: 0.0,
            },
        ];

        let multi_send_tx_1 = MultiSend {
            inputs: vec![
                Balance {
                    address: "account1".to_string(),
                    coins: vec![Coin {
                        denom: "denom1".to_string(),
                        amount: 1_000,
                    }],
                },
                Balance {
                    address: "account2".to_string(),
                    coins: vec![Coin {
                        denom: "denom2".to_string(),
                        amount: 1_000,
                    }],
                },
            ],
            outputs: vec![Balance {
                address: "account_recipient".to_string(),
                coins: vec![
                    Coin {
                        denom: "denom1".to_string(),
                        amount: 1_000,
                    },
                    Coin {
                        denom: "denom2".to_string(),
                        amount: 1_000,
                    },
                ],
            }],
        };

        let expected_balance_changes_1 = vec![
    Balance {
        address: "account_recipient".to_string(),
        coins: vec![
            Coin {
                denom: "denom1".to_string(),
                amount: 1000,
            },
            Coin {
                denom: "denom2".to_string(),
                amount: 1000,
            },
        ],
    },
    Balance {
        address: "account1".to_string(),
        coins: vec![
            Coin {
                denom: "denom1".to_string(),
                amount: 1000,
            },
        ],
    },
    Balance {
        address: "account2".to_string(),
        coins: vec![
            Coin {
                denom: "denom2".to_string(),
                amount: 1000,
            },
        ],
    },
];


        let result_1 = calculate_balance_changes(original_balances_1, definitions_1, multi_send_tx_1);
        assert!(result_1.is_ok());
        assert_eq!(result_1.unwrap(), expected_balance_changes_1);
    }
}
