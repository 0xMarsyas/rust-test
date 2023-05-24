#[derive(Debug, Clone, PartialEq)]
pub struct MultiSend {
    inputs: Vec<Balance>,
    outputs: Vec<Balance>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Coin {
    pub denom: String,
    pub amount: i128,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Balance {
    address: String,
    coins: Vec<Coin>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DenomDefinition {
    denom: String,
    issuer: String,
    burn_rate: f64,
    commission_rate: f64,
}

fn find_definition<'a>(denom: &'a String, definitions: &'a Vec<DenomDefinition>) -> Option<&'a DenomDefinition> {
    for definition in definitions {
        if &definition.denom == denom {
            return Some(definition);
        }
    }
    None
}

fn find_balance<'a>(address: &'a String, balances: &'a mut Vec<Balance>) -> Option<&'a mut Balance> {
    for balance in balances {
        if &balance.address == address {
            return Some(balance);
        }
    }
    None
}

pub fn calculate_balance_changes(
    mut original_balances: Vec<Balance>,
    definitions: Vec<DenomDefinition>,
    mut multi_send_tx: MultiSend,
) -> Result<Vec<Balance>, String> {
    let mut input_sum: i128 = 0;
    let mut output_sum: i128 = 0;

    for input in &multi_send_tx.inputs {
        for coin in &input.coins {
            input_sum += coin.amount;
        }
    }

    for output in &multi_send_tx.outputs {
        for coin in &output.coins {
            output_sum += coin.amount;
        }
    }

    if input_sum != output_sum {
        return Err("Sum of inputs and outputs does not match.".to_string());
    }

    for input in &mut multi_send_tx.inputs {
        for coin in &mut input.coins {
            let definition = find_definition(&coin.denom, &definitions)
                .ok_or(format!("Could not find DenomDefinition for input coin denom {}", &coin.denom))?;

            let burn = (coin.amount as f64 * definition.burn_rate).round() as i128;
            let commission = (coin.amount as f64 * definition.commission_rate).round() as i128;

            coin.amount -= burn;
            coin.amount -= commission;

            let issuer_balance = find_balance(&definition.issuer, &mut original_balances)
                .ok_or(format!("Could not find original balance for issuer {}", &definition.issuer))?;

            let issuer_coin_position = issuer_balance.coins.iter().position(|c| c.denom == coin.denom);

            match issuer_coin_position {
                Some(pos) => {
                    issuer_balance.coins[pos].amount += burn + commission;
                },
                None => {
                    issuer_balance.coins.push(Coin {
                        denom: coin.denom.clone(),
                        amount: burn + commission,
                    });
                }
            }
        }
    }

    for input in &multi_send_tx.inputs {
        let original_balance = find_balance(&input.address, &mut original_balances)
            .ok_or(format!("Could not find original balance for address {}", &input.address))?;

        for coin in &input.coins {
            let original_coin = original_balance.coins.iter()
                .find(|c| c.denom == coin.denom)
                .ok_or(format!("Could not find original coin for denom {}", &coin.denom))?;

            if coin.amount > original_coin.amount {
                return Err(format!("Sender does not have enough balance for denom {}", &coin.denom));
            }
        }
    }

    for output in &multi_send_tx.outputs {
        let mut original_balance = find_balance(&output.address, &mut original_balances);

        if original_balance.is_none() {
            original_balances.push(Balance {
                address: output.address.clone(),
                coins: Vec::new(),
            });
            original_balance = find_balance(&output.address, &mut original_balances);
        }

        for coin in &output.coins {
            let balance = original_balance
                .as_mut()
                .ok_or(format!("Could not find original balance for address {}", &output.address))?;
            let coin_position = balance.coins.iter().position(|c| c.denom == coin.denom);

            match coin_position {
                Some(pos) => {
                    balance.coins[pos].amount += coin.amount;
                },
                None => {
                    balance.coins.push(Coin {
                        denom: coin.denom.clone(),
                        amount: coin.amount,
                    });
                }
            }
        }
    }

    Ok(original_balances)
}  // Remove the extra '}'

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_balance_changes_example1() {
        let original_balances = vec![
            Balance {
                address: "account1".to_string(),
                coins: vec![Coin { denom: "denom1".to_string(), amount: 1000 }],
            },
            Balance {
                address: "account2".to_string(),
                coins: vec![Coin { denom: "denom2".to_string(), amount: 500 }],
            },
            Balance {
                address: "issuer_account_A".to_string(),
                coins: vec![],
            },
            Balance {
                address: "issuer_account_B".to_string(),
                coins: vec![],
            },
        ];
        let definitions = vec![
            DenomDefinition {
                denom: "denom1".to_string(),
                issuer: "issuer_account_A".to_string(),
                burn_rate: 0.08,
                commission_rate: 0.12,
            },
            DenomDefinition {
                denom: "denom2".to_string(),
                issuer: "issuer_account_B".to_string(),
                burn_rate: 0.2,
                commission_rate: 0.1,
            },
        ];

        let multi_send_tx = MultiSend {
            inputs: vec![
                Balance {
                    address: "account1".to_string(),
                    coins: vec![Coin { denom: "denom1".to_string(), amount: 1000 }],
                },
                Balance {
                    address: "account2".to_string(),
                    coins: vec![Coin { denom: "denom2".to_string(), amount: 500 }],
                },
            ],
            outputs: vec![
                Balance {
                    address: "account_recipient".to_string(),
                    coins: vec![
                        Coin { denom: "denom1".to_string(), amount: 880 },
                        Coin { denom: "denom2".to_string(), amount: 350 },
                    ],
                },
            ],  // Fix the closing delimiter here
        };

        let result = super::calculate_balance_changes(original_balances.clone(), definitions, multi_send_tx).unwrap();
        assert_eq!(result, vec![  // Add the missing '[' here
            Balance {
                address: "account1".to_string(),
                coins: vec![],
            },
            Balance {
                address: "account2".to_string(),
                coins: vec![],
            },
            Balance {
                address: "issuer_account_A".to_string(),
                coins: vec![Coin { denom: "denom1".to_string(), amount: 220 }],  // 1000 * 0.12 + 1000 * 0.08
            },
            Balance {
                address: "issuer_account_B".to_string(),
                coins: vec![Coin { denom: "denom2".to_string(), amount: 150 }],  // 500 * 0.2 + 500 * 0.1
            },
            Balance {
                address: "account_recipient".to_string(),
                coins: vec![
                    Coin { denom: "denom1".to_string(), amount: 880 }, 
                    Coin { denom: "denom2".to_string(), amount: 350 },
                ],
            },
        ]);
    }

    // Add the missing closing bracket for 'mod tests' here
}

    #[test]
    fn test_no_denom_definition_for_input_coin() {
        let original_balances = vec![
            Balance {
                address: "account1".to_string(),
                coins: vec![Coin { denom: "denom1".to_string(), amount: 1000 }],
            },
        ];
        let definitions = vec![
            DenomDefinition {
                denom: "denom2".to_string(),
                issuer: "issuer_account_B".to_string(),
                burn_rate: 0.2,
                commission_rate: 0.1,
            },
        ];
        let multi_send_tx = MultiSend {
            inputs: vec![
                Balance {
                    address: "account1".to_string(),
                    coins: vec![Coin { denom: "denom1".to_string(), amount: 1000 }],
                },
            ],
            outputs: vec![
                Balance {
                    address: "account_recipient".to_string(),
                    coins: vec![
                        Coin { denom: "denom1".to_string(), amount: 1000 },
                    ],
                },
            ],
        };
        
        let result = calculate_balance_changes(original_balances, definitions, multi_send_tx);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Denom not found in denom definitions.".to_string());
    }

    #[test]
    fn test_no_balance_for_issuer() {
        let original_balances = vec![
            Balance {
                address: "account1".to_string(),
                coins: vec![Coin { denom: "denom1".to_string(), amount: 1000 }],
            },
            Balance {
                address: "issuer_account_A".to_string(),
                coins: vec![],
            },
        ];
        let definitions = vec![
            DenomDefinition {
                denom: "denom1".to_string(),
                issuer: "issuer_account_B".to_string(),  // issuer_account_B does not exist in original_balances
                burn_rate: 0.08,
                commission_rate: 0.12,
            },
        ];
        let multi_send_tx = MultiSend {
            inputs: vec![
                Balance {
                    address: "account1".to_string(),
                    coins: vec![Coin { denom: "denom1".to_string(), amount: 1000 }],
                },
            ],
            outputs: vec![
                Balance {
                    address: "account_recipient".to_string(),
                    coins: vec![
                        Coin { denom: "denom1".to_string(), amount: 880 },
                    ],
                },
            ],
        };
        

        let result = calculate_balance_changes(original_balances, definitions, multi_send_tx);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No balance entry found for issuer.".to_string());
    }

    #[test]
    fn test_not_enough_balance_to_cover_input() {
        let original_balances = vec![
            Balance {
                address: "account1".to_string(),
                coins: vec![Coin { denom: "denom1".to_string(), amount: 500 }],  // account1 only has 500 of denom1
            },
            Balance {
                address: "issuer_account_A".to_string(),
                coins: vec![],
            },
        ];
        let definitions = vec![
            DenomDefinition {
                denom: "denom1".to_string(),
                issuer: "issuer_account_A".to_string(),
                burn_rate: 0.08,
                commission_rate: 0.12,
            },
        ];
        let multi_send_tx = MultiSend {
            inputs: vec![
                Balance {
                    address: "account1".to_string(),
                    coins: vec![Coin { denom: "denom1".to_string(), amount: 1000 }],  // trying to send 1000 of denom1
                },
            ],
            outputs: vec![
                Balance {
                    address: "account_recipient".to_string(),
                    coins: vec![
                        Coin { denom: "denom1".to_string(), amount: 880 },
                    ],
                },
            ],
        };
        
        let result = calculate_balance_changes(original_balances, definitions, multi_send_tx);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not enough balance to cover input.".to_string());
    }

    #[test]
    fn test_no_original_coin_for_denom() {
        let original_balances = vec![
            Balance {
                address: "account1".to_string(),
                coins: vec![],  // account1 does not have denom1
            },
            Balance {
                address: "issuer_account_A".to_string(),
                coins: vec![],
            },
        ];
        let definitions = vec![
            DenomDefinition {
                denom: "denom1".to_string(),
                issuer: "issuer_account_A".to_string(),
                burn_rate: 0.08,
                commission_rate: 0.12,
            },
        ];
        let multi_send_tx = MultiSend {
            inputs: vec![
                Balance {
                    address: "account1".to_string(),
                    coins: vec![Coin { denom: "denom1".to_string(), amount: 1000 }],
                },
            ],
            outputs: vec![
                Balance {
                    address: "account_recipient".to_string(),
                    coins: vec![
                        Coin { denom: "denom1".to_string(), amount: 880 },
                    ],
                },
            ],
        };
        
        let result = calculate_balance_changes(original_balances, definitions, multi_send_tx);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Original coin for denom not found.".to_string());
    }
