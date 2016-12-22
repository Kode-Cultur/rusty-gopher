# rusty-gopher
rusty-gopher is a gopher daemon written in rust.

## Building

As of now (2016-12-22) you need rust-nightly to build rusty-gopher. The recommended way to install/manage your rust installation is [rustup](https://www.rustup.rs/).

rusty-gopher hasn't been published on [crates.io](https://crates.io) yet, so you have to manually clone it.

```sh
git clone https://gitlab.glaxx.net/glaxx/rusty-gopher.git
cd rusty-gopher
cargo build --release
```

## Configuration

The configuration is quite simple, as there is nothing much to configure. An example configuration may look as follows:

```ini
[General]
# If you choose to let your gopher daemon listen on the standard port (70, 
# anything below 1024) you have to be root, which is not desirable after
# binding to that port. So rusty-gopher will change its user corresponding to
# the following config value.
user=gopher

# The root directory in which your files are located.
rootdir=/var/gopher

# You can specify on which address:port your gopher daemon should listen.
listento=0.0.0.0:70
```

You may generate an empty configuration file by typing:

```sh
rusty-gopher genconfig [<path>]
```

## Running

```sh
# If no config file is specified /etc/rusty_gopher.cfg will be read.
rusty-gopher serve [<path to config file>]
```

## Adding content / gophermaps

This feature hasn't been implemented yet.
