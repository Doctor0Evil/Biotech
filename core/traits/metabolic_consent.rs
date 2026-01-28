pub struct MetabolicConsent {
    pub host_id: u64,
    pub fraction_allowed: f32,        // Limited percentage of BRAIN max-capacity
    pub smart_active: bool,           // Automation enabled status
    pub timestamp: u64,
}

impl MetabolicConsent {
    pub fn within_brain_limits(&self, brain_limit: f32) -> bool {
        self.fraction_allowed <= brain_limit && self.smart_active
    }

    pub fn authorize(&mut self, smart_state: bool, brain_limit: f32) {
        self.smart_active = smart_state && self.within_brain_limits(brain_limit);
    }

    pub fn withdraw(&mut self) {
        self.smart_active = false;
    }
}
