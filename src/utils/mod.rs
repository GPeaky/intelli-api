use core::fmt::Write;

pub(crate) use ids_generator::IdsGenerator;
pub(crate) use password_hash::*;
pub(crate) use ports::MachinePorts;

mod bitset;
mod ids_generator;
mod password_hash;
mod ports;

// Todo: Consider adding a trait to the String type to make this more idiomatic
pub fn write(query: &mut String, counter: &mut u8, field: &str) {
    if *counter > 1 {
        write!(query, ",").unwrap();
    }

    write!(query, " {} = ${}", field, counter).unwrap();
    *counter += 1;
}
