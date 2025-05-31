#!/bin/sh
cargo build --release
sudo cp -v target/release/client /usr/local/bin/scheduler-client
sudo cp -v target/release/server /usr/local/bin/scheduler-server

sudo mkdir -p /etc/scheduler
sudo cp -v config.json /etc/scheduler/config.json
sudo chown -R user:user /etc/scheduler/

sudo cp -v scheduler.service /etc/systemd/system/
sudo systemctl daemon-reload

sudo mkdir /var/log/scheduler
sudo chown user:user /var/log/scheduler
