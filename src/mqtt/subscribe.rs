
use paho_mqtt as mqtt;
use std::thread::sleep;
use crate::Config;

pub fn subscribe(cli: &mqtt::Client,topic: &str,config: &Config) {
    if let Err(e) = cli.subscribe(topic, config.broker.qos) {
        eprintln!("Error subscribes topics: {:?}", e);
        sleep(config.broker.reconnect);
    }
}
