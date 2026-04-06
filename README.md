# A Home Assistant shell bridge

This simple bridge allows dockered home assistant to execute commands on the
host system.

## Instalation

### 1. Install this script

```sh
cargo install --git https://github.com/mendess/hass-bridge.git
```

### 2. Create a service file
```systemd
[Unit]
Description=Shell bridge for home assistant
After=network.target

[Service]
ExecStart=/bin/bash -l -c 'exec ${CARGO_HOME:-$HOME/.cargo}/bin/hass-bridge'
Restart=always
RestartSec=3

[Install]
WantedBy=default.target
```
to `/usr/lib/systemd/user/hass-bridge.service`

### 3. Enable the service

```sh
systemctl --user daemon-reload
systemctl --user enable hass-bridge
```

### 4. Add to home assistant

Add to configuration.yaml

```yaml
shell_command:
  host_run: ./host-run.sh {{ command }}
```

Add the [host-run](./host-run.sh) helper script

```sh
curl -s https://raw.githubusercontent.com/mendess/hass-bridge/refs/heads/master/host-run.sh | docker exec homeassistant tee host-run.sh
docker exec -i homeassistant chmod -v 777 host-run.sh
```

Restart home assistant.


## Usage

This can now be added to automations and scripts like so
```yaml
- action: shell_command.host_run
  metadata: {}
  data:
    command: my-nice-host-command
  alias: runs the my nice host command
```
