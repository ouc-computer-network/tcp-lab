use pyo3::prelude::*;
use pyo3::types::PyBytes;
use tcp_lab_core::{Packet, TcpHeader};

/// Convert a Rust Packet to a Python `tcp_lab.structs.Packet` object.
pub fn to_py_packet<'py>(py: Python<'py>, packet: Packet) -> PyResult<Bound<'py, PyAny>> {
    let tcp_lab_mod = py.import("tcp_lab")?;
    let structs_mod = tcp_lab_mod.getattr("structs")?;
    let header_cls = structs_mod.getattr("TcpHeader")?;
    let packet_cls = structs_mod.getattr("Packet")?;

    let h = packet.header;
    let py_header = header_cls.call(
        (
            h.seq_num,
            h.ack_num,
            h.flags,
            h.window_size,
            h.checksum,
            h.urgent_ptr,
        ),
        None,
    )?;

    let py_payload = PyBytes::new(py, &packet.payload);

    packet_cls.call1((py_header, py_payload))
}

/// Convert a Python `tcp_lab.structs.Packet` object to a Rust Packet.
pub fn from_py_packet(obj: &Bound<'_, PyAny>) -> PyResult<Packet> {
    let header_obj = obj.getattr("header")?;
    let payload_obj = obj.getattr("payload")?;

    let seq_num: u32 = header_obj.getattr("seq_num")?.extract()?;
    let ack_num: u32 = header_obj.getattr("ack_num")?.extract()?;
    let flags: u8 = header_obj.getattr("flags")?.extract()?;
    let window_size: u16 = header_obj.getattr("window_size")?.extract()?;
    let checksum: u16 = header_obj.getattr("checksum")?.extract()?;
    let urgent_ptr: u16 = header_obj.getattr("urgent_ptr")?.extract()?;

    let payload: Vec<u8> = payload_obj.extract()?;

    let header = TcpHeader {
        src_port: 0, // Not used in this lab model usually
        dst_port: 0,
        seq_num,
        ack_num,
        flags,
        window_size,
        checksum,
        urgent_ptr,
    };

    Ok(Packet { header, payload })
}
