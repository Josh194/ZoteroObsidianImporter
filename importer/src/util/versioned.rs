use std::marker::PhantomData;

use serde::{de, Deserialize};
use serde_path_to_error::Track;

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VersionedFile<'de_, T: de::Deserializer<'de_>> {
	pub version: i64,
	pub data: T,
	#[serde(skip)] _phantom: PhantomData<&'de_ ()>
}

#[derive(Debug, Clone, Copy)]
pub enum Error<E: de::Error> {
	InvalidVersion(i64),
	Inner(E)
}

impl<E: de::Error> From<E> for Error<E> {
	fn from(value: E) -> Self { Self::Inner(value) }
}

pub fn deserialize<
	'de,
	D: de::Deserializer<'de>,
	Target: Deserialize<'de>,
	Value: Deserialize<'de> + de::Deserializer<'de, Error = D::Error>
> (version: i64, deserializer: D) -> Result<Target, Error<D::Error>> {
	let file = VersionedFile::<'de, Value>::deserialize(deserializer)?;
	
	if file.version != version { return Err(Error::InvalidVersion(file.version)); }

	Ok(Target::deserialize(file.data)?)
}

pub fn deserialize_json_str_track<'de, T: Deserialize<'de>>(version: i64, data: &'de str) -> Result<T, Error<serde_json::Error>> {
	deserialize::<_, T, serde_json::Value>(version, serde_path_to_error::Deserializer::new(&mut serde_json::Deserializer::from_str(data), &mut Track::new()))
}