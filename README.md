# PLight

A configurable program for organizing dynamic backlighting.

## Setup

### Step 1: Clone repo and compile

1. Clone this repository with:
```bash
git clone https://github.com/pguin-sudo/plight /src/plight
cd /src/plight
```

2. Build using cargo:
```bash
cargo build --release
```

3. Add semantic link to bin:
```bash
sudo ln -s /src/plight/target/release/plight /bin/plight
```

1. Set up config in `/etc/plight/config.toml`:
```bash
sudo mkdir /etc/plight && sudo touch /etc/plight/config.toml && sudo vim /etc/plight/config.toml
```

### Step 2: Setup arduino

Setup arduino with arduino-cli

1. Move to the arduino directory:
```bash
cd /src/plight/arduino
```

2. Configure your setup in `/src/plight/arduino/sketch.yaml`:

3. Edit permissions of your serial `/dev/ttyUSBX`:
```bash
sudo chmod 777 /dev/ttyUSBX 
```

4. Compile:
```bash
arduino-cli compile 
```

5. Upload:
```bash
arduino-cli upload 
```

### Step 3: Create a systemd Service File
           
Create a systemd service file to manage the daemon:

1. Create a new file called `plight.service` in `/etc/systemd/system/`:

```bash
sudo vim /etc/systemd/system/plight.service
```

2. Add the following content to the file:

```ini
[Unit]
Description=PLight - dynamic backlighting

[Service]
ExecStart=/bin/plight
Restart=always
User=root

[Install]
WantedBy=multi-user.target
```

### Step 4: Start and Enable the Daemon

Run the following commands to start and enable the daemon:

```bash
sudo systemctl start plight.service
sudo systemctl enable plight.service
```

### Step 4: Check the Logs

You can check the logs of your daemon using:

```bash
sudo journalctl -u plight.service -f
```
