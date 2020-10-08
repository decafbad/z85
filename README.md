Rust library of Z85, ZEROMQ's binary-to-text encoding mechanism. 
https://rfc.zeromq.org/spec:32/Z85/

Starting from 3.0 version, this library adds padding support, which makes it *not* fully compatible with ZeroMQ's RFC.

Here is how padding works:
85^5 is bigger than 2^32, therefore a five-byte Z85 data chunk cannot start with '#'. Count of this char sets how many bytes are missing from the tail chunk.