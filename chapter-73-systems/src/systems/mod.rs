mod dispatch;
pub use dispatch::UnifiedDispatcher;

pub fn build() -> Box<dyn UnifiedDispatcher + 'static> {
    Box::new(dispatch::SingleThreadedDispatcher::new())
}