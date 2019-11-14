#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

mod proto;

#[cfg(not(feature = "client"))]
mod server;

#[cfg(feature = "client")]
mod mofos;

#[cfg(feature = "client")]
mod client;

fn main() {
    main::main()
}

fn common_init() {
    env_logger::init();
}

#[cfg(feature = "client")]
mod main {
    use std::env;
    use std::ffi::OsStr;
    use std::net::SocketAddr;
    use std::process;

    use log::{error, info};

    use self::client::Client;
    use self::mofos::MofosFS;

    use super::client;
    use super::common_init;
    use super::mofos;

    struct MofosConfig {
        fuse_args: Vec<String>,
        host: String,
        rdir: String,
        ldir: String,
        port: u16,
    }

    pub fn main() {
        common_init();

        info!("client mode enabled");

        match arg_parse(env::args().collect()) {
            Ok(config) => {
                let MofosConfig {
                    fuse_args,
                    host,
                    rdir,
                    ldir,
                    port,
                } = config;

                let spec = format!("{}:{}", host, port);
                let addr = spec.parse().expect("invalid host specification");

                let fuse: Vec<&OsStr> = fuse_args.iter().map(|x| OsStr::new(x)).collect();

                let fs = MofosFS::new(addr, &rdir);
                let client = Client::new(addr);

                if let Err(e) = fuse::mount(fs, &ldir, fuse.as_slice()) {
                    println!("{}", e);
                    process::exit(1);
                } else {
                    info!("mofos exiting");
                }
            }

            Err(s) => {
                error!("bad argument: {}", s);
                process::exit(127);
            }
        }
    }

    fn arg_parse(args: Vec<String>) -> Result<MofosConfig, String> {
        let mut fuse_args = Vec::new();
        let mut host = None;
        let mut directory = None;
        let mut local = None;
        let mut port = Some(22);

        for arg in args {
            if arg.contains(":") {
                let split: Vec<&str> = arg.split(":").collect();

                if split.len() != 2 {
                    return Err(String::from("invalid host format"));
                } else {
                    host = Some(String::from(split[0]));
                    directory = Some(String::from(split[1]));
                }
            } else if arg.starts_with("-p=") {
                let v: Vec<&str> = arg.split("=").collect();
                let p = String::from(v[1]);

                port = Some(p.parse::<u16>().unwrap());
            } else if arg[0] != "-" {
                fuse_args.push(String::from(arg));
            } else {
                local = arg.clone();
            }
        }

        let host = host.expect("missing remote host");
        let local = local.expect("local mountpoint unspecified");
        let directory = directory.expect("missing remote directory");
        let port = port.expect("missing destination port");

        let config = MofosConfig {
            fuse_args,
            host,
            ldir: local,
            rdir: directory,
            port,
        };

        info!(
            "mounting {} on {} using remote port {}",
            directory, local, port
        );

        Ok(config)
    }
}

#[cfg(not(feature = "client"))]
mod main {
    use std::env;
    use std::path::Path;
    use std::process;

    use super::common_init;
    use super::server::MofosServer;

    use log::{error, info};

    #[derive(Default)]
    struct MofosConfig {
        port: u16,
        directory: String,
    }

    pub fn main() {
        common_init();
        info!("server mode enabled");

        match parse_args(env::args().collect()) {
            Ok(config) => {
                let addr = format!("0.0.0.0:{}", config.port)
                    .parse()
                    .expect("bad port: {}");
                let mut server = MofosServer::new(addr, Path::new(&config.directory))
                    .expect("failed to setup server");

                match server.run() {
                    Ok(()) => info!("server exited correctly"),
                    Err(e) => error!("server failed: {}", e),
                }
            }

            Err(e) => {
                error!("bad argument: {}", e);
                process::exit(127);
            }
        }
    }

    fn parse_args(args: Vec<String>) -> Result<MofosConfig, String> {
        let mut config = MofosConfig::default();

        if args.len() == 1 {
            usage(&args[0]);
            process::exit(127);
        }

        for mut i in 1..args.len() {
            if args[i] == "-p" || args[i] == "--port" {
                if i == args.len() - 1 {
                    return Err(String::from("missing required argument for -p"));
                }

                if let Ok(port) = args[i + 1].parse::<u16>() {
                    config.port = port;
                } else {
                    return Err(String::from(format!("invalid port {}", args[i + 1])));
                }
            } else if args[i] == "-t" || args[i] == "--target" {
                if i == args.len() - 1 {
                    return Err(String::from("missing required argument for -t"));
                }
                config.directory = String::from(args[i + 1].clone());
                i += 1;
            } else {
                return Err(String::from(format!("{}", args[i])));
            }
        }

        Ok(config)
    }

    fn usage(pname: &String) {
        println!(
            "you should not run this manually, the server is supposed to be started by the client"
        );
    }
}
