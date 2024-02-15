pub(crate) use ids_generator::IdsGenerator;
use std::fmt::Write;

mod ids_generator;
mod time;

// Todo: Consider adding a trait to the String type to make this more idiomatic
pub fn write(query: &mut String, counter: &mut u8, field: &str) {
    if *counter > 1 {
        write!(query, ",").unwrap();
    }

    write!(query, " {} = ${}", field, counter).unwrap();
    *counter += 1;
}
