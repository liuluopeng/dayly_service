// Custom formatter for HTTP logs
use chrono::Local;
use std::fmt;
use tracing::{Event, field::Field};
use tracing_subscriber::field::RecordFields;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, format::Writer};
use tracing_subscriber::registry::LookupSpan;

pub struct HttpFormatter;

impl<S, N> FormatEvent<S, N> for HttpFormatter
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut w: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        use tracing::Level;
        let meta = event.metadata();

        let level = match *meta.level() {
            Level::ERROR => "\x1b[31mERROR\x1b[0m",
            Level::WARN => "\x1b[33mWARN\x1b[0m",
            Level::INFO => "\x1b[32mINFO\x1b[0m",
            Level::DEBUG => "\x1b[34mDEBUG\x1b[0m",
            Level::TRACE => "\x1b[90mTRACE\x1b[0m",
        };

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        write!(w, "{} [{}] ", level, timestamp)?;
        ctx.format_fields(w.by_ref(), event)?;
        writeln!(w)
    }
}

pub struct ColoredFields;

impl<'writer> FormatFields<'writer> for ColoredFields {
    fn format_fields<R: RecordFields>(&self, mut w: Writer<'writer>, fields: R) -> fmt::Result {
        use tracing::field::Visit;

        struct StatusColorVisitor<'a, 'b>(&'a mut Writer<'b>);

        impl<'a, 'b> Visit for StatusColorVisitor<'a, 'b> {
            fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
                let name = field.name();

                if name == "status" {
                    if let Ok(code) = format!("{:?}", value).parse::<u64>() {
                        let color = match code {
                            200..=299 => "\x1b[32m",
                            300..=399 => "\x1b[36m",
                            400..=499 => "\x1b[33m",
                            _ => "\x1b[31m",
                        };
                        let _ = write!(self.0, " {}={}{}\x1b[0m", name, color, code);
                    } else {
                        let _ = write!(self.0, " {}={:?}", name, value);
                    }
                } else {
                    let _ = write!(self.0, " {}={:?}", name, value);
                }
            }
        }

        fields.record(&mut StatusColorVisitor(&mut w));
        Ok(())
    }
}
