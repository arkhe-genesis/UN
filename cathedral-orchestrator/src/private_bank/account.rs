use super::types::PrivateAccount;

pub struct AccountManager {
    // Exemplo de estado local para contas privadas.
}

impl AccountManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_account(&self, _agent_id: &[u8; 32]) -> Option<PrivateAccount> {
        None
    }
}
