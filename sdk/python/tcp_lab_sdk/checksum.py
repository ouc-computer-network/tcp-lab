"""Checksum helpers shared across assignments."""

def internet_checksum(data: bytes) -> int:
    """Return the 16-bit ones' complement checksum."""
    if len(data) % 2 == 1:
        data += b"\x00"

    total = 0
    for i in range(0, len(data), 2):
        word = (data[i] << 8) + data[i + 1]
        total = (total + word) & 0xFFFF_FFFF

    while total >> 16:
        total = (total & 0xFFFF) + (total >> 16)

    return (~total) & 0xFFFF
