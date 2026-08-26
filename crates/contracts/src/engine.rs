//! Contract registry and dispatch.

use agora_domain::ids::ContractId;

/// Command a contract asks the runtime to execute on its behalf.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum ContractCommand {
    /// Placeholder until Phase 10 defines the command set.
    Noop {
        /// Originating contract.
        contract: ContractId,
    },
}

/// Deterministic contract state machine.
pub trait Contract {
    /// Input the contract reacts to.
    type Input;

    /// Advances the state machine; returns commands to execute.
    fn on_input(&mut self, input: &Self::Input) -> Vec<ContractCommand>;
}
