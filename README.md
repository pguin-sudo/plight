# PLight

A configurable program for organizing dynamic backlighting.

## Setup

### Step 1: Clone repo and compile

1. Clone this repository with:
```bash
git clone https://github.com/pguin-sudo/plight /usr/src/plight
cd /usr/src/plight
```

2. Build using cargo:
```bash
cargo build --release
```

3. Add semantic link to bin:
```bash
sudo ln -s /usr/src/plight/target/release/plight /usr/bin/plight
```

### Step 2: Setup arduino

Setup arduino with arduino-cli

1. Move to the arduino directory:
```bash
cd /usr/src/plight/arduino
```

2. Configure your setup in `/src/plight/arduino/sketch.yaml`:

3. Edit permissions of your serial `/dev/ttyUSBX`:
```bash
sudo chmod 777 /dev/ttyUSBX 
```

4. Compile:
```bashr
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
sudo vim /etc/systemd/user/plight.service
```

2. Add the following content to the file:

```ini
[Unit]
Description=PLight - dynamic backlighting

[Service]
ExecStart=/bin/plight
Restart=always

[Install]
WantedBy=default.target
```

3. Change mode of USB device:

```bash
sudo vim /etc/udev/rules.d/99-usb-tty.rules
```

```rules
KERNEL=="ttyUSBX", MODE="0777"
```

### Step 4: Start and Enable the Daemon

Run the following commands to start and enable the daemon:

```bash
sudo systemctl --user start plight.service
sudo systemctl --user enable plight.service
```

### Step 4: Check the Logs

You can check the logs of your daemon using:

```bash
sudo journalctl -u plight.service -f
```
