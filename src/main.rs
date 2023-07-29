mod error;
mod mqtt;
mod report;
use error::MyError;
use report::Measurements;

use rumqttc::{AsyncClient, MqttOptions, Transport};
use serial::SerialPort;
use std::{env, io::Read, time::Duration};
use tokio::{select, task::JoinHandle};

struct Config {
    pub mqtt_host: String,
    pub mqtt_topic_prefix: String,
    pub mqtt_qos: i32,
    pub serial_port: String,
    pub credentials: Option<(String, String)>,
}

impl Config {
    fn from_env() -> Self {
        let defaults = Self::default();
        Self {
            mqtt_host: env::var("MQTT_HOST").unwrap_or(defaults.mqtt_host),
            mqtt_topic_prefix: env::var("MQTT_TOPIC").unwrap_or(defaults.mqtt_topic_prefix),
            mqtt_qos: env::var("MQTT_QOS")
                .and_then(|v| v.parse().map_err(|_| env::VarError::NotPresent))
                .unwrap_or(defaults.mqtt_qos),
            serial_port: env::var("SERIAL_PORT").unwrap_or(defaults.serial_port),
            credentials: env::var("MQTT_USERNAME")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .zip(
                    env::var("MQTT_PASSWORD")
                        .ok()
                        .filter(|s| !s.trim().is_empty()),
                ),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mqtt_host: "tcp://10.10.10.13:1883".to_owned(),
            mqtt_topic_prefix: "dsmr".to_owned(),
            mqtt_qos: 0,
            serial_port: "/dev/ttyUSB1".to_owned(),
            credentials: None,
        }
    }
}

#[tokio::main]
async fn main() -> ! {
    let cfg = Config::from_env();

    let mut mqttoptions = MqttOptions::new("dsmr-reader", &cfg.mqtt_host, 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(30));
    mqttoptions.set_transport(Transport::Tcp);
    if let Some((user, pass)) = &cfg.credentials {
        mqttoptions.set_credentials(user, pass);
    }

    loop {
        let (mut client, mut eventloop) = AsyncClient::new(mqttoptions.clone(), 12);

        let eventloop: JoinHandle<_> = tokio::spawn(async move {
            loop {
                if let Err(e) = eventloop.poll().await {
                    eprintln!("Eventloop error: {}", e);
                }
            }
        });

        select! {
            handle = eventloop => {
                eprintln!("Eventloop stopped: {}", handle.unwrap_err());
            }
            run = run(&cfg, &mut client) => {
                eprintln!("Encountered error running: {}", run.unwrap_err());
            }
        }

        // Cleanup before reseting
        if let Err(e) = client.disconnect().await {
            eprintln!("Error disconnecting: {}", e);
        }

        // Wait a bit before retrying.
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn run(cfg: &Config, mut client: &mut AsyncClient) -> Result<(), MyError> {
    // Open Serial
    let mut port = serial::open(&cfg.serial_port)?;
    let settings = serial::PortSettings {
        baud_rate: serial::BaudRate::Baud115200,
        char_size: serial::CharSize::Bits8,
        parity: serial::Parity::ParityNone,
        stop_bits: serial::StopBits::Stop1,
        flow_control: serial::FlowControl::FlowNone,
    };
    port.configure(&settings)?;
    port.set_timeout(Duration::from_secs(10))?;
    let reader = dsmr5::Reader::new(port.bytes().take_while(Result::is_ok).map(Result::unwrap));

    for readout in reader {
        let telegram = readout.to_telegram().map_err(MyError::Dsmr5Error)?;
        let measurements: Measurements = telegram.objects().filter_map(Result::ok).collect();

        let messages = measurements.into_mqtt_messages(cfg.mqtt_topic_prefix.clone());
        for msg in messages {
            msg.send(&mut client).await?;
        }
    }

    // Reader should never be exhausted
    Err(MyError::EndOfReader())
}
