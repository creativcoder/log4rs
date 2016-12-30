//! log4rs is a highly configurable logging framework modeled after Java's
//! Logback and log4j libraries.
//!
//! # Architecture
//!
//! The basic units of configuration are *appenders*, *encoders*, *filters*, and
//! *loggers*.
//!
//! ## Appenders
//!
//! An appender takes a log record and logs it somewhere, for example, to a
//! file, the console, or the syslog.
//!
//! ## Encoders
//!
//! An encoder is responsible for taking a log record, transforming it into the
//! appropriate output format, and writing it out. An appender will normally
//! use an encoder internally.
//!
//! ## Filters
//!
//! Filters are associated with appenders and, like the name would suggest,
//! filter log events coming into that appender.
//!
//! ## Loggers
//!
//! A log event is targeted at a specific logger, which are identified by
//! string names. The logging macros built in to the `log` crate set the logger
//! of a log event to the one identified by the module containing the
//! invocation location.
//!
//! Loggers form a hierarchy: logger names are divided into components by "::".
//! One logger is the ancestor of another if the first logger's component list
//! is a prefix of the second logger's component list.
//!
//! Loggers are associated with a maximum log level. Log events for that logger
//! with a level above the maximum will be ignored. The maximum log level for
//! any logger can be configured manually; if it is not, the level will be
//! inherited from the logger's parent.
//!
//! Loggers are also associated with a set of appenders. Appenders can be
//! associated directly with a logger. In addition, the appenders of the
//! logger's parent will be associated with the logger unless the logger has
//! its *additivity* set to `false`. Log events sent to the logger that are not
//! filtered out by the logger's maximum log level will be sent to all
//! associated appenders.
//!
//! The "root" logger is the ancestor of all other loggers. Since it has no
//! ancestors, its additivity cannot be configured.
//!
//! # Configuration
//!
//! log4rs can be configured programmatically by using the builders in the
//! `config` module to construct a log4rs `Config` object, which can be passed
//! to the `init_config` function.
//!
//! The more common configuration method, however, is via a separate config
//! file. The `init_file` function takes the path to a config file as
//! well as a `Deserializers` object which is responsible for instantiating the
//! various objects specified by the config file. The `file` module
//! documentation covers the exact configuration syntax, but an example in the
//! YAML format is provided below.
//!
//! log4rs makes heavy use of Cargo features to enable consumers to pick the
//! functionality they wish to use. File-based configuration requires the `file`
//! feature, and each file format requires its own feature as well. In addition,
//! each component has its own feature. For example, YAML support requires the
//! `yaml_format` feature and the console appender requires the
//! `console_appender` feature.
//!
//! By default, the `console_appender`, `file_appender`, `pattern_encoder`,
//! `file`, and `yaml_format` features are enabled.
//!
//! As a convenience, the `all_components` feature activates all logger components.
//!
//! # Examples
//!
//! Configuration via a YAML file:
//!
//! ```yaml
//! # Scan this file for changes every 30 seconds
//! refresh_rate: 30 seconds
//!
//! appenders:
//!   # An appender named "stdout" that writes to stdout
//!   stdout:
//!     kind: console
//!
//!   # An appender named "requests" that writes to a file with a custom pattern encoder
//!   requests:
//!     kind: file
//!     path: "log/requests.log"
//!     encoder:
//!       pattern: "{d} - {m}{n}"
//!
//! # Set the default logging level to "warn" and attach the "stdout" appender to the root
//! root:
//!   level: warn
//!   appenders:
//!     - stdout
//!
//! loggers:
//!   # Raise the maximum log level for events sent to the "app::backend::db" logger to "info"
//!   app::backend::db:
//!     level: info
//!
//!   # Route log events sent to the "app::requests" logger to the "requests" appender,
//!   # and *not* the normal appenders installed at the root
//!   app::requests:
//!     level: info
//!     appenders:
//!       - requests
//!     additive: false
//! ```
//!
//! ```no_run
//! # #[cfg(feature = "file")]
//! # fn f() {
//! log4rs::init_file("log4rs.yml", Default::default()).unwrap();
//! # }
//! ```
//!
//! Programmatically constructing a configuration:
//!
//! ```no_run
//! extern crate log;
//! extern crate log4rs;
//!
//! # #[cfg(all(feature = "console_appender",
//! #           feature = "file_appender",
//! #           feature = "pattern_encoder"))]
//! # fn f() {
//! use log::LogLevelFilter;
//! use log4rs::append::console::ConsoleAppender;
//! use log4rs::append::file::FileAppender;
//! use log4rs::encode::pattern::PatternEncoder;
//! use log4rs::config::{Appender, Config, Logger, Root};
//!
//! fn main() {
//!     let stdout = ConsoleAppender::builder().build();
//!
//!     let requests = FileAppender::builder()
//!         .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
//!         .build("log/requests.log")
//!         .unwrap();
//!
//!     let config = Config::builder()
//!         .appender(Appender::builder().build("stdout", Box::new(stdout)))
//!         .appender(Appender::builder().build("requests", Box::new(requests)))
//!         .logger(Logger::builder().build("app::backend::db", LogLevelFilter::Info))
//!         .logger(Logger::builder()
//!             .appender("requests")
//!             .additive(false)
//!             .build("app::requests", LogLevelFilter::Info))
//!         .build(Root::builder().appender("stdout").build(LogLevelFilter::Warn))
//!         .unwrap();
//!
//!     let handle = log4rs::init_config(config).unwrap();
//!
//!     // use handle to change logger configuration at runtime
//! }
//! # }
//! # fn main() {}
//! ```
#![doc(html_root_url="https://sfackler.github.io/log4rs/doc/v0.5.2")]
#![warn(missing_docs)]

#[cfg(feature = "antidote")]
extern crate antidote;
extern crate crossbeam;
extern crate fnv;
#[cfg(feature = "humantime")]
extern crate humantime;
extern crate log;
#[cfg(feature = "log-mdc")]
extern crate log_mdc;
#[cfg(feature = "typemap")]
extern crate typemap;
#[cfg(feature = "chrono")]
extern crate chrono;
#[cfg(feature = "flate2")]
extern crate flate2;
#[cfg(all(windows, feature = "kernel32-sys"))]
extern crate kernel32;
#[cfg(all(not(windows), feature = "libc"))]
extern crate libc;
#[cfg(feature = "serde")]
extern crate serde;
#[cfg(feature = "serde_yaml")]
extern crate serde_yaml;
#[cfg(feature = "serde_json")]
extern crate serde_json;
#[cfg(feature = "serde-value")]
extern crate serde_value;
#[cfg(feature = "toml")]
extern crate toml;
#[cfg(all(windows, feature = "winapi"))]
extern crate winapi;

#[cfg(test)]
extern crate tempdir;

use crossbeam::sync::ArcCell;
use fnv::FnvHasher;
use std::cmp;
use std::collections::HashMap;
use std::error;
use std::hash::BuildHasherDefault;
use std::io;
use std::io::prelude::*;
use std::sync::Arc;
use log::{LogLevel, LogMetadata, LogRecord, LogLevelFilter, SetLoggerError, MaxLogLevelFilter};

#[cfg(feature = "file")]
pub use priv_file::{init_file, Error};

use append::Append;
use config::Config;
use filter::Filter;

pub mod append;
pub mod config;
pub mod filter;
#[cfg(feature = "file")]
pub mod file;
pub mod encode;
#[cfg(feature = "file")]
mod priv_serde;
#[cfg(feature = "file")]
mod priv_file;
#[cfg(feature = "console_writer")]
mod priv_io;

type FnvHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FnvHasher>>;

struct ConfiguredLogger {
    level: LogLevelFilter,
    appenders: Vec<usize>,
    children: FnvHashMap<String, ConfiguredLogger>,
}

impl ConfiguredLogger {
    fn add(&mut self,
           path: &str,
           mut appenders: Vec<usize>,
           additive: bool,
           level: LogLevelFilter) {
        let (part, rest) = match path.find("::") {
            Some(idx) => (&path[..idx], &path[idx + 2..]),
            None => (path, ""),
        };

        if let Some(child) = self.children.get_mut(part) {
            child.add(rest, appenders, additive, level);
            return;
        }

        let child = if rest.is_empty() {
            if additive {
                appenders.extend(self.appenders.iter().cloned());
            }

            ConfiguredLogger {
                level: level,
                appenders: appenders,
                children: FnvHashMap::default(),
            }
        } else {
            let mut child = ConfiguredLogger {
                level: self.level,
                appenders: self.appenders.clone(),
                children: FnvHashMap::default(),
            };
            child.add(rest, appenders, additive, level);
            child
        };

        self.children.insert(part.to_owned(), child);
    }

    fn max_log_level(&self) -> LogLevelFilter {
        let mut max = self.level;
        for child in self.children.values() {
            max = cmp::max(max, child.max_log_level());
        }
        max
    }

    fn find(&self, path: &str) -> &ConfiguredLogger {
        let mut node = self;

        for part in path.split("::") {
            match node.children.get(part) {
                Some(child) => node = child,
                None => break,
            }
        }

        node
    }

    fn enabled(&self, level: LogLevel) -> bool {
        self.level >= level
    }

    fn log(&self, record: &log::LogRecord, appenders: &[Appender]) {
        if self.enabled(record.level()) {
            for &idx in &self.appenders {
                if let Err(err) = appenders[idx].append(record) {
                    handle_error(&*err);
                }
            }
        }
    }
}

struct Appender {
    appender: Box<Append>,
    filters: Vec<Box<Filter>>,
}

impl Appender {
    fn append(&self, record: &LogRecord) -> Result<(), Box<error::Error>> {
        for filter in &self.filters {
            match filter.filter(record) {
                filter::Response::Accept => break,
                filter::Response::Neutral => {}
                filter::Response::Reject => return Ok(()),
            }
        }

        self.appender.append(record)
    }
}

struct SharedLogger {
    root: ConfiguredLogger,
    appenders: Vec<Appender>,
}

impl SharedLogger {
    fn new(config: config::Config) -> SharedLogger {
        let (appenders, root, mut loggers) = config.unpack();

        let root = {
            let appender_map = appenders.iter()
                .enumerate()
                .map(|(i, appender)| (appender.name(), i))
                .collect::<HashMap<_, _>>();

            let mut root = ConfiguredLogger {
                level: root.level(),
                appenders: root.appenders()
                    .iter()
                    .map(|appender| appender_map[&**appender])
                    .collect(),
                children: FnvHashMap::default(),
            };

            // sort loggers by name length to ensure that we initialize them top to bottom
            loggers.sort_by_key(|l| l.name().len());
            for logger in loggers {
                let appenders = logger.appenders()
                    .iter()
                    .map(|appender| appender_map[&**appender])
                    .collect();
                root.add(logger.name(), appenders, logger.additive(), logger.level());
            }

            root
        };

        let appenders = appenders.into_iter()
            .map(|appender| {
                let (_, appender, filters) = appender.unpack();
                Appender {
                    appender: appender,
                    filters: filters,
                }
            })
            .collect();

        SharedLogger {
            root: root,
            appenders: appenders,
        }
    }
}

struct Logger(Arc<ArcCell<SharedLogger>>);

impl Logger {
    fn new(config: config::Config) -> Logger {
        Logger(Arc::new(ArcCell::new(Arc::new(SharedLogger::new(config)))))
    }

    fn max_log_level(&self) -> LogLevelFilter {
        self.0.get().root.max_log_level()
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        self.enabled_inner(metadata.level(), metadata.target())
    }

    fn log(&self, record: &log::LogRecord) {
        let shared = self.0.get();
        shared.root.find(record.target()).log(record, &shared.appenders);
    }
}

impl Logger {
    fn enabled_inner(&self, level: LogLevel, target: &str) -> bool {
        self.0.get().root.find(target).enabled(level)
    }
}

fn handle_error<E: error::Error + ?Sized>(e: &E) {
    let _ = writeln!(io::stderr(), "log4rs: {}", e);
}

/// Initializes the global logger as a log4rs logger with the provided config.
///
/// A `Handle` object is returned which can be used to adjust the logging
/// configuration.
pub fn init_config(config: config::Config) -> Result<Handle, SetLoggerError> {
    let mut handle = None;
    log::set_logger(|max_log_level| {
            let logger = Logger::new(config);
            max_log_level.set(logger.max_log_level());
            handle = Some(Handle {
                shared: logger.0.clone(),
                max_log_level: max_log_level,
            });
            Box::new(logger)
        })
        .map(|()| handle.unwrap())
}

/// A handle to the active logger.
pub struct Handle {
    shared: Arc<ArcCell<SharedLogger>>,
    max_log_level: MaxLogLevelFilter,
}

impl Handle {
    /// Sets the logging configuration.
    pub fn set_config(&self, config: Config) {
        let shared = SharedLogger::new(config);
        self.max_log_level.set(shared.root.max_log_level());
        self.shared.set(Arc::new(shared));
    }
}

trait ErrorInternals {
    fn new(message: String) -> Self;
}

trait ConfigPrivateExt {
    fn unpack(self) -> (Vec<config::Appender>, config::Root, Vec<config::Logger>);
}

trait PrivateConfigErrorsExt {
    fn unpack(self) -> Vec<config::Error>;
}

trait PrivateConfigAppenderExt {
    fn unpack(self) -> (String, Box<Append>, Vec<Box<Filter>>);
}

#[cfg(test)]
mod test {
    use log::{LogLevel, LogLevelFilter};

    use super::*;

    #[test]
    fn enabled() {
        let root = config::Root::builder().build(LogLevelFilter::Debug);
        let mut config = config::Config::builder();
        let logger = config::Logger::builder().build("foo::bar", LogLevelFilter::Trace);
        config = config.logger(logger);
        let logger = config::Logger::builder().build("foo::bar::baz", LogLevelFilter::Off);
        config = config.logger(logger);
        let logger = config::Logger::builder().build("foo::baz::buz", LogLevelFilter::Error);
        config = config.logger(logger);
        let config = config.build(root).unwrap();

        let logger = super::Logger::new(config);

        assert!(logger.enabled_inner(LogLevel::Warn, "bar"));
        assert!(!logger.enabled_inner(LogLevel::Trace, "bar"));
        assert!(logger.enabled_inner(LogLevel::Debug, "foo"));
        assert!(logger.enabled_inner(LogLevel::Trace, "foo::bar"));
        assert!(!logger.enabled_inner(LogLevel::Error, "foo::bar::baz"));
        assert!(logger.enabled_inner(LogLevel::Debug, "foo::bar::bazbuz"));
        assert!(!logger.enabled_inner(LogLevel::Error, "foo::bar::baz::buz"));
        assert!(!logger.enabled_inner(LogLevel::Warn, "foo::baz::buz"));
        assert!(!logger.enabled_inner(LogLevel::Warn, "foo::baz::buz::bar"));
        assert!(logger.enabled_inner(LogLevel::Error, "foo::baz::buz::bar"));
    }
}
