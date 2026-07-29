use crate::frb_generated::StreamSink;
use std::sync::RwLock;
use tracing::Subscriber;
use tracing_subscriber::Layer;

static SINK: RwLock<Option<StreamSink<String>>> = RwLock::new(None);

/// 自定义 tracing Layer，把事件格式化后推到 Dart 的 StreamSink
struct DartLayer;

impl<S: Subscriber> Layer<S> for DartLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let sink = match SINK.read() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let Some(sink) = sink.as_ref() else { return };

        let mut visitor = FieldVisitor(String::new());
        event.record(&mut visitor);

        let meta = event.metadata();
        let level = match *meta.level() {
            tracing::Level::ERROR => "E",
            tracing::Level::WARN => "W",
            tracing::Level::INFO => "I",
            tracing::Level::DEBUG => "D",
            tracing::Level::TRACE => "T",
        };
        let target = meta.target();
        let msg = visitor.0;

        let line = if msg.is_empty() {
            format!("[{level}] {target}")
        } else {
            format!("[{level}] {target}: {msg}")
        };

        let _ = sink.add(line);
    }
}

struct FieldVisitor(String);

impl tracing::field::Visit for FieldVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        self.0.push_str(&format!("{}={:?}", field.name(), value));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        self.0.push_str(&format!("{}={}", field.name(), value));
    }
}

#[flutter_rust_bridge::frb(sync)]
pub fn init_rust_logger(sink: StreamSink<String>) {
    let mut guard = match SINK.write() {
        Ok(val) => val,
        Err(val) => val.into_inner(),
    };
    *guard = Some(sink);
    drop(guard);

    use tracing_subscriber::layer::SubscriberExt;
    let subscriber = tracing_subscriber::registry().with(DartLayer);
    let _ = tracing::subscriber::set_global_default(subscriber);
}
