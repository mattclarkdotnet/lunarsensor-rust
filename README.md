# TODO
* make logs available at an endpoint
* add motivation and description to README

# Rust setup for cross compilation to armv6

Add the cross compilation target for Alpine on Pi Zero W (armv6 with hard float, musl instead of gnu libc), and build the 

``` sh
rustup target add arm-unknown-linux-musleabihf
brew tap FiloSottile/homebrew-musl-cross
brew install FiloSottile/musl-cross/musl-cross --without-x86_64 --with-arm-hf
CROSS_COMPILE=arm-linux-musleabihf- cargo build --release --target arm-unknown-linux-musleabihf
```

# Pi Zero setup with Alpine

## Creating the bootable Alpine image

```sh
SDCARD=/dev/disk4
VOL=/Volumes/BOOTPART
diskutil unmountDisk $SDCARD
diskutil partitionDisk $SDCARD GPT FAT32 BOOTPART 1G FAT32 FREE R
fdisk -e $SDCARD
 f 1
 w
 q
tar xf ~/Downloads/alpine-rpi-3.17.2-armhf.tar -C $VOL
echo "dtparam=i2c_arm=on" > $VOL/usercfg.txt
echo "Optional to enable UART"
echo "modules=loop,squashfs,sd-mod,usb-storage console=serial0,115200" > $VOL/cmdline.txt
echo "enable_uart=1" >> $VOL/usercfg.txt 
cp target/arm-unknown-linux-musleabihf/release/lunarsensor-rust $VOL/lunarsensor
diskutil unmountDisk $SDCARD
```

## Configuring Alpine and the app

Once booted the root user has no password

After logging in as root run ```setup-alpine``` using either a USB keyboard and HDMI monitor, or over the serial console.  Set the hostname to "lunarsensor" and lunar.fyi will find it automatically.

Once that is done, run the following commands, and also copy the binary to /opt/lunarsensor/lunarsensor.

The bind address of "::" will bind to both ipv6 and ipv4 on all interfaces, it's like "0.0.0.0" but for both protocols.  Binding to privileged port 80 without running as root is enabled by the ```capabilities="^cap_net_bind_service"``` line in the init script.

```sh
setup-alpine
apk update
apk upgrade
apk add dbus
apk add avahi
rc-update add dbus default
rc-update add avahi-daemon default
echo 'i2c-dev' > /etc/modules-load.d/i2c.conf
modprobe i2c-dev
adduser -Sh /opt/lunarsensor lunarsensor
addgroup i2c
addgroup lunarsensor i2c

mkdir -p /opt/lunarsensor
chown lunarsensor /opt/lunarsensor/
lbu include /opt/lunarsensor

mkdir -p /var/log/lunarsensor
chown lunarsensor /var/log/lunarsensor/
lbu include /var/log/lunarsensor

cat > /opt/lunarsensor/Rocket.toml << EOF
[default]
address = "::"
port = 80
EOF

cat > /etc/logrotate.d/lunarsensor << EOF
/var/log/lunarsensor/lunarsensor {
    missingok
    notifempty
}


cat > /etc/init.d/lunarsensor << EOF
#!/sbin/openrc-run
supervisor=supervise-daemon
name=\${RC_SVCNAME}
capabilities="^cap_net_bind_service"
command="/opt/lunarsensor/lunarsensor"
command_background=true
directory="/opt/lunarsensor"
command_user="lunarsensor"
output_log="/var/log/lunarsensor/lunarsensor"
error_log="/var/log/lunarsensor/lunarsensor"

start_pre() {
    chgrp i2c /dev/i2c-1
}
EOF

chmod a+x /etc/init.d/lunarsensor
lbu include /etc/init.d/lunarsensor

rc-update add lunarsensor default

lbu commit
```


