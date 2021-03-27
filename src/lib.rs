pub mod pluginv2 {
    tonic::include_proto!("pluginv2");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
