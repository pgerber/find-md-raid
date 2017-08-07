# Find Linux MD Raid Magic Number on a Disk

I used this to find the partition boundaries on a disk with a corrupted partition table.
Based on the offset you should be able to calculate the partition boundaries.

The location of the metadata varies depending on the metadata version. Take a look at
--metadata section of mdadm's manpage.

## Usage

`find_raid` takes one argument, the path to the device to scan. It outputs the offsets of
the found magic numbers.

```
    $ find_raid /dev/sda
    hit at byte 1052672
    hit at byte 7999590400
    hit at byte 11551875072
    hit at byte 24862347264
    hit at byte 26261688320
    hit at byte 29180690432

```

## Ceveats

* You'll likely get false positives. The magic number is assumed to be aligned at
  512-bit boundaries which help to reduce the number of false positive but you'll still
  encounter them.
* The code assumes the magic number is 0xa92b4efc represented in little-endian. I've
  not verified that this is always the case.
