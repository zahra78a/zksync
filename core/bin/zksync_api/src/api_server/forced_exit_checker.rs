use crate::api_server::tx_sender::SubmitError;
use zksync_config::ZkSyncConfig;
use zksync_storage::StorageProcessor;
use zksync_types::Address;

use crate::internal_error;

use chrono::Utc;
#[derive(Clone)]
pub struct ForcedExitChecker {
    /// Mimimum age of the account for `ForcedExit` operations to be allowed.
    pub forced_exit_minimum_account_age: chrono::Duration,
}

impl ForcedExitChecker {
    pub fn new(config: &ZkSyncConfig) -> Self {
        let forced_exit_minimum_account_age = chrono::Duration::seconds(
            config.api.common.forced_exit_minimum_account_age_secs as i64,
        );

        Self {
            forced_exit_minimum_account_age,
        }
    }

    pub async fn check_forced_exit<'a>(
        &self,
        storage: &mut StorageProcessor<'a>,
        target_account_address: Address,
    ) -> Result<(), SubmitError> {
        let account_age = storage
            .chain()
            .operations_ext_schema()
            .account_created_on(&target_account_address)
            .await
            .map_err(|err| internal_error!(err, target_account_address))?;

        match account_age {
            Some(age) if Utc::now() - age < self.forced_exit_minimum_account_age => {
                let msg = format!(
                    "Target account exists less than required minimum amount ({} hours)",
                    self.forced_exit_minimum_account_age.num_hours()
                );

                Err(SubmitError::InvalidParams(msg))
            }
            None => Err(SubmitError::invalid_params("Target account does not exist")),

            Some(..) => Ok(()),
        }
    }
}
