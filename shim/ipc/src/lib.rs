use std::process::ExitStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Msg {
	pub id: u32,
	pub code: i32,
	pub success: bool
}

impl Msg {
	pub fn new(id: u32, status: &ExitStatus) -> Self {
		Self { id, code: status.code().unwrap_or_default(), success: status.success() }
	}

	pub fn serialize(self) -> Box<[u8]> {
		[
			&self.id.to_ne_bytes() as &[u8],
			&self.code.to_ne_bytes() as &[u8],
			&[self.success.into()] as &[u8]
		].concat().into_boxed_slice()
	}

	pub fn deserialize(data: &[u8]) -> Result<Self, MsgDeserializeError> {
		let array: &[u8; 9] = data.try_into().map_err(|_| MsgDeserializeError::InvalidSize)?;

		Ok(Self {
			id: u32::from_ne_bytes(array[0..4].try_into().unwrap()),
			code: i32::from_ne_bytes(array[4..8].try_into().unwrap()),
			success: array[8] != 0
		})
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgDeserializeError {
	InvalidSize
}