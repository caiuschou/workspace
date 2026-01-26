use std::sync::Arc;

/// Options for handling list changed notifications.
#[derive(Clone)]
pub struct ListChangedOptions<T> {
    pub auto_refresh: bool,
    pub debounce_ms: Option<u64>,
    pub on_changed: Arc<dyn Fn(Result<Option<Vec<T>>, String>) + Send + Sync>,
}

impl<T> ListChangedOptions<T> {
    pub fn new(
        on_changed: impl Fn(Result<Option<Vec<T>>, String>) + Send + Sync + 'static,
    ) -> Self {
        Self {
            auto_refresh: true,
            debounce_ms: None,
            on_changed: Arc::new(on_changed),
        }
    }

    pub fn with_debounce_ms(mut self, debounce_ms: u64) -> Self {
        self.debounce_ms = Some(debounce_ms);
        self
    }

    pub fn with_auto_refresh(mut self, auto_refresh: bool) -> Self {
        self.auto_refresh = auto_refresh;
        self
    }
}
