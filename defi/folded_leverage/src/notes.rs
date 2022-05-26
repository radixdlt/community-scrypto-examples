// Checks the user's total tokens and deposit balance of those tokens
pub fn check_deposit_balance(&self, user_auth: Proof) -> String {
    let user_badge_data: User = user_auth.non_fungible().data();
    return info!("The user's balance information is: {:?}", user_badge_data.deposit_balance);
}