pub struct ConfirmationRuleState {
    quorum: f64,
    current_tip: String,
}

impl ConfirmationRuleState {
    pub fn new(quorum: f64, tip: String) -> Self {
        Self {
            quorum: quorum,
            current_tip: tip,
        }
    }
}