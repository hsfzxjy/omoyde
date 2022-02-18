All about metas.bin, the file storing metadata of photos.

This file stores an array of struct `PhotoMeta`s with _Big-Endian_. Each `PhotoMeta` has a fixed length of 9 bytes, with the layout described as belows:

| field name | nbytes | type |
| ---------- | ------ | ---- |
| pid        | 3      | u24  |
| datetime   | 4      | u32  |
| h          | 1      | u8   |
| w          | 1      | u8   |

Detailed explanation of these fields:

- **pid** The unique ID for each photo.
- **datetime** The date (& time) of photo, by which we sort them in timeline. It is the number of _seconds_ that have elapsed since the Unix epoch (UTC).
- **h & w** The height and width of photo. Note that they have been reduced in order to fit within a single byte, with the aspect ratio kept as close as possible.
