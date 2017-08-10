# Find Linux MD Raid Metadata on a Disk

I used this to find the partition boundaries on a disk with a corrupted partition table.
Based on the offset you should be able to calculate the partition boundaries.

The location of the metadata varies depending on the metadata version. Take a look at
--metadata section of mdadm's manpage. A more detailed description of the metadata layout
can be found in the [official wiki](https://raid.wiki.kernel.org/index.php/RAID_superblock_formats).

## Usage

`find_raid` takes one argument, the path to the device to scan. It outputs the offsets of
the metadata, the major metadata version, array name, and creation and update timestamps.

```
$ find_raid /dev/sda
hit at byte 4096 (version: 1.x, name: "pg:5", creation time: 2017-08-07T17:45:03+02:00, update time: 2017-08-07T18:12:15+02:00)
hit at byte 31391744 (version: 0.90.0, name: unknown, creation time: 2017-08-06T05:58:33+02:00, update time: 2017-08-07T22:27:18+02:00)
```

## Caveats

* You'll likely get false positives. The magic number is assumed to be aligned at
  512-bit boundaries which help to reduce the number of false positive but you'll still
  encounter them.
* In case of version 1.x metadata the minor version number isn't available. This because the minor version isn't stored
  in the metadata but rather depends on the location of the metadata.
* Metadata of version 0.x uses native-endian while 1.x metadata uses little-endian. By default big-endian support is only
  enabled on big-endian platforms. Use the `big_endian` feature gate to force enable it (`cargo build --features big_endian`).
