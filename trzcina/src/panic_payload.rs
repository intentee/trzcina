use std::any::Any;

pub struct PanicPayload {
    pub payload: Box<dyn Any + Send>,
}

impl PanicPayload {
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.payload
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| self.payload.downcast_ref::<&'static str>().copied())
    }
}
