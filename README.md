# IMPORTANT NOTICE
Development on this tool has ceased due to Roblox updates (namely, ratelimiting on key APIs.) Sorry for any inconvenience!

# roblox_inventory_scanner

roblox_inventory_scanner is a tool in rust that allows you to scan any Roblox user's inventory for limited items.

## Installation

At this time, roblox_inventory_scanner can be built from source using ``cargo``. The only tested platforms are Ubuntu (under WSL) and Windows 10.
Prebuilt binaries for Windows 10 can be obtained from the releases.

## Usage

After building, you can open the executable and enter a Roblox user id. A few seconds later a list of all items they own (and their total RAP/Value) will be output.
Currently, a scan on a terminated user will NOT return all their items. This is due to Roblox changing the ratelimit on the API I use. I'll fix it soon.


## Contributing
Contributions are welcome! Just open a pull request with any changes.
