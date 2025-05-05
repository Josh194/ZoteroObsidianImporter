#[derive(Debug, Clone, Copy)]
enum ByteOrder {
	Big,
	Little
}

fn get_byte_order(bytes: &[u8; 2]) -> Option<ByteOrder> {
	match u16::from_be_bytes(*bytes) {
		0xfeff => Some(ByteOrder::Big),
		0xfffe => Some(ByteOrder::Little),
		_ => None
	}
}

#[derive(Debug, Clone, Copy)]
pub enum ParseFailure {
	InvalidSize
}

// ! Check for utf8 BOM `EF BB BF` (cannot change since utf8 has no endianness)
pub fn parse_string(mut data: &[u8]) -> Result<String, ParseFailure> {
	// println!("UTF8");
	if (data.len() & 0x1) != 0 { return Ok(String::from_utf8_lossy(data).into_owned()); }
	if data.len() == 0 { return Ok("".to_owned()) }

	let Some(byte_order) = get_byte_order(&data[0..=1].try_into().unwrap()) else {
		// println!("UTF8");
		return Ok(String::from_utf8_lossy(data).into_owned());
	};

	data = &data[2..];

	// let _ = try_cast_slice(string.as_slice()).inspect_err(|e| { println!("{e}")}).unwrap_or(&[0, 0]);

	let mut buffer: Box<[u16]> = zerocopy::FromZeros::new_box_zeroed_with_elems(data.len() >> 1).unwrap();

	for (out, chunk) in buffer.iter_mut().zip(data.chunks_exact(2)) {
		let chunk: &[u8; 2] = chunk.try_into().unwrap();

		*out = match byte_order {
			ByteOrder::Big => u16::from_be_bytes(*chunk),
			ByteOrder::Little => u16::from_le_bytes(*chunk),
		}
	}

	// println!("UTF16");
	Ok(String::from_utf16_lossy(&buffer))
}