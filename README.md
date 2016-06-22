# fragile

Runs a command in a temporary NixOS container.

```
echo '{...}: { services.cron.enable = true; }' > config.nix
sudo fragile config.nix systemctl status
```
