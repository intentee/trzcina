use std::any::Any;
use std::panic::resume_unwind;

pub struct DropPanickingValue<TPanicPayload: Any + Copy + Send> {
    pub panic_payload: TPanicPayload,
}

impl<TPanicPayload: Any + Copy + Send> Drop for DropPanickingValue<TPanicPayload> {
    fn drop(&mut self) {
        resume_unwind(Box::new(self.panic_payload))
    }
}
