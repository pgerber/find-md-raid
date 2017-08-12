# Test Data

This directory contains test data used by the tests defined in [.travis.yml](../.travis.yml). The tests
simply execute `find_raid` on the the images in `*.meta.xz` and compare the output with the expected
output stored in `*.expected`/`*.expected-little`/`*.expected-big`.

## Structure

Compressed disk images:

* **${version}-big.meta.xz** → image with metadata of **version** using **big**-endian representation ¹
* **${version}-little.meta.xz** → image with metadata of **version** using **little**-endian representation

Expected Output on Stdout:

* **${version}-${endian}.meta.expected** → expected output
* **${version}-${endian}.meta.expected-big** → expected output when **big**-endian support is enabled ²
* **${version}-${endian}.meta.expected-little** → expected output when only **little**-endian support is enabled ²

¹ V1.x metadata is always little-endian.  
² Only exists if output differs with big-endian support enabled. Otherwise, `*.expected` is used.
