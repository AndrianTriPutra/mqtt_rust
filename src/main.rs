
use std::{
    process,
    time::Duration,
    env,
    fs::File,
    io::Read
};

mod gettime;
mod mqtt;
mod publisher;
mod subscriber;

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    general: General,
    broker: Broker,
}

#[derive(Debug, Serialize, Deserialize)]
struct General {
    devid: String,
    tz: bool,
    #[serde(with = "humantime_serde")] 
    periodic: Duration,
}

#[derive(Debug, Serialize, Deserialize)]
struct Broker {
    host: String,
    user: String,
    pass: String,
    qos: i32,
    topic: String,
    #[serde(with = "humantime_serde")] 
    reconnect: Duration,
    retries: i8,
}

#[derive(Serialize, Deserialize, Debug)]
struct Sensor {
    sensor_id: u8,
    temp: f32,
    rh: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    dev_id: String,
    ts: String,
    data: Vec<Sensor>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let config: Config = serde_yaml::from_str(&contents)?;

        Ok(config)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        eprintln!("argument not found");
        process::exit(1);
    }
    let mode  = args[1..2].join(" ");
    let path = args[2..3].join(" ");
    println!("mode : [{}]", mode);
    println!("path : [{}]", path);

    let config = Config::load(&path).expect("Failed to read config file");

    let ct = gettime::current_time(config.general.tz);
    let unix = (ct.timestamp_micros()).to_string();
    let client_id = format!("{}/{}/{}", config.broker.topic, config.general.devid, unix);
    let topic = format!("{}/{}/data", config.broker.topic, config.general.devid);

    let timestamp = ct.to_rfc3339();
    println!("RUN at : [{}]", timestamp);
    println!("============= CONFIG =============");
    println!("DEVID : {}", config.general.devid);
    println!("============== MQTT ==============");
    println!("HOST  : {}", config.broker.host);
    println!("USER  : {}", config.broker.user);
    println!("PASS  : {}", config.broker.pass);
    println!("CLIENT: {}", client_id);
    println!("TOPIC : {}", topic);
    println!("============== MQTT ==============");

    if mode=="publisher"{
        println!("run publisher");
        publisher::publisher(&config,client_id,topic)
    }else if mode=="subscriber"{
        println!("run subscriber");
        subscriber::subscriber(&config,client_id,topic)
    }else{
        eprintln!("mode not found");
        process::exit(1);
    }
    
}
