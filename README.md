Renode Embassy Test
===================

[Embassy](https://github.com/embassy-rs/embassy) based example Project for the [nRF52840-Dongle](https://www.nordicsemi.com/Products/Development-hardware/nrf52840-dongle) emulated with [Renode](https://renode.io/).

# Development environment

Install [Nix](https://nixos.org/download.html) with [flake support](https://nixos.wiki/wiki/Flakes#Non-NixOS).

```
$ nix-develop
```

# Firmware

```
$ cd firmware
$ cargo build
```

# Renode

```
$ cd renode
$ renode nrf52840.resc
```

## Test

```
$ renode-test test-led.robot
```

