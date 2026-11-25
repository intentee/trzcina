use std::panic::resume_unwind;

pub struct DropPanickingValue {
    pub panic_message: &'static str,
}

impl Drop for DropPanickingValue {
    fn drop(&mut self) {
        resume_unwind(Box::new(self.panic_message))
    }
}
