use paho_mqtt as mqtt;
use std::thread::sleep;
use crate::Config;

pub fn publish(cli: &mqtt::Client, topic: &str, content: &str, config: &Config) {
    let msg = mqtt::Message::new(topic.to_string(), content.to_string(), config.broker.qos);
    println!("publish to {} with msg {} ", msg.topic(), msg.payload_str());

    if let Err(e) = cli.publish(msg) {
        eprintln!("Error sending message: {:?}", e);
        // Handle the publish error by waiting before retrying
        sleep(config.broker.reconnect);
    }
}
