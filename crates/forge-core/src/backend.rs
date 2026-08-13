use std::fmt::Debug;

pub trait BuildBackend: Debug + Send + Sync {
    fn name(&self) -> &'static str;
}
