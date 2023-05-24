pub struct MultiSend {
    inputs: Vec<Balance>,
    outputs: Vec<Balance>,
}

pub struct Coin {
    pub denom: String,
    pub amount: i128,
}

pub struct Balance {
    address: String,
    coins: Vec<Coin>,
}

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
                .ok_or("Could not find DenomDefinition for input coin denom".to_string())?;

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
                return Err("Sender does not have enough balance to cover the input amount on top of burn_rate and commission_rate".to_string());
            }
        }
    }

    Ok(original_balances) 
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_calculate_balance_changes_example1() {
        let mut rng = rand::thread_rng();
        let original_balances = vec![
            //...
        ];
        //...
        // Original test implementation
        //...
    }

    #[test]
    fn test_input_output_sum_mismatch() {
        let mut rng = rand::thread_rng();
        let original_balances = vec![
            //...
        ];
        // Test case implementation where the sum of inputs does not equal sum of outputs
    }

    #[test]
    fn test_no_denom_definition_for_input_coin() {
        let mut rng = rand::thread_rng();
        let original_balances = vec![
            //...
        ];
        // Test case implementation where a denom of an input coin does not have a corresponding DenomDefinition
    }

    #[test]
    fn test_no_balance_for_issuer() {
        let mut rng = rand::thread_rng();
        let original_balances = vec![
            //...
        ];
        // Test case implementation where the issuer does not have an existing balance in the original balances list
    }

    #[test]
    fn test_not_enough_balance_to_cover_input() {
        let mut rng = rand::thread_rng();
        let original_balances = vec![
            //...
        ];
        // Test case implementation where the sender does not have enough balance to cover the input amount on top of burn_rate and commission_rate
    }

    #[test]
    fn test_no_original_coin_for_denom() {
        let mut rng = rand::thread_rng();
        let original_balances = vec![
            //...
        ];
        // Test case implementation where a denom of a coin does not exist in the original balances list
    }
}


fn main() {
    println!("Hello, Coreum!");
}
