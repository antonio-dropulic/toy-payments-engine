mod utils;

use utils::{
    generate_deposit_dispute_resolve, generate_deposit_withdraw,
    generate_deposits_dispute_resolve_many_acc, generate_deposits_many_acc,
    generate_deposits_withdraw_many_acc,
};

use crate::utils::generate_deposits;

fn main() {
    let _res = std::fs::create_dir_all("test_data/inputs");
    let _res = std::fs::create_dir("test_data/outputs");

    generate_deposits("test_data/inputs/deposit.csv");
    generate_deposit_withdraw("test_data/inputs/deposit_withdraw.csv");
    generate_deposit_dispute_resolve("test_data/inputs/deposit_dispute_resolve.csv");

    generate_deposits_many_acc("test_data/inputs/deposit_many_acc.csv");
    generate_deposits_withdraw_many_acc("test_data/inputs/deposit_withdraw_many_acc.csv");
    generate_deposits_dispute_resolve_many_acc(
        "test_data/inputs/deposit_dispute_resolve_many_acc.csv",
    );
}
