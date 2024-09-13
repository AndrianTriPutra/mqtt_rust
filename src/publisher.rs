use crate::{
    Config,
    Sensor,
    Data,
    mqtt,
    gettime
};

use std::{
    process,
    thread::sleep,
};

pub fn publisher(config: &Config, client_id: String, topic:String){
    let (mut cli, _) = mqtt::connect::connecting(&client_id, &config);
    let mut counter = 0;
    let mut failed: i8 = 0;

    loop {
        counter += 1;
        let ct = gettime::current_time(config.general.tz);
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
            (cli, _)  = mqtt::connect::connecting(&client_id, &config);
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

