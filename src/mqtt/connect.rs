use paho_mqtt as mqtt;
use std::process;
use std::time::Duration;
use std::thread::sleep;
use crate::Config;

pub fn connecting(client_id: &str, config: &Config) -> (mqtt::Client, mqtt::Receiver<Option<mqtt::Message>>) {
    let mut attempts = 1;
    loop {
        let create_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri(&config.broker.host)
            .client_id(client_id)
            .finalize();

        let cli = match mqtt::Client::new(create_opts) {
            Ok(client) => client,
            Err(err) => {
                eprintln!("Error creating the client: {:?}", err);
                sleep(config.broker.reconnect);
                attempts += 1;
                if attempts > config.broker.retries {
                    eprintln!("Exceeded maximum connection retries. Exiting...");
                    process::exit(1);
                }
                continue;
            }
        };

        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .user_name(&config.broker.user)
            .password(&config.broker.pass)
            .keep_alive_interval(Duration::from_secs(20))
            .clean_session(true)
            .finalize();

        if cli.connect(conn_opts).is_ok() {
            println!("Successfully connected after trying {} times", attempts);
            let rx = cli.start_consuming();
            return (cli, rx); // Return both cli and rx as a tuple
        } else {
            eprintln!("Unable to connect. Retrying... attempts {}", attempts);
            sleep(config.broker.reconnect);
            attempts += 1;
            if attempts > config.broker.retries {
                eprintln!("Exceeded maximum connection retries. Exiting...");
                process::exit(1);
            }
        }
    }
}

