pub mod error;
pub mod packets;
pub mod ser;
mod types;

extern crate ultimaonline_net_macros as macros;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
