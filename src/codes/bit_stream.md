Bit Streams
===========

This trait implements the basic operations for reading and writing bit streams. Complex
codes such as Elias ɣ are then implemented by traits such as 

The two basic operation available are reading or writing a unary code (that is, a sequence of 
zeros followed by a one), and reading or writing a block of bits of fixed size.

This trait has a [`BitOrder`] parameter that makes the stream behave in a little-endian or
big-endian fashion. In a little-endian bit stream, bit $k$ of the stream is bit $k \bmod 8$