use scrypto::prelude::*;
use std::cmp;

/// A struct which defines the data for the the `VestingSchedule` non-fungible tokens, which are tokens that the
/// beneficiaries are given in order for the vesting blueprint to keep track of their vesting schedule and the amount of
/// funds which should have vested for them by a given epoch.
///
/// # Note:
///
/// If we had been building this application in a traditional financial system we would use dates to determine when the
/// cliff happens, when the beneficiary was enrolled in the vesting schedule, and other things concerning time. However,
/// we do not have a concept of "date" in Scrypto, instead we have the concept of epochs which we can use to inform us
/// of the passing of time and as a replacement for dates.
#[derive(NonFungibleData)]
pub struct BeneficiaryVestingSchedule {
    /// This variable denotes the epoch which the beneficiary was first enrolled in the vesting schedule. This variable
    /// may be smaller than or equal to the `cliff_epoch` as the cliff can't happen before the enrollment epoch.
    enrollment_epoch: u64,

    /// In vesting schedules, there is a concept of a "cliff period" which is a period of time where the assets do not
    /// vest. Only after the cliff date does the vesting actually begin. This variable defines when the cliff begins
    /// for the vesting schedule of the beneficiary.
    cliff_epoch: u64,

    /// This is the epoch by which the vesting ends. By this epoch, the beneficiary should have received all the funds
    /// that is owed to them.
    end_epoch: u64,

    /// This is the total amount of funds that will be vested by the end of the `ending_epoch`, the beneficiary should
    /// have been given this amount by the contract.
    total_vesting_amount: Decimal,

    /// This is the amount of tokens which will be available for withdraw once the cliff period has ended.
    amount_available_on_cliff: Decimal,
}

impl BeneficiaryVestingSchedule {
    /// Creates the VestingSchedule data
    ///
    /// This function creates a new VestingSchedule and performs the required checks on the passed arguments to ensure
    /// that the provided epochs and vesting amounts are valid.
    ///
    /// A total of `total_vesting_amount` tokens will be vested for the owner of this vesting schedule over a period
    /// beginning in `relative_cliff_epoch` epochs and ending in `relative_ending_epoch` epochs. After
    /// `relative_cliff_epoch` epochs, the owner of this vesting schedule will be given `percentage_available_on_cliff`%
    /// of the `total_vesting_amount` and the remaining funds will be vested over the period.
    ///
    /// This function performs a number of checks before creating a new VestingSchedule:
    ///
    /// * **Check 1:** Checks that the `relative_cliff_epoch` is larger than or equal to 0.
    /// * **Check 2:** Checks that the `relative_ending_epoch` is larger than or equal to the `relative_cliff_epoch`.
    /// * **Check 3:** Checks that the `percentage_available_on_cliff` is between 0 and 1.
    ///
    /// # Returns:
    ///
    /// * `VestingSchedule` - A vesting schedule initialized with the provided data.
    ///
    /// # Note:
    ///
    /// The `relative_cliff_epoch` and `relative_ending_epoch` arguments are both relative arguments. Saying that
    /// `relative_cliff_epoch = 10` means that the cliff beings after 10 epochs from the current epoch. It does
    /// **NOT** mean that the cliff begins in epoch 10. Similarity, a `relative_ending_epoch = 100` means that the
    /// ending of the vesting period is 100 epochs from the current epoch. It does **NOT** mean that the end epoch
    /// is epoch 100.
    pub fn new(
        relative_cliff_epoch: u64,
        relative_ending_epoch: u64,
        total_vesting_amount: Decimal,
        percentage_available_on_cliff: Decimal,
    ) -> Self {
        // Performing the checks to ensure that the vesting schedule may be created.
        assert!(
            relative_cliff_epoch >= 0,
            "[New Vesting Schedule]: Relative cliff epoch must be larger than or equal to zero."
        );
        assert!(
            relative_ending_epoch >= relative_cliff_epoch,
            "[New Vesting Schedule]: Relative ending epoch must be larger than or equal to the relative cliff epoch."
        );
        assert!(
            (percentage_available_on_cliff >= dec!("0")) && (percentage_available_on_cliff <= dec!("1")),
            "[New Vesting Schedule]: The percentage of funds available on cliff must be a value between 0 and 1"
        );

        // Converting the relative epochs to absolute epochs
        let enrollment_epoch: u64 = Runtime::current_epoch();
        let cliff_epoch: u64 = enrollment_epoch + relative_cliff_epoch;
        let end_epoch: u64 = enrollment_epoch + relative_ending_epoch;

        // Creating the vesting schedule
        return Self {
            enrollment_epoch,
            cliff_epoch,
            end_epoch,
            total_vesting_amount,
            amount_available_on_cliff: total_vesting_amount * percentage_available_on_cliff,
        };
    }

    /// Calculates and returns the vesting gradient.
    ///
    /// This method calculates the gradient of the linear vesting schedule from the cliff epoch, end epoch, cliff
    /// amount, total amount.
    ///
    /// # Returns:
    ///
    /// * `Decimal` - A decimal of the gradient of the vesting schedule.
    pub fn vesting_gradient(&self) -> Decimal {
        return (self.total_vesting_amount - self.amount_available_on_cliff) / (self.end_epoch - self.cliff_epoch);
    }

    /// Calculates and returns the total amount vested by a given epoch
    ///
    /// # Arguments:
    ///
    /// * `epoch` (u64) - The epoch for which we want to determine the total vested amount.
    ///
    /// # Returns:
    ///
    /// * `Decimal` - The amount of tokens vested so far.
    pub fn get_vested_amount(&self, epoch: u64) -> Decimal {
        // If the cliff epoch has not come yet, then the amount vested is zero. Otherwise the amount is the minimum of
        // the linear vesting equation and the total vesting amount.
        return if epoch < self.cliff_epoch {
            dec!("0")
        } else {
            cmp::min(
                self.vesting_gradient() * (epoch - self.cliff_epoch) + self.amount_available_on_cliff,
                self.total_vesting_amount,
            )
        };
    }

    /// Calculates and returns the amount unvested tokens.
    ///
    /// # Arguments:
    ///
    /// * `epoch` (u64) - The epoch for which we want to determine the total unvested amount.
    ///
    /// # Returns:
    ///
    /// * `Decimal` - The amount of tokens vested so far.
    pub fn get_unvested_amount(&self, epoch: u64) -> Decimal {
        return self.total_vesting_amount - self.get_vested_amount(epoch);
    }
}
