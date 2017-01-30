# dzendemux
Configurable status bar demultiplexor

Prints to stdout the line of data combined from different watchers, like clock or some file content
(cuttently only latter two are inplemented).

Configurable through toml-file, allows setting update frequency for each watcher.
Line format is specified as Liquid template, so this program can be used as a input for any other status bar
like i3bar.

Config file path can be secified as first argument, but if there is not any, program searches for
$HOME/.dzendemux.toml and fallbacks to dzendemux.toml in current directory if such file was not found.

Example config can be seen in current repository's root.

Liquid templates also allows simple logic, so some basic sctipting is even possible without recompilation.

Written in Rust, using futures and tokio.
