
error_chain! {
    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error) #[cfg(unix)];
        TomlDecode(::toml::DecodeError);
    }

    errors {
        CoreRunError
        ConfigError(t: &'static str) {
            description("config file error")
            display("config file error: {}", t)
        }
    }
}
