pub fn set_up_logging(default_log_level: log::LevelFilter) -> anyhow::Result<()> {
    use fern::colors::{Color, ColoredLevelConfig};

    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        
        .info(Color::White)
        .debug(Color::White)
        
        .trace(Color::BrightBlack);

    let colors_level = colors_line.info(Color::Green);

    let console_out = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_line}[{date}][{target}][{level}{color_line}] {message}\x1B[0m",
                color_line = format_args!(
                    "\x1B[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
                date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                target = record.target(),
                level = colors_level.color(record.level()),
                message = message,
            ));
        })

        .level(default_log_level)

        .chain(std::io::stdout());

    let file_out = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{date}][{target}][{level}] {message}",
                date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                target = record.target(),
                level = record.level(),
                message = message,
            ));
        })
        .level(log::LevelFilter::Trace)
        .level_for("async_io", log::LevelFilter::Warn)
        .level_for("polling", log::LevelFilter::Warn)
        .chain(
            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open("output.log")
                .unwrap(),
        );

    fern::Dispatch::new()
        .chain(console_out)
        .chain(file_out)
        .apply()
        .map_err(|err| anyhow::anyhow!("{:?}", err))
}
