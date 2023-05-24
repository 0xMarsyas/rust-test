use std::collections::HashMap;

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

// Helper function to find a DenomDefinition for a given denom
fn find_definition(denom: &String, definitions: &Vec<DenomDefinition>) -> Option<&DenomDefinition> {
    for definition in definitions {
        if &definition.denom == denom {
            return Some(definition);
        }
    }
    None
}

// Helper function to find a Balance for a given address
fn find_balance(address: &String, balances: &Vec<Balance>) -> Option<&Balance> {
    for balance in balances {
        if &balance.address == address {
            return Some(balance);
        }
    }
    None
}

pub fn calculate_balance_changes(
    original_balances: Vec<Balance>,
    definitions: Vec<DenomDefinition>,
    mut multi_send_tx: MultiSend,
) -> Result<Vec<Balance>, String> {
    // Initialize the variables
    let mut input_sum: i128 = 0;
    let mut output_sum: i128 = 0;

    // Calculate sum of inputs and outputs
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

    // Verify sum of inputs and outputs
    if input_sum != output_sum {
        return Err("Sum of inputs and outputs does not match.".to_string());
    }

    // Calculate and apply burn and commission
    for input in &mut multi_send_tx.inputs {
        for coin in &mut input.coins {
            let definition = find_definition(&coin.denom, &definitions)
                .ok_or("Could not find DenomDefinition for input coin denom".to_string())?;
            
            // Calculate burn and commission
            let burn = (coin.amount as f64 * definition.burn_rate).round() as i128;
            let commission = (coin.amount as f64 * definition.commission_rate).round() as i128;

            // Apply burn and commission
            coin.amount -= burn;
            coin.amount -= commission;

            // Add burn and commission to issuer balance
            // TO-DO: Add logic to find and update the issuer's balance
        }
    }
    
    // Check if the sender has enough balances to cover the input amount on top of burn_rate and commission_rate
    for input in &multi_send_tx.inputs {
        let original_balance = find_balance(&input.address, &original_balances)
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

    // TO-DO 4: Calculate and apply the burn and commission for the issuer
    Ok(original_balances) // Placeholder: Return the original_balances for now
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_balance_changes() {
        // TO-DO: Add your test code here
    }
}

fn main() {
    println!("Hello, Coreum!");
}
