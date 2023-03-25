# Sleep-On-LAN

The idea of this project is very similar to [Wake-On-LAN](https://en.wikipedia.org/wiki/Wake-on-LAN),
but instead of "waking" on PC/laptop it allows you to suspend/hibernate/poweroff (depending on configuration)
your machine on "magic package" receival.

## Implementation details

This projects provides a UDP service, which puts machine into sleep/suspend on receival of "magic package".
The structure of "magic package" is identical to "Wake-On-LAN" magic package,
but package is expected to contain reversed machine MAC, i.e.:
* Wake-On-LAN package - `6 * 0xFF + 16 * MAC address = 102 bytes total`
* Sleep-On-LAN package - `6 * 0xFF + 16 * reverse(MAC address) = 102 bytes total`

Such approach allows you to use existing WOL clients (cli, mobile, etc) for interaction with this service.

Example for [wol](https://sourceforge.net/projects/wake-on-lan/):
```bash
wol --port 9 11:22:33:44:55:66  # wakes your PC
wol --port 9 66:55:44:33:22:11  # put your PC to sleep
```

## Installation

### Arch Linux
* Download release package
* Extract the content: `tar -xvzf sleep-on-lan.tar.gz`
* Build and install: `makepkg -si`
* (Optional) Enable bundled systemd service to autostart Sleep-On-LAN service on boot: `sudo systemctl enable --now sleep-on-lan.service`

## Configuration

Project configuration file is located at `/etc/sleep-on-lan.conf`. This is INI config file with next structure:
```ini
[main]
interface = enp3s0                  # interface, whose MAC address should be used
port = 9                            # port to listen for magic package
sleep-cmd = systemctl hybrid-sleep  # sleep command to execute
```

## Similar projects
* https://github.com/SR-G/sleep-on-lan
* https://github.com/ahirata/sleep-on-lan
