use std::io::{Read, Write};
use header::Header;

pub trait MessageInterface
{
	type Message;
	fn read<R: Read>(header: Header, reader: &mut R) -> Result<Self::Message, ()>;
	fn send<W: Write>(&self, writer: &mut W) -> Result<usize, ()>;
	fn fill_header(&self, header: &mut Header);
}