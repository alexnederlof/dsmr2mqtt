# DSMR to MQTT

Reads out a dutch smart meter using the DSMRv5 protocol
and publishes some of the stats out to an mqtt broker.

## DSMRv5 Parser

For parsing the data telegrams I use a modified version of the `dsmr5` crate,
you can view its source [here](https://github.com/NULLx76/dsmr5)

# Config

The following variables can be configured

| name        | meaning                                      | default        |
| ----------- | -------------------------------------------- | -------------- |
| MQTT_HOST   | The address of your MQTT server (IP or FQDN) | `::1`          |
| MQTT_TOPIC  | The topic prefix                             | `dsmr`         |
| MQTT_QOS    | The MQTT QOS                                 | `0`            |
| SERIAL_PORT | The port to listen to                        | `/dev/ttyUSB1` |

# Docker builds

To build for all platforms, use

```
docker buildx build --platform linux/amd64,linux/arm64,linux/arm/v7 \
    --push \
    -t alexnederlof/dsmr2mqtt .
```
