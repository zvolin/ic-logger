//! A logger that prints all messages with a simple, readable output format.
//!
//! Just initialize logging without any configuration:
//!
//! ```rust
//! ic_logger::init().unwrap();
//! log::warn!("This is an example message.");
//! ```
//!
//! Hardcode a default log level:
//!
//! ```rust
//! ic_logger::init_with_level(log::Level::Warn).unwrap();
//! ```

use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};

/// Implements [`Log`] and a set of simple builder methods for configuration.
///
/// Use the various "builder" methods on this struct to configure the logger,
/// then call [`init`] to configure the [`log`] crate.
pub struct IcLogger {
    /// The default logging level
    default_level: LevelFilter,

    /// The specific logging level for each module
    ///
    /// This is used to override the default value for some specific modules.
    /// After initialization, the vector is sorted so that the first (prefix) match
    /// directly gives us the desired log level.
    module_levels: Vec<(String, LevelFilter)>,
}

impl IcLogger {
    /// Initializes the global logger with a IcLogger instance with
    /// default log level set to `Level::Warn`.
    ///
    /// ```no_run
    /// use ic_logger::IcLogger;
    /// IcLogger::new().init().unwrap();
    /// log::warn!("This is an example message.");
    /// ```
    ///
    /// [`init`]: #method.init
    #[must_use = "You must call init() to begin logging"]
    pub fn new() -> IcLogger {
        IcLogger {
            default_level: LevelFilter::Warn,
            module_levels: Vec::new(),
        }
    }

    /// Set the 'default' log level.
    ///
    /// You can override the default level for specific modules and their sub-modules using [`with_module_level`]
    ///
    /// [`with_module_level`]: #method.with_module_level
    #[must_use = "You must call init() to begin logging"]
    pub fn with_level(mut self, level: LevelFilter) -> IcLogger {
        self.default_level = level;
        self
    }

    /// Override the log level for some specific modules.
    ///
    /// This sets the log level of a specific module and all its sub-modules.
    /// When both the level for a parent module as well as a child module are set,
    /// the more specific value is taken. If the log level for the same module is
    /// specified twice, the resulting log level is implementation defined.
    ///
    /// # Examples
    ///
    /// Silence an overly verbose crate:
    ///
    /// ```no_run
    /// use ic_logger::IcLogger;
    /// use log::LevelFilter;
    ///
    /// IcLogger::new().with_module_level("chatty_dependency", LevelFilter::Warn).init().unwrap();
    /// ```
    ///
    /// Disable logging for all dependencies:
    ///
    /// ```no_run
    /// use ic_logger::IcLogger;
    /// use log::LevelFilter;
    ///
    /// IcLogger::new()
    ///     .with_level(LevelFilter::Off)
    ///     .with_module_level("my_crate", LevelFilter::Info)
    ///     .init()
    ///     .unwrap();
    /// ```
    #[must_use = "You must call init() to begin logging"]
    pub fn with_module_level(mut self, target: &str, level: LevelFilter) -> IcLogger {
        self.module_levels.push((target.to_string(), level));

        /* Normally this is only called in `init` to avoid redundancy, but we can't initialize the logger in tests */
        #[cfg(test)]
        self.module_levels
            .sort_by_key(|(name, _level)| name.len().wrapping_neg());

        self
    }

    /// 'Init' the actual logger, instantiate it and configure it,
    /// this method MUST be called in order for the logger to be effective.
    pub fn init(mut self) -> Result<(), SetLoggerError> {
        /* Sort all module levels from most specific to least specific. The length of the module
         * name is used instead of its actual depth to avoid module name parsing.
         */
        self.module_levels
            .sort_by_key(|(name, _level)| name.len().wrapping_neg());
        let max_level = self.module_levels.iter().map(|(_name, level)| level).copied().max();
        let max_level = max_level
            .map(|lvl| lvl.max(self.default_level))
            .unwrap_or(self.default_level);
        log::set_max_level(max_level);
        log::set_boxed_logger(Box::new(self))?;
        Ok(())
    }
}

impl Default for IcLogger {
    /// See [this](struct.IcLogger.html#method.new)
    fn default() -> Self {
        IcLogger::new()
    }
}

impl Log for IcLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        &metadata.level().to_level_filter()
            <= self
                .module_levels
                .iter()
                /* At this point the Vec is already sorted so that we can simply take
                 * the first match
                 */
                .find(|(name, _level)| metadata.target().starts_with(name))
                .map(|(_name, level)| level)
                .unwrap_or(&self.default_level)
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level_string = format!("{:<5}", record.level().to_string());

            let target = if !record.target().is_empty() {
                record.target()
            } else {
                record.module_path().unwrap_or_default()
            };

            ic_cdk::println!("[{level_string} {target}] {}", record.args());
        }
    }

    fn flush(&self) {}
}

/// Initialise the logger with its default configuration.
///
/// Log messages will not be filtered.
/// The `RUST_LOG` environment variable is not used.
pub fn init() -> Result<(), SetLoggerError> {
    IcLogger::new().init()
}

/// Initialise the logger with a specific log level.
///
/// Log messages below the given [`Level`] will be filtered.
/// The `RUST_LOG` environment variable is not used.
pub fn init_with_level(level: Level) -> Result<(), SetLoggerError> {
    IcLogger::new().with_level(level.to_level_filter()).init()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_module_levels_allowlist() {
        let logger = IcLogger::new()
            .with_level(LevelFilter::Off)
            .with_module_level("my_crate", LevelFilter::Info);

        assert!(logger.enabled(&create_log("my_crate", Level::Info)));
        assert!(logger.enabled(&create_log("my_crate::module", Level::Info)));
        assert!(!logger.enabled(&create_log("my_crate::module", Level::Debug)));
        assert!(!logger.enabled(&create_log("not_my_crate", Level::Debug)));
        assert!(!logger.enabled(&create_log("not_my_crate::module", Level::Error)));
    }

    #[test]
    fn test_module_levels_denylist() {
        let logger = IcLogger::new()
            .with_level(LevelFilter::Debug)
            .with_module_level("my_crate", LevelFilter::Trace)
            .with_module_level("chatty_dependency", LevelFilter::Info);

        assert!(logger.enabled(&create_log("my_crate", Level::Info)));
        assert!(logger.enabled(&create_log("my_crate", Level::Trace)));
        assert!(logger.enabled(&create_log("my_crate::module", Level::Info)));
        assert!(logger.enabled(&create_log("my_crate::module", Level::Trace)));
        assert!(logger.enabled(&create_log("not_my_crate", Level::Debug)));
        assert!(!logger.enabled(&create_log("not_my_crate::module", Level::Trace)));
        assert!(logger.enabled(&create_log("chatty_dependency", Level::Info)));
        assert!(!logger.enabled(&create_log("chatty_dependency", Level::Debug)));
        assert!(!logger.enabled(&create_log("chatty_dependency::module", Level::Debug)));
        assert!(logger.enabled(&create_log("chatty_dependency::module", Level::Warn)));
    }

    fn create_log(name: &str, level: Level) -> Metadata {
        let mut builder = Metadata::builder();
        builder.level(level);
        builder.target(name);
        builder.build()
    }
}
