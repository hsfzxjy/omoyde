All about msg.bin, the file storing user created messages.

The file stores an array of struct `Msg`s with _Big-Endian_. Each `Msg` has a varied length, with the layout described as belows:

| field name | nbytes | type   |
| ---------- | ------ | ------ |
| type       | 1      | char   |
| dtBase     | 4      | u32    |
| dtOffset   | 1      | i8     |
| lText      | 2      | u16    |
| text       | lText  | string |

Detailed explanation of these fields:

- **type** The type of message. Usually it's a letter in lower case (e.g., `'m'`, `'q'`).
- **dtBase & dtOffset** These two fields together encode the date (& time) of message, by which we sort them in timeline. It is the number of _milliseconds_ that have elapsed since the Unix epoch (UTC), computed as `dtBase * 1000 + dtOffset`.
- **lText** Number of bytes that field **text** occupies.
- **text** The content of message. This field is encoded with the encoding specified in config field `system.msg_encoding`.
