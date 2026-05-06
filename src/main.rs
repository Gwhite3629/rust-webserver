
use rustwebserver::{
    ThreadPool,
    Server,
    GlobalConfig,
    CONFIG,
};

fn main() {

    let args: Vec<String> = std::env::args().collect::<Vec<String>>();
    if args.len() == 1 {
        println!("usage: {} <config>", args[0]);
        return;
    }

    match CONFIG.set(GlobalConfig::new(args)) {
        Ok(_) => (),
        Err(_) => panic!("Failed to setup config"),
    }

    //println!("{CONFIG:#?}");

    let names: Vec<&String> = CONFIG.get().unwrap().servers.keys().collect();

    let pool = ThreadPool::new(names.len());

    for name in names {
        pool.execute(move || {
            let mut server = Server::new(name.clone());
            server.start();
        });
    }

}
