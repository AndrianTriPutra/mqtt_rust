use crate::Config;
use crate::mqtt;
use crate::gettime;

use std::process;

pub fn subscriber(config: &Config,mut client_id: String, topic:String){
    let mut cli = mqtt::connect::connecting(&client_id, &config);
    let mut failed: i8 = 0;

    let mut ct = gettime::current_time(config.general.tz);

    let rx = cli.start_consuming();
    mqtt::subscribe::subscribe(&cli,&topic,&config);
    println!("Ready to Receive Message . . .");

    for msg in rx.iter() {
        if let Some(msg) = msg {
            ct = gettime::current_time(config.general.tz);
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