use std::io::{Cursor, Error as IoError, Write};

use byteorder::{BigEndian, WriteBytesExt};
use chrono::{DateTime, NaiveDateTime, Timelike, Utc};

use lazy_static::lazy_static;
use rand::Rng;
use uuid::Uuid;

use crate::Identifier;

lazy_static! {
	/// The maximum possible datetime.
	pub static ref MAX_DATETIME: DateTime<Utc> =
		DateTime::from_utc(NaiveDateTime::from_timestamp(i64::from(i32::MAX), 0), Utc)
			.with_nanosecond(1_999_999_999u32)
			.unwrap();
}

pub enum Component<'a> {
	Uuid(Uuid),
	FixedLengthString(&'a str),
	Identifier(&'a Identifier),
	DateTime(DateTime<Utc>),
}

impl<'a> Component<'a> {
	// Really just implemented to not set off a clippy warning
	pub fn is_empty(&self) -> bool {
		false
	}

	pub fn len(&self) -> usize {
		match *self {
			Component::Uuid(_) => 16,
			Component::FixedLengthString(s) => s.len(),
			Component::Identifier(t) => t.0.len() + 1,
			Component::DateTime(_) => 8,
		}
	}

	pub fn write(&self, cursor: &mut Cursor<Vec<u8>>) -> Result<(), IoError> {
		match *self {
			Component::Uuid(uuid) => cursor.write_all(uuid.as_bytes()),
			Component::FixedLengthString(s) => cursor.write_all(s.as_bytes()),
			Component::Identifier(i) => {
				cursor.write_all(&[i.0.len() as u8])?;
				cursor.write_all(i.0.as_bytes())
			}
			Component::DateTime(datetime) => {
				let time_to_end = nanos_since_epoch(&MAX_DATETIME) - nanos_since_epoch(&datetime);
				cursor.write_u64::<BigEndian>(time_to_end)
			}
		}
	}
}

/// Gets the number of nanoseconds since unix epoch for a given datetime.
///
/// # Arguments
/// * `datetime`: The datetime to convert.
fn nanos_since_epoch(datetime: &DateTime<Utc>) -> u64 {
	let timestamp = datetime.timestamp() as u64;
	let nanoseconds = u64::from(datetime.timestamp_subsec_nanos());
	timestamp * 1_000_000_000 + nanoseconds
}

// Serializes component(s) into bytes.
///
/// # Arguments
/// * `components`: The components to serialize to bytes.
pub fn components_into_bytes(components: &[Component]) -> Vec<u8> {
	let len = components.iter().fold(0, |len, component| len + component.len());
	let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::with_capacity(len));

	for component in components {
		if let Err(err) = component.write(&mut cursor) {
			panic!("Could not write bytes: {}", err);
		}
	}

	cursor.into_inner()
}

pub fn generate_random_i32() -> i32 {
	let mut rng = rand::thread_rng();
	rng.gen::<i32>()
}