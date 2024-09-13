mod mqtt;
use std::{
    process,
    time::Duration,
    thread::sleep,
    env
};

use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::Read;

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

fn get_current_time(tz: bool) -> DateTime<FixedOffset> {
    if tz {
        Local::now().with_timezone(&Local::now().offset().fix())
    } else {
        Utc::now().with_timezone(&Utc.fix())
    }
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

    let ct = get_current_time(config.general.tz);
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
        publisher(&config,client_id,topic)
    }else if mode=="subscriber"{
        println!("run subscriber");
        subscriber(&config,client_id,topic)
    }else{
        eprintln!("mode not found");
        process::exit(1);
    }
    
}


fn publisher(config: &Config, client_id: String, topic:String){
    let mut cli = mqtt::connect::connecting(&client_id, &config);
    let mut counter = 0;
    let mut failed: i8 = 0;

    loop {
        counter += 1;
        let ct = get_current_time(config.general.tz);
        let timestamp = ct.to_rfc3339();

        if !cli.is_connected() {
            failed += 1;
            eprintln!("Client disconnected. Attempting to reconnect... Retry [{}]", failed);

            if failed > config.broker.retries {
                eprintln!("Exceeded maximum retries. Exiting...");
                process::exit(1);
            }

            let unix = (ct.timestamp_micros()).to_string();
            let client_id = format!("{}/{}/{}", config.broker.topic, config.general.devid, unix);
            cli = mqtt::connect::connecting(&client_id, &config);
        }

        let sensors = vec![
            Sensor {
                sensor_id: 1,
                temp: 15.1 + counter as f32,
                rh: 25.5 + counter as f32,
            },
            Sensor {
                sensor_id: 2,
                temp: 25.1 + counter as f32,
                rh: 55.5 + counter as f32,
            },
        ];

        let payload = Data {
            dev_id: config.general.devid.to_string(),
            ts: timestamp,
            data: sensors,
        };

        let content = serde_json::to_string_pretty(&payload).unwrap();
        mqtt::publish::publish(&cli, &topic, &content, &config);

        sleep(config.general.periodic);
    }
}


fn subscriber(config: &Config,mut client_id: String, topic:String){
    let mut cli = mqtt::connect::connecting(&client_id, &config);
    let mut failed: i8 = 0;

    let mut ct = get_current_time(config.general.tz);

    let rx = cli.start_consuming();
    mqtt::subscribe::subscribe(&cli,&topic,&config);
    println!("Ready to Receive Message . . .");

    for msg in rx.iter() {
        if let Some(msg) = msg {
            ct = get_current_time(config.general.tz);
            let timestamp = ct.to_rfc3339();

            println!("Received at [{}] topic {} with msg {}",timestamp,msg.topic(), msg.payload_str());
        }else if !cli.is_connected() {
            failed += 1;
            eprintln!("Client disconnected. Attempting to reconnect... Retry [{}]", failed);

            if failed > config.broker.retries {
                eprintln!("Exceeded maximum retries. Exiting...");
                process::exit(1);
            }

            let unix = (ct.timestamp_micros()).to_string();
            client_id = format!("{}/{}", config.broker.topic, unix);
            cli = mqtt::connect::connecting(&client_id, &config);
            
            println!("Resubscribe topics...");
            mqtt::subscribe::subscribe(&cli,&topic,&config);
        }
    }

    if cli.is_connected() {
        println!("Disconnecting");
        cli.unsubscribe(&topic).unwrap();
        cli.disconnect(None).unwrap();
    }
    println!("Exiting");
}