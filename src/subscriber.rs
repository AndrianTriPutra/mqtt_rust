use std::process;
use paho_mqtt as paho;
use crate::{ 
    Config,
    mqtt,
    gettime
};

pub fn subscriber(config: &Config, mut client_id: String, topic: String) {
    let mut failed: i8 = 0;

    let (mut cli, mut rx) = mqtt::connect::connecting(&client_id, &config);
    mqtt::subscribe::subscribe(&cli, &topic, &config);
    println!("Ready to Receive Message . . .");

    // Call handler function to process messages and handle disconnections
    handler(&mut cli, &mut rx, &mut client_id, &topic, &mut failed, config);

    if cli.is_connected() {
        println!("Disconnecting");
        cli.unsubscribe(&topic).unwrap();
        cli.disconnect(None).unwrap();
    }
    println!("Exiting");
}

// New handler function to process messages and handle disconnections
fn handler(
    cli: &mut paho::Client,
    rx: &mut paho::Receiver<Option<paho::Message>>,
    client_id: &mut String,
    topic: &str,
    failed: &mut i8,
    config: &Config,
) {
    loop {
        match rx.recv() {
            Ok(Some(msg)) => {
                let ct = gettime::current_time(config.general.tz);
                let timestamp = ct.to_rfc3339();
                println!(
                    "Received at [{}] topic {} with msg {}",
                    timestamp,
                    msg.topic(),
                    msg.payload_str()
                );
            }
            Ok(None) | Err(_) => {
                if !cli.is_connected() {
                    *failed += 1;
                    eprintln!(
                        "Client disconnected. Attempting to reconnect... Retry [{}]",
                        *failed
                    );

                    if *failed > config.broker.retries {
                        eprintln!("Exceeded maximum retries. Exiting...");
                        process::exit(1);
                    }

                    let ct = gettime::current_time(config.general.tz);
                    let unix = ct.timestamp_micros().to_string();
                    *client_id = format!("{}/{}", config.broker.topic, unix);

                    // Reconnect and reassign `cli` and `rx`
                    let (new_cli, new_rx) = mqtt::connect::connecting(client_id, config);
                    *cli = new_cli;
                    *rx = new_rx;

                    println!("Resubscribe topics...");
                    mqtt::subscribe::subscribe(cli, topic, config);
                }
            }
        }
    }
}
