// Test fixture with structurally similar code (Type-2 clones)

fn process_customer(customer_id: u64) -> Result<CustomerData, Error> {
    let customer = database.fetch_customer(customer_id)?;
    let orders = database.fetch_orders(customer_id)?;
    let preferences = database.fetch_preferences(customer_id)?;
    
    Ok(CustomerData {
        customer,
        orders,
        preferences,
    })
}

// Structurally similar but with renamed variables and functions
fn handle_user(user_key: u64) -> Result<UserInfo, Error> {
    let user = db.get_user(user_key)?;
    let purchases = db.get_purchases(user_key)?;
    let settings = db.get_settings(user_key)?;
    
    Ok(UserInfo {
        user,
        purchases,
        settings,
    })
}

// Another similar pattern with modifications
fn load_account(account_num: u64) -> Result<AccountDetails, Error> {
    let account = repo.find_account(account_num)?;
    let transactions = repo.find_transactions(account_num)?;
    // Added line - Type-3 clone
    log::debug!("Loading account {}", account_num);
    let config = repo.find_config(account_num)?;
    
    Ok(AccountDetails {
        account,
        transactions,
        config,
    })
}